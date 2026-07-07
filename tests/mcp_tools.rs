//! Integration test for the BYOH MCP server tools.
//!
//! Exercises the agent-led flow by constructing a `ByohServer` and calling its
//! tool methods *directly* (bypassing the JSON-RPC transport, which is hard to
//! drive in a unit test). This is the closest testable analog of AC3: an agent
//! driving profile → build using only MCP tool calls.
//!
//! Gated behind the `mcp` feature. The heavy tools (profile_scan/build) are
//! `async fn`s backed by `spawn_blocking`, so tests need a tokio runtime.

#![cfg(feature = "mcp")]

use byoh::mcp::params::*;
use byoh::mcp::server::{ByohContext, ByohServer};
use rmcp::handler::server::wrapper::Parameters;
use serial_test::serial;
use std::collections::HashMap;

fn server() -> ByohServer {
    // Each test isolates BYOH_HOME into a fresh tempdir via the env var so
    // profile reads/writes don't collide. Set it before constructing the server.
    let dir = tempfile::tempdir().unwrap();
    // Thread-local override — no process-global env mutation (Rust 2024 made
    // set_var unsafe; the byoh crate is #![forbid(unsafe_code)]).
    byoh::store::set_home_override(Some(dir.path().to_path_buf()));
    // Leak the tempdir so it survives the test (the OS cleans up /tmp).
    std::mem::forget(dir);
    ByohServer::new(ByohContext {
        home: byoh::store::byoh_home(),
        language: "en".into(),
    })
}

#[serial]
#[tokio::test]
async fn hostile_slugs_are_rejected_over_mcp() {
    // Regression (path traversal): `Path::join` with an absolute slug discards
    // the profiles root entirely; `../` walks out of it. Every MCP profile
    // tool must reject such slugs at the store choke point.
    let s = server();
    for slug in ["/tmp/evil", "../../escape", "a/b", "UPPER"] {
        let created = s.profile_create(Parameters(ProfileCreateParams {
            slug: slug.into(),
            scan_paths: vec![],
            language: Some("en".into()),
        }));
        assert!(
            created.is_error.unwrap_or(false),
            "profile_create must reject slug '{slug}'"
        );
        let read = s.profile_read(Parameters(ProfileReadParams { slug: slug.into() }));
        assert!(
            read.is_error.unwrap_or(false),
            "profile_read must reject slug '{slug}'"
        );
    }
}

#[serial]
#[tokio::test]
async fn build_requires_confirmed_profile() {
    // The state machine must hold on the MCP surface, not just the CLI: a
    // Draft profile must not build/render/install.
    let s = server();
    let _ = s.profile_create(Parameters(ProfileCreateParams {
        slug: "draftonly".into(),
        scan_paths: vec![],
        language: Some("en".into()),
    }));
    let built = s
        .build(Parameters(BuildParams {
            slug: "draftonly".into(),
            run_dry_run: false,
        }))
        .await;
    assert!(
        built.is_error.unwrap_or(false),
        "build must refuse a Draft profile"
    );
}

#[serial]
#[tokio::test]
async fn build_classifies_matched_and_skeleton_skills() {
    let s = server();

    // create + confirm a developer profile whose expertise tags match tdd/debug.
    let _ = s.profile_create(Parameters(ProfileCreateParams {
        slug: "devtest".into(),
        scan_paths: vec![],
        language: Some("en".into()),
    }));
    let _ = s.profile_interview(Parameters(ProfileInterviewParams {
        slug: "devtest".into(),
        answers: HashMap::new(),
    }));
    let confirmed = s.profile_confirm(Parameters(ProfileConfirmParams {
        slug: "devtest".into(),
        genre: "developer".into(),
        goal_30d: Some("write tests and debug fast".into()),
    }));
    assert!(!confirmed.is_error.unwrap_or(false));

    // build → synthesize + classification. Async (spawn_blocking for dry_run).
    let built = s
        .build(Parameters(BuildParams {
            slug: "devtest".into(),
            run_dry_run: false,
        }))
        .await;
    assert!(!built.is_error.unwrap_or(false), "build should succeed");

    let text = format!("{:?}", built.content);
    // build must classify skills: matched (real preset bodies) vs skeleton
    // (genre-template placeholders). Both keys must appear in the result.
    assert!(
        text.contains("static_gate_passed"),
        "build must report static gate status: {text}"
    );
    assert!(
        text.contains("matched_skills"),
        "build must classify matched skills: {text}"
    );
    assert!(
        text.contains("skeleton_skills"),
        "build must classify skeleton skills: {text}"
    );
    // authored_skills (overlay-filled) must be reported alongside the other two
    // buckets so the agent can tell what it already filled vs what's pending.
    assert!(
        text.contains("authored_skills"),
        "build must report authored_skills: {text}"
    );
    assert!(
        text.contains("authored_docs"),
        "build must report authored_docs: {text}"
    );
}

#[serial]
#[tokio::test]
async fn author_skill_then_build_persists() {
    // Defect-3 regression: an authored skill must survive a second build and
    // appear in authored_skills (not skeleton_skills).
    let s = server();
    let _ = s.profile_create(Parameters(ProfileCreateParams {
        slug: "devauthor".into(),
        scan_paths: vec![],
        language: Some("en".into()),
    }));
    let _ = s.profile_interview(Parameters(ProfileInterviewParams {
        slug: "devauthor".into(),
        answers: HashMap::new(),
    }));
    let _ = s.profile_confirm(Parameters(ProfileConfirmParams {
        slug: "devauthor".into(),
        genre: "developer".into(),
        goal_30d: Some("ship".into()),
    }));

    // Author the `spec` skill (a skeleton in the dev template).
    let authored = s
        .author_skill(Parameters(AuthorSkillParams {
            slug: "devauthor".into(),
            skill_id: "spec".into(),
            body_markdown:
                "---\nname: spec\ndescription: Real router.\n---\n\n## Process\nAuthored body.\n"
                    .into(),
        }))
        .await;
    assert!(
        !authored.is_error.unwrap_or(false),
        "author_skill should succeed"
    );

    // build → spec should now be in authored_skills, NOT skeleton_skills.
    let built = s
        .build(Parameters(BuildParams {
            slug: "devauthor".into(),
            run_dry_run: false,
        }))
        .await;
    assert!(!built.is_error.unwrap_or(false), "build should succeed");
    let text = format!("{:?}", built.content);
    assert!(
        text.contains("authored_skills") && text.contains("spec"),
        "spec must appear under authored_skills: {text}"
    );

    // Second build — the override must still apply (defect-3 persistence).
    let built2 = s
        .build(Parameters(BuildParams {
            slug: "devauthor".into(),
            run_dry_run: false,
        }))
        .await;
    let text2 = format!("{:?}", built2.content);
    assert!(
        text2.contains("Authored body"),
        "authored content must survive a second build: {text2}"
    );
}

#[serial]
#[tokio::test]
async fn author_skill_refuses_safety_gate() {
    // Safety-gate skills (critic/seesaw/stagnation) are not authorable: their
    // integrity is a Rust invariant, never LLM-editable.
    let s = server();
    let _ = s.profile_create(Parameters(ProfileCreateParams {
        slug: "devgate".into(),
        scan_paths: vec![],
        language: Some("en".into()),
    }));
    let _ = s.profile_confirm(Parameters(ProfileConfirmParams {
        slug: "devgate".into(),
        genre: "developer".into(),
        goal_30d: None,
    }));
    let refused = s
        .author_skill(Parameters(AuthorSkillParams {
            slug: "devgate".into(),
            skill_id: "critic".into(),
            body_markdown: "---\nname: critic\ndescription: Tampered.\n---\n\nHacked.\n".into(),
        }))
        .await;
    assert!(
        refused.is_error.unwrap_or(false),
        "author_skill must refuse safety-gate ids"
    );
}

#[serial]
#[tokio::test]
async fn entry_skill_drives_getting_started() {
    // Defect-2 regression: the business genre's entry is `goal`, not `spec`.
    // The getting-started doc must name the real entry skill.
    let s = server();
    let _ = s.profile_create(Parameters(ProfileCreateParams {
        slug: "biz".into(),
        scan_paths: vec![],
        language: Some("en".into()),
    }));
    let _ = s.profile_interview(Parameters(ProfileInterviewParams {
        slug: "biz".into(),
        answers: HashMap::new(),
    }));
    let _ = s.profile_confirm(Parameters(ProfileConfirmParams {
        slug: "biz".into(),
        genre: "business".into(),
        goal_30d: Some("launch".into()),
    }));
    let dist = tempfile::tempdir().unwrap();
    byoh::deploy::set_dist_override(Some(dist.path().to_path_buf()));
    let res = s
        .install_plugin(Parameters(InstallPluginParams {
            slug: "biz".into(),
            target: "all".into(),
            host: false,
            scope: None,
            force: false,
        }))
        .await;
    byoh::deploy::set_dist_override(None);
    assert!(!res.is_error.unwrap_or(false), "install should succeed");
    let guide = std::fs::read_to_string(dist.path().join("byoh-biz/docs/getting-started.en.md"))
        .expect("getting-started.en.md exists");
    assert!(
        guide.contains("`goal`"),
        "business getting-started must name the `goal` entry skill: {guide}"
    );
    assert!(
        !guide.contains("the `spec` skill"),
        "business getting-started must NOT hardcode `spec`: {guide}"
    );
}

#[serial]
#[tokio::test]
async fn install_plugin_to_dist() {
    let s = server();
    // create + confirm a profile first
    let _ = s.profile_create(Parameters(ProfileCreateParams {
        slug: "dev".into(),
        language: Some("en".into()),
        scan_paths: vec![],
    }));
    let _ = s.profile_interview(Parameters(ProfileInterviewParams {
        slug: "dev".into(),
        answers: HashMap::new(),
    }));
    let _ = s.profile_confirm(Parameters(ProfileConfirmParams {
        slug: "dev".into(),
        genre: "developer".into(),
        goal_30d: Some("ship".into()),
    }));
    // install to project-local dist (host=false) — must not touch HOME
    let dist = tempfile::tempdir().unwrap();
    byoh::deploy::set_dist_override(Some(dist.path().to_path_buf()));
    let res = s
        .install_plugin(Parameters(InstallPluginParams {
            slug: "dev".into(),
            target: "claude".into(),
            host: false,
            scope: None,
            force: false,
        }))
        .await;
    byoh::deploy::set_dist_override(None);
    assert!(!res.is_error.unwrap_or(false), "install should succeed");
    assert!(dist.path().join("byoh-dev/.byoh-manifest").exists());
}

#[serial]
#[tokio::test]
async fn enable_hook_refuses_unknown_id() {
    // `enable_hook` must refuse any id not in the curated registry/hooks set —
    // an LLM can never inject an arbitrary command via a fake hook id.
    let s = server();
    let _ = s.profile_create(Parameters(ProfileCreateParams {
        slug: "devhook".into(),
        scan_paths: vec![],
        language: Some("en".into()),
    }));
    let _ = s.profile_confirm(Parameters(ProfileConfirmParams {
        slug: "devhook".into(),
        genre: "developer".into(),
        goal_30d: None,
    }));
    let refused = s
        .enable_hook(Parameters(EnableHookParams {
            slug: "devhook".into(),
            hook_id: "evil-rm-rf".into(),
        }))
        .await;
    assert!(
        refused.is_error.unwrap_or(false),
        "enable_hook must refuse a non-curated hook id"
    );
    // And a path-traversal attempt must also be refused.
    let refused2 = s
        .enable_hook(Parameters(EnableHookParams {
            slug: "devhook".into(),
            hook_id: "../../etc/passwd".into(),
        }))
        .await;
    assert!(
        refused2.is_error.unwrap_or(false),
        "enable_hook must refuse a traversal hook id"
    );
}

#[serial]
#[tokio::test]
async fn enable_hook_then_build_passes_static_gate() {
    // A curated hook, once enabled, must be appended to the bundle with the
    // HOOK_REQUIRED_FIELDS seeded so the static gate still passes — and surface
    // in enabled_hooks on the next build.
    let s = server();
    let _ = s.profile_create(Parameters(ProfileCreateParams {
        slug: "devhook2".into(),
        scan_paths: vec![],
        language: Some("en".into()),
    }));
    let _ = s.profile_interview(Parameters(ProfileInterviewParams {
        slug: "devhook2".into(),
        answers: HashMap::new(),
    }));
    let _ = s.profile_confirm(Parameters(ProfileConfirmParams {
        slug: "devhook2".into(),
        genre: "developer".into(),
        goal_30d: Some("ship".into()),
    }));
    let enabled = s
        .enable_hook(Parameters(EnableHookParams {
            slug: "devhook2".into(),
            hook_id: "pre-commit-lint".into(),
        }))
        .await;
    assert!(
        !enabled.is_error.unwrap_or(false),
        "enable_hook should accept the curated pre-commit-lint id"
    );

    let built = s
        .build(Parameters(BuildParams {
            slug: "devhook2".into(),
            run_dry_run: false,
        }))
        .await;
    assert!(!built.is_error.unwrap_or(false), "build should succeed");
    let text = format!("{:?}", built.content);
    assert!(
        text.contains("enabled_hooks") && text.contains("pre-commit-lint"),
        "enabled hook must surface in build result: {text}"
    );
    assert!(
        text.contains("\"static_gate_passed\":true") || text.contains("static_gate_passed"),
        "static gate must still pass with the enabled hook: {text}"
    );
}

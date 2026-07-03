//! Integration test for the BYOH MCP server tools.
//!
//! Exercises the agent-led flow by constructing a `ByohServer` and calling its
//! tool methods *directly* (bypassing the JSON-RPC transport, which is hard to
//! drive in a unit test). This is the closest testable analog of AC3: an agent
//! driving profile → compile using only MCP tool calls.
//!
//! Gated behind the `mcp` feature. The heavy tools (profile_scan/dry_run) are
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
async fn genre_list_returns_four_genres() {
    let s = server();
    let res = s.genre_list();
    assert!(!res.is_error.unwrap_or(false));
    // Assert the promise in the name, not just "didn't crash": all 4 genres.
    let text = format!("{:?}", res.content);
    for g in ["developer", "creator", "researcher", "business"] {
        assert!(text.contains(g), "genre_list must include '{g}'");
    }
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
async fn compile_requires_confirmed_profile() {
    // The state machine must hold on the MCP surface, not just the CLI: a
    // Draft profile must not compile/render/install.
    let s = server();
    let _ = s.profile_create(Parameters(ProfileCreateParams {
        slug: "draftonly".into(),
        scan_paths: vec![],
        language: Some("en".into()),
    }));
    let compiled = s.compile(Parameters(CompileParams {
        slug: "draftonly".into(),
        run_static_gate: true,
    }));
    assert!(
        compiled.is_error.unwrap_or(false),
        "compile must refuse a Draft profile"
    );
}

#[serial]
#[tokio::test]
async fn agent_led_flow_create_confirm_compile_clone() {
    let s = server();

    // S1: create a draft profile.
    let created = s.profile_create(Parameters(ProfileCreateParams {
        slug: "devtest".into(),
        scan_paths: vec![],
        language: Some("en".into()),
    }));
    assert!(
        !created.is_error.unwrap_or(false),
        "profile_create should succeed"
    );

    // S3 confirm (skip the interview in this test — unanswered questions stay
    // open, and confirm advances a Draft profile through Interviewed itself).
    let _ = s.profile_interview(Parameters(ProfileInterviewParams {
        slug: "devtest".into(),
        answers: HashMap::new(),
    }));
    let confirmed = s.profile_confirm(Parameters(ProfileConfirmParams {
        slug: "devtest".into(),
        genre: "developer".into(),
        goal_30d: Some("ship faster".into()),
    }));
    assert!(
        !confirmed.is_error.unwrap_or(false),
        "profile_confirm should succeed"
    );

    // S4: compile → bundle + static gate.
    let compiled = s.compile(Parameters(CompileParams {
        slug: "devtest".into(),
        run_static_gate: true,
    }));
    assert!(
        !compiled.is_error.unwrap_or(false),
        "compile should succeed"
    );

    // Dry-run (deps missing → graceful fallback, not an error). Async (spawn_blocking).
    let dry = s
        .compile_dry_run(Parameters(CompileDryRunParams {
            slug: "devtest".into(),
        }))
        .await;
    assert!(
        !dry.is_error.unwrap_or(false),
        "dry_run should not hard-error"
    );

    // Clone a vetted preset skill into the bundle.
    let cloned = s.registry_clone_skill(Parameters(RegistryCloneSkillParams {
        genre: "developer".into(),
        skill_id: "tdd".into(),
        slug: "devtest".into(),
    }));
    assert!(
        !cloned.is_error.unwrap_or(false),
        "registry_clone_skill should succeed"
    );
}

#[serial]
#[test]
fn evolve_cycle_runs_under_safety_gates() {
    let s = server();
    let res = s.evolve_cycle(Parameters(EvolveCycleParams {
        slug: "dev".into(),
        genre: "developer".into(),
        edit_type: "AddSkill".into(),
        metric: EvolveMetricParams {
            with_: 0.8,
            without: 0.5,
            samples_with: 5,
            samples_without: 5,
        },
    }));
    assert!(!res.is_error.unwrap_or(false));
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

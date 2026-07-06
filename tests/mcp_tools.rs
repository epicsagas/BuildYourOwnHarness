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

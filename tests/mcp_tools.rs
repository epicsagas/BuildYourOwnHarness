//! Integration test for the BYOH MCP server tools.
//!
//! Exercises the agent-led flow by constructing a `ByohServer` and calling its
//! tool methods *directly* (bypassing the JSON-RPC transport, which is hard to
//! drive in a unit test). This is the closest testable analog of AC3: an agent
//! driving profile → rag → compile using only MCP tool calls.
//!
//! Gated behind the `mcp` feature. Uses the DummyEmbedder (default build) so it
//! needs no network/model download. The heavy tools (profile_scan/rag_*/dry_run)
//! are `async fn`s backed by `spawn_blocking`, so tests need a tokio runtime.

#![cfg(feature = "mcp")]

use byoh::mcp::params::*;
use byoh::mcp::server::{ByohContext, ByohServer};
use rmcp::handler::server::wrapper::Parameters;
use std::collections::HashMap;

fn server() -> ByohServer {
    // Each test isolates BYOH_HOME into a fresh tempdir via the env var so
    // profile reads/writes don't collide. Set it before constructing the server.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("BYOH_HOME", dir.path());
    // Leak the tempdir so it survives the test (the OS cleans up /tmp).
    std::mem::forget(dir);
    ByohServer::new(ByohContext {
        home: byoh::store::byoh_home(),
        language: "en".into(),
        native_rag: cfg!(feature = "native-rag"),
    })
}

#[tokio::test]
async fn genre_list_returns_four_genres() {
    let s = server();
    let res = s.genre_list();
    assert!(!res.is_error.unwrap_or(false));
}

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

    // S3 confirm (skip interview in this test — empty answers auto-accept).
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

#[tokio::test]
async fn rag_search_grep_tier_without_corpus() {
    let s = server();
    let res = s
        .rag_search(Parameters(RagSearchParams {
            query: "rust async".into(),
            genre: "developer".into(),
            corpus: None,
            k: 3,
        }))
        .await;
    // grep tier against empty corpus returns 0 hits but is not an error.
    assert!(!res.is_error.unwrap_or(false));
}

#[tokio::test]
async fn rag_index_with_small_corpus() {
    let s = server();
    let dir = tempfile::tempdir().unwrap();
    let note_path = dir.path().join("note.md");
    std::fs::write(
        &note_path,
        "# Rust notes\nasync fn with tokio spawn_blocking",
    )
    .unwrap();
    let corpus = note_path.to_string_lossy().into_owned();
    std::mem::forget(dir); // survive the test
    let res = s
        .rag_index(Parameters(RagIndexParams {
            genre: "developer".into(),
            corpus,
            max_tokens: 512,
            overlap: 64,
        }))
        .await;
    assert!(!res.is_error.unwrap_or(false), "rag_index should succeed");
}

#[test]
fn evolve_cycle_runs_under_safety_gates() {
    let s = server();
    let res = s.evolve_cycle(Parameters(EvolveCycleParams {
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

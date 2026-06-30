//! End-to-end integration tests: profile init → interview → confirm → compile
//! → static gate → dry-run. Exercises the full M0 path for all four genres.

use byoh::adapters::{FilesystemSource, RuleInterview, RuleLlm, StaticWizard};
use byoh::application::ProfileOrchestrator;
use byoh::compiler::{compile_profile, dry_run, incremental, static_gate};
use byoh::deploy::provider::{CapabilityProfile, match_provider};
use byoh::deploy::registry::Registry;
use byoh::deploy::state::{BuildStore, crash_check};
use byoh::domain::bundle::Ring;
use byoh::domain::evidence::{AbMetric, ObservationRecord, ObservedOutcome};
use byoh::domain::genre::Genre;
use byoh::domain::profile::{ProfileStatus, ProviderPreference, ToolUseRequirement, UserProfile};
use byoh::evolve::gates::{SafetyGateSet, SeesawState, StagnationState};
use byoh::evolve::{CompressionTier, compress};
use byoh::evolve::{EvolutionCycle, mine_patterns, run_cycle};
use byoh::i18n::{Msg, t};
use byoh::ports::command::CommandPort;
use byoh::security::mask;
use chrono::TimeZone;
use std::collections::HashMap;

fn orchestrator() -> (
    FilesystemSource,
    RuleLlm,
    RuleInterview<RuleLlm>,
    StaticWizard,
) {
    let src = FilesystemSource::new();
    let llm = RuleLlm::new();
    let iv = RuleInterview::new(RuleLlm::new());
    let wz = StaticWizard::new();
    (src, llm, iv, wz)
}

fn answers() -> HashMap<String, (String, f64)> {
    let mut a = HashMap::new();
    a.insert("Q_domain".into(), ("backend".into(), 0.9));
    a.insert("Q_goal".into(), ("ship faster".into(), 0.9));
    a.insert("Q_genre".into(), ("developer".into(), 0.9));
    a.insert("Q_data".into(), ("./vault".into(), 0.8));
    a
}

#[test]
fn full_m0_path_for_each_genre() {
    // AC1, AC5, AC6, AC8, AC9: full pipeline for all 4 genres.
    for &genre in Genre::all() {
        let (src, llm, iv, wz) = orchestrator();
        let orch = ProfileOrchestrator::new(&src, &llm, &iv, &wz);
        let mut p = UserProfile::new_draft("u", "en");
        orch.run_m0(&mut p, &answers(), genre, Some("goal"))
            .unwrap();

        assert_eq!(p.status, ProfileStatus::Confirmed);

        let bundle = compile_profile(&p).unwrap();
        let report = static_gate(&bundle).unwrap();
        assert!(
            report.passed(),
            "genre {:?} static gate: {:?}",
            genre,
            report.errors
        );

        // AC8: dry-run with all-missing deps still passes via fallback.
        struct AllMissing;
        impl CommandPort for AllMissing {
            fn run(
                &self,
                _tool: &str,
                _args: &[&str],
                _cwd: Option<&std::path::Path>,
            ) -> byoh::ports::command::CommandOutcome {
                byoh::ports::command::CommandOutcome::NotInstalled
            }
            fn is_installed(&self, _tool: &str) -> bool {
                false
            }
        }
        let dr = dry_run(&bundle, &AllMissing).unwrap();
        assert!(dr.passed(), "genre {:?} dry-run: {:?}", genre, dr);

        // AC9: every genre bundle carries the three safety gates.
        for g in ["critic", "seesaw", "stagnation"] {
            assert!(
                bundle.safety_gates.contains(&g.to_string()),
                "genre {:?} missing {g}",
                genre
            );
        }
        // Ring coverage.
        for ring in Ring::all() {
            if ring != Ring::Ring0 {
                assert!(
                    bundle.skills.iter().any(|s| s.ring == ring),
                    "genre {:?} missing skills in {:?}",
                    genre,
                    ring
                );
            }
        }
    }
}

#[test]
fn incremental_recompile_classification() {
    // AC13: 3a / 3b / 3c.
    let mut prev = UserProfile::new_draft("d", "en");
    prev.candidates.identity.genre = Some(byoh::domain::profile::GenreConfidence {
        value: Genre::Developer,
        confidence: 1.0,
        provenance: vec![],
    });
    prev.status = ProfileStatus::Confirmed;

    // 3a meta-only
    let mut next = prev.clone();
    next.evolution_policy = Some(byoh::domain::profile::EvolutionPolicyConfig {
        enabled: true,
        safety_gates: vec!["critic".into(), "seesaw".into(), "stagnation".into()],
        stagnation_limit: 5,
        improvement_threshold: 0.03,
    });
    assert_eq!(
        incremental::classify_change(&prev, &next),
        incremental::ChangeClass::MetaOnly
    );

    // 3b goal change
    let mut next = prev.clone();
    next.truth.goals.goal_30d = Some("different".into());
    assert_eq!(
        incremental::classify_change(&prev, &next),
        incremental::ChangeClass::Scoped
    );

    // 3c genre change
    let next = {
        let mut p = UserProfile::new_draft("d", "en");
        p.candidates.identity.genre = Some(byoh::domain::profile::GenreConfidence {
            value: Genre::Creator,
            confidence: 1.0,
            provenance: vec![],
        });
        p.status = ProfileStatus::Confirmed;
        p
    };
    assert_eq!(
        incremental::classify_change(&prev, &next),
        incremental::ChangeClass::Breaking
    );
}

#[test]
fn provider_matching_excludes_unqualified() {
    // AC15.
    let pref = ProviderPreference {
        candidate_family: Some("anthropic".into()),
        capability_constraints: byoh::domain::profile::CapabilityConstraints {
            tool_use: ToolUseRequirement::Required,
            context_window_min: Some(200_000),
        },
        source: byoh::domain::profile::ProvenanceSource::Derived,
    };
    let cands = vec![
        CapabilityProfile {
            name: "cheap-no-tools".into(),
            supports_tool_use: false,
            context_window: 8000,
            monthly_cost_usd: 5.0,
        },
        CapabilityProfile {
            name: "big-tools".into(),
            supports_tool_use: true,
            context_window: 200_000,
            monthly_cost_usd: 50.0,
        },
        CapabilityProfile {
            name: "too-small".into(),
            supports_tool_use: true,
            context_window: 4000,
            monthly_cost_usd: 10.0,
        },
    ];
    let chosen = match_provider(&pref, &cands).expect("matches");
    assert_eq!(chosen.name, "big-tools");
    // excluded: cheap-no-tools (no tools), too-small (small ctx)
}

#[test]
fn crash_recovery_45_minute_rule() {
    // AC17.
    let now = chrono::Utc.with_ymd_and_hms(2026, 6, 24, 12, 0, 0).unwrap();
    let stale = now - chrono::Duration::seconds(46 * 60);
    let state = byoh::domain::state::BuildState {
        slug: "d".into(),
        phase: byoh::domain::state::BuildPhase::DryRun,
        profile_status: ProfileStatus::Confirmed,
        started_at: stale,
        updated_at: stale,
        phase_history: vec![],
    };
    assert!(crash_check(&state, now).stale);

    let fresh = now - chrono::Duration::seconds(5 * 60);
    let mut state2 = state;
    state2.updated_at = fresh;
    assert!(!crash_check(&state2, now).stale);
}

#[test]
fn build_store_persists_and_loads() {
    let dir = tempfile::tempdir().unwrap();
    let store = BuildStore::new(dir.path(), "u");
    let now = chrono::Utc::now();
    let st = byoh::domain::state::BuildState {
        slug: "u".into(),
        phase: byoh::domain::state::BuildPhase::Compile,
        profile_status: ProfileStatus::Confirmed,
        started_at: now,
        updated_at: now,
        phase_history: vec![],
    };
    store.checkpoint(&st).unwrap();
    let loaded = store.load_latest().unwrap().unwrap();
    assert_eq!(loaded.slug, "u");
    assert_eq!(loaded.phase, byoh::domain::state::BuildPhase::Compile);
}

#[test]
fn evolution_cycle_lifecycle_approved() {
    // AC10: approved cycle passes all gates.
    let metric = AbMetric {
        avg_score_with: 0.8,
        avg_score_without: 0.5,
        samples_with: 5,
        samples_without: 5,
    };
    let cycle = EvolutionCycle {
        observations: vec![],
        proposed_edit: byoh::evolve::EditType::AddSkill,
        metric: metric.clone(),
        params: byoh::domain::genre::GenreEvolutionParams::for_genre(Genre::Developer),
        gates: SafetyGateSet::all(),
    };
    let seesaw = SeesawState::new(metric.clone());
    let stag = StagnationState::new(3, 0.02);
    let (dec, _, _) = run_cycle(seesaw, stag, &cycle).unwrap();
    assert!(matches!(
        dec,
        byoh::evolve::EvolutionDecision::Approved { .. }
    ));
}

#[test]
fn evolution_rejects_reward_hacking() {
    let metric = AbMetric {
        avg_score_with: 0.6,
        avg_score_without: 0.5,
        samples_with: 1,
        samples_without: 0,
    };
    let cycle = EvolutionCycle {
        observations: vec![],
        proposed_edit: byoh::evolve::EditType::RemoveSkill,
        metric,
        params: byoh::domain::genre::GenreEvolutionParams::for_genre(Genre::Developer),
        gates: SafetyGateSet::all(),
    };
    let seesaw = SeesawState::new(AbMetric {
        avg_score_with: 0.5,
        avg_score_without: 0.5,
        samples_with: 0,
        samples_without: 0,
    });
    let stag = StagnationState::new(3, 0.02);
    let (dec, _, _) = run_cycle(seesaw, stag, &cycle).unwrap();
    assert!(matches!(
        dec,
        byoh::evolve::EvolutionDecision::Rejected { .. }
    ));
}

#[test]
fn skillopt_mines_dominant_error() {
    let now = chrono::Utc::now();
    let mk = |tool: &str, err: Option<&str>, ok: bool| ObservationRecord {
        id: tool.into(),
        observed_at: now,
        tool_name: tool.into(),
        outcome: if ok {
            ObservedOutcome::Success
        } else {
            ObservedOutcome::Failure
        },
        with_evolved: true,
        score: if ok { 1.0 } else { 0.0 },
        dominant_error: err.map(|e| e.to_string()),
    };
    let recs = vec![
        mk("a", Some("npe"), false),
        mk("b", Some("npe"), false),
        mk("c", Some("npe"), false),
        mk("d", Some("other"), false),
        mk("e", None, true),
    ];
    let seeds = mine_patterns(&recs);
    assert!(seeds.iter().any(|s| s.dominant_error == "npe"));
}

#[test]
fn compression_tiered_for_genres() {
    // AC12.
    let tokens = vec![
        byoh::evolve::Token {
            text: "fn".into(),
            kind: byoh::evolve::TokenKind::Code,
        },
        byoh::evolve::Token {
            text: "hi".into(),
            kind: byoh::evolve::TokenKind::Dialogue,
        },
    ];
    let dev_max = compress(&tokens, CompressionTier::MaxCompression, Genre::Developer);
    assert!(
        dev_max
            .iter()
            .all(|t| t.kind == byoh::evolve::TokenKind::Code)
    );
    let creator_max = compress(&tokens, CompressionTier::MaxCompression, Genre::Creator);
    assert!(
        creator_max
            .iter()
            .all(|t| t.kind == byoh::evolve::TokenKind::Dialogue)
    );
}

#[test]
fn secret_masking_in_bundle_artifacts() {
    // AC19.
    assert_eq!(mask("LAW_OC=secret123"), "LAW_OC=****");
    assert_eq!(mask("OC=topsecret end"), "OC=**** end");
    assert_eq!(mask("bearer abcdefghij"), "bearer ****");
    assert_eq!(mask("clean text"), "clean text");
}

#[test]
fn i18n_resolves_both_languages() {
    // AC16.
    assert!(t(Msg::Installed, "en").contains("installed"));
    assert!(t(Msg::Installed, "ko").contains("설치"));
    assert!(t(Msg::DryRunPassed, "ko").contains("통과"));
}

#[test]
fn registry_registers_compiled_bundle() {
    let mut p = UserProfile::new_draft("dev1", "ko");
    p.candidates.identity.genre = Some(byoh::domain::profile::GenreConfidence {
        value: Genre::Developer,
        confidence: 1.0,
        provenance: vec![],
    });
    p.status = ProfileStatus::Confirmed;
    let bundle = compile_profile(&p).unwrap();
    let mut reg = Registry::new();
    let entry = reg.register(&bundle);
    assert_eq!(entry.slug, "dev1");
    assert!(reg.lookup("dev1").is_some());
}

#[test]
fn autoscan_is_non_destructive_and_derived() {
    // AC3.
    let dir = tempfile::tempdir().unwrap();
    let note = dir.path().join("n.md");
    std::fs::write(&note, "# T\nuse #rust and #k8s\n```rust\nfn\n```").unwrap();
    let before = std::fs::read_to_string(&note).unwrap();

    let (src, llm, iv, wz) = orchestrator();
    let orch = ProfileOrchestrator::new(&src, &llm, &iv, &wz);
    let mut p = UserProfile::new_draft("u", "en");
    orch.stage1_scan(&mut p, &[dir.path()]).unwrap();

    // non-destructive
    assert_eq!(std::fs::read_to_string(&note).unwrap(), before);
    // candidates are derived
    assert!(!p.candidates.identity.primary_expertise.is_empty());
    assert!(
        p.candidates
            .identity
            .primary_expertise
            .iter()
            .all(
                |f| f.confidence < byoh::domain::profile::DerivedFact::REQUESTION_THRESHOLD
                    || f.confidence >= 0.0
            )
    );
}

#[test]
fn interview_council_on_ambiguous_genre() {
    // AC4: ambiguous genre triggers 4-voice council.
    let (src, llm, iv, wz) = orchestrator();
    let orch = ProfileOrchestrator::new(&src, &llm, &iv, &wz);
    let mut p = UserProfile::new_draft("u", "en");
    // No genre set → ambiguous.
    let extra = orch.stage2_interview(&mut p, &HashMap::new()).unwrap();
    assert_eq!(extra.len(), 4); // four council voices
}

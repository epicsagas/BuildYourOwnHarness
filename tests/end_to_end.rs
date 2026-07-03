//! End-to-end integration tests: profile init → interview → confirm → compile
//! → static gate → dry-run. Exercises the full M0 path for all four genres.

use byoh::adapters::{FilesystemSource, RuleInterview, RuleLlm};
use byoh::application::ProfileOrchestrator;
use byoh::compiler::{compile_profile, dry_run, static_gate};
use byoh::deploy::provider::{CapabilityProfile, match_provider};
use byoh::domain::bundle::Ring;
use byoh::domain::evidence::AbMetric;
use byoh::domain::genre::Genre;
use byoh::domain::profile::{ProfileStatus, ProviderPreference, ToolUseRequirement, UserProfile};
use byoh::evolve::gates::{SafetyGateSet, SeesawState, StagnationState};
use byoh::evolve::{EvolutionCycle, run_cycle};
use byoh::i18n::{Msg, t};
use byoh::ports::command::CommandPort;
use byoh::security::mask;
use std::collections::HashMap;

fn orchestrator() -> (FilesystemSource, RuleLlm, RuleInterview) {
    let src = FilesystemSource::new();
    let llm = RuleLlm::new();
    let iv = RuleInterview::new();
    (src, llm, iv)
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
        let (src, llm, iv) = orchestrator();
        let orch = ProfileOrchestrator::new(&src, &llm, &iv);
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
fn autoscan_is_non_destructive_and_derived() {
    // AC3.
    let dir = tempfile::tempdir().unwrap();
    let note = dir.path().join("n.md");
    std::fs::write(&note, "# T\nuse #rust and #k8s\n```rust\nfn\n```").unwrap();
    let before = std::fs::read_to_string(&note).unwrap();

    let (src, llm, iv) = orchestrator();
    let orch = ProfileOrchestrator::new(&src, &llm, &iv);
    let mut p = UserProfile::new_draft("u", "en");
    orch.stage1_scan(&mut p, &[dir.path()]).unwrap();

    // non-destructive
    assert_eq!(std::fs::read_to_string(&note).unwrap(), before);
    // candidates are derived, and every scan-derived confidence sits below the
    // re-question threshold so the interview re-asks rather than trusting scans.
    assert!(!p.candidates.identity.primary_expertise.is_empty());
    assert!(
        p.candidates
            .identity
            .primary_expertise
            .iter()
            .all(|f| f.confidence < byoh::domain::profile::DerivedFact::REQUESTION_THRESHOLD)
    );
}

#[test]
fn interview_council_on_ambiguous_genre() {
    // AC4: ambiguous genre triggers 4-voice council.
    let (src, llm, iv) = orchestrator();
    let orch = ProfileOrchestrator::new(&src, &llm, &iv);
    let mut p = UserProfile::new_draft("u", "en");
    // No genre set → ambiguous.
    let extra = orch.stage2_interview(&mut p, &HashMap::new()).unwrap();
    assert_eq!(extra.len(), 4); // four council voices
}

#[test]
fn interview_partial_answers_terminate() {
    // Regression: answering a strict subset of the open questions used to spin
    // the S2 loop forever (and hang the MCP server, which calls it sync).
    let (src, llm, iv) = orchestrator();
    let orch = ProfileOrchestrator::new(&src, &llm, &iv);
    let mut p = UserProfile::new_draft("u", "en");
    let mut one = HashMap::new();
    one.insert("Q_goal".into(), ("ship the payments api".into(), 0.9));
    orch.stage2_interview(&mut p, &one).unwrap();
    assert_eq!(
        p.truth.goals.goal_30d.as_deref(),
        Some("ship the payments api")
    );
    // Domain/genre remain open — nothing was auto-accepted.
    assert!(p.truth.identity.domain.is_none());
    assert!(p.candidates.identity.genre.is_none());
}

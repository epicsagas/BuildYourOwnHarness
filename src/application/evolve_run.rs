//! Evolution orchestrator — load state → run one cycle → persist.
//!
//! Wraps the pure `evolve::run_cycle` with file-backed [`EvolveStore`] state so
//! the seesaw/stagnation gates carry across `byoh evolve` runs. Shared by the
//! CLI (`run_evolve`) and the MCP `evolve_cycle` tool — neither duplicates the
//! load/run/save logic.

use std::path::Path;

use crate::domain::evidence::AbMetric;
use crate::domain::genre::{Genre, GenreEvolutionParams};
use crate::evolve::gates::{SafetyGateSet, SeesawState, StagnationState};
use crate::evolve::state::{EvolveState, EvolveStore};
use crate::evolve::{run_cycle, EditType, EvolutionCycle, EvolutionDecision};
use crate::store::sanitize_slug;
use crate::Result;

/// Parse an `EditType` from its string name (the CLI/MCP wire form).
pub fn parse_edit_type(s: &str) -> Result<EditType> {
    Ok(match s {
        "AddSkill" => EditType::AddSkill,
        "ModifyInstinct" => EditType::ModifyInstinct,
        "ModifyConfig" => EditType::ModifyConfig,
        "AddGuardRule" => EditType::AddGuardRule,
        "ModifyPrompt" => EditType::ModifyPrompt,
        "RemoveSkill" => EditType::RemoveSkill,
        other => {
            return Err(crate::domain::error::ByohError::Schema(format!(
                "unknown edit_type '{other}' (AddSkill|ModifyInstinct|ModifyConfig|AddGuardRule|ModifyPrompt|RemoveSkill)"
            )))
        }
    })
}

/// Run one evolution cycle for `slug`, persisting state under `root`.
///
/// - First run (no state file) seeds a baseline from the genre's evolution params.
/// - Subsequent runs load the prior seesaw/stagnation state so the gates see the
///   metric *history*.
/// - State is saved regardless of the decision (the seesaw/stagnation counters
///   must advance even on a rollback).
///
/// Returns the honest `EvolutionDecision` and the new persisted state.
pub fn evolve_one_cycle(
    root: &Path,
    slug: &str,
    genre: Genre,
    edit: EditType,
    metric: AbMetric,
) -> Result<(EvolutionDecision, EvolveState)> {
    let slug = sanitize_slug(slug)?;
    let store = EvolveStore::new(root, slug);
    let params = GenreEvolutionParams::for_genre(genre);

    // Load prior state or seed a baseline (explicit, not "empty == approved").
    let prior = store.load()?.unwrap_or_else(|| {
        EvolveState::baseline(params.stagnation_limit, params.improvement_threshold)
    });

    let seesaw: SeesawState = prior.seesaw.clone();
    let stagnation: StagnationState = prior.stagnation.clone();

    let cycle = EvolutionCycle {
        observations: vec![],
        proposed_edit: edit,
        metric,
        params,
        gates: SafetyGateSet::all(),
    };

    let (decision, new_seesaw, new_stagnation) = run_cycle(seesaw, stagnation, &cycle)?;

    let new_state = EvolveState {
        version: crate::evolve::state::STATE_VERSION,
        seesaw: new_seesaw,
        stagnation: new_stagnation,
        cycle_n: prior.cycle_n + 1,
    };
    store.save(&new_state)?;

    Ok((decision, new_state))
}

/// Human label for a decision (CLI/MCP shared).
pub fn decision_label(d: &EvolutionDecision) -> &'static str {
    match d {
        EvolutionDecision::Approved { .. } => "Approved",
        EvolutionDecision::Rejected { .. } => "Rejected",
        EvolutionDecision::RolledBack { .. } => "RolledBack",
        EvolutionDecision::AutoTuned => "AutoTuned",
    }
}

/// Whether a decision is a "negative" outcome (CLI exits non-zero on these).
pub fn decision_is_negative(d: &EvolutionDecision) -> bool {
    matches!(
        d,
        EvolutionDecision::Rejected { .. } | EvolutionDecision::RolledBack { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strong_metric() -> AbMetric {
        AbMetric {
            avg_score_with: 0.85,
            avg_score_without: 0.5,
            samples_with: 8,
            samples_without: 8,
        }
    }

    #[test]
    fn first_cycle_seeds_baseline_and_persists() {
        let dir = tempfile::tempdir().unwrap();
        let (_decision, state) = evolve_one_cycle(
            dir.path(),
            "dev",
            Genre::Developer,
            EditType::AddSkill,
            strong_metric(),
        )
        .unwrap();
        assert_eq!(state.cycle_n, 1);
        // state file exists for the next run
        let reloaded = EvolveStore::new(dir.path(), "dev").load().unwrap().unwrap();
        assert_eq!(reloaded.cycle_n, 1);
    }

    #[test]
    fn cycle_n_advances_across_runs() {
        let dir = tempfile::tempdir().unwrap();
        for expected in 1..=3 {
            let (_d, state) = evolve_one_cycle(
                dir.path(),
                "dev",
                Genre::Developer,
                EditType::AddSkill,
                strong_metric(),
            )
            .unwrap();
            assert_eq!(state.cycle_n, expected);
        }
    }

    #[test]
    fn rejects_bad_slug() {
        let dir = tempfile::tempdir().unwrap();
        assert!(evolve_one_cycle(
            dir.path(),
            "../evil",
            Genre::Developer,
            EditType::AddSkill,
            strong_metric()
        )
        .is_err());
    }

    #[test]
    fn parse_edit_type_known_and_unknown() {
        assert!(parse_edit_type("AddSkill").is_ok());
        assert!(parse_edit_type("Nonsense").is_err());
    }
}

//! Evolution lifecycle — Observe → Analyze → Evolve → Gate (ARCH §7.1).

use crate::domain::evidence::{AbMetric, ObservationRecord};
use crate::domain::genre::GenreEvolutionParams;
use crate::evolve::gates::{
    CriticVerdict, EditType, SafetyGateSet, SeesawState, StagnationAction, StagnationState,
    critic_review,
};

/// A single evolution cycle's inputs.
#[derive(Debug, Clone)]
pub struct EvolutionCycle {
    pub observations: Vec<ObservationRecord>,
    pub proposed_edit: EditType,
    pub metric: AbMetric,
    pub params: GenreEvolutionParams,
    pub gates: SafetyGateSet,
}

/// The cycle's decision.
#[derive(Debug, Clone, PartialEq)]
pub enum EvolutionDecision {
    Approved { critic: CriticVerdict },
    Rejected { reason: String },
    RolledBack { reason: String },
    AutoTuned,
}

/// Run one full cycle. All three gates must be present (R11).
pub fn run_cycle(
    mut seesaw: SeesawState,
    mut stagnation: StagnationState,
    cycle: &EvolutionCycle,
) -> crate::domain::Result<(EvolutionDecision, SeesawState, StagnationState)> {
    // Enforce all three gates (R11/AC10).
    cycle.gates.validate_all()?;

    // Analyze → metric observed by seesaw + stagnation.
    let seesaw_regression = seesaw.observe(&cycle.metric);
    let stagnation_action = stagnation.observe(&cycle.metric);

    // Evolve → gate.
    if matches!(stagnation_action, StagnationAction::Rollback) {
        return Ok((
            EvolutionDecision::RolledBack {
                reason: format!(
                    "stagnation: {} sessions without ≥{:.0}% improvement",
                    cycle.params.stagnation_limit,
                    cycle.params.improvement_threshold * 100.0
                ),
            },
            seesaw,
            stagnation,
        ));
    }
    if seesaw.catastrophic() {
        return Ok((
            EvolutionDecision::RolledBack {
                reason: "seesaw: catastrophic forgetting detected".into(),
            },
            seesaw,
            stagnation,
        ));
    }
    if seesaw_regression {
        return Ok((EvolutionDecision::AutoTuned, seesaw, stagnation));
    }

    let critic = critic_review(
        &cycle.proposed_edit,
        &cycle.metric,
        cycle.params.critic_weight,
    );
    if critic.approved {
        Ok((EvolutionDecision::Approved { critic }, seesaw, stagnation))
    } else {
        Ok((
            EvolutionDecision::Rejected {
                reason: critic.reason.clone(),
            },
            seesaw,
            stagnation,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::genre::Genre;

    fn metric(with: f64, without: f64, sw: u32) -> AbMetric {
        AbMetric {
            avg_score_with: with,
            avg_score_without: without,
            samples_with: sw,
            samples_without: 0,
        }
    }

    #[test]
    fn missing_gate_rejects_cycle() {
        let cycle = EvolutionCycle {
            observations: vec![],
            proposed_edit: EditType::AddSkill,
            metric: metric(0.6, 0.5, 1),
            params: GenreEvolutionParams::for_genre(Genre::Developer),
            gates: SafetyGateSet {
                critic: true,
                seesaw: true,
                stagnation: false,
            },
        };
        let seesaw = SeesawState::new(metric(0.5, 0.5, 0));
        let stag = StagnationState::new(3, 0.02);
        let err = run_cycle(seesaw, stag, &cycle).unwrap_err();
        assert!(matches!(
            err,
            crate::domain::ByohError::SafetyGateMissing { .. }
        ));
    }

    #[test]
    fn approved_cycle_passes_all_gates() {
        let cycle = EvolutionCycle {
            observations: vec![],
            proposed_edit: EditType::AddSkill,
            metric: metric(0.7, 0.5, 5),
            params: GenreEvolutionParams::for_genre(Genre::Developer),
            gates: SafetyGateSet::all(),
        };
        let seesaw = SeesawState::new(metric(0.5, 0.5, 0));
        let stag = StagnationState::new(3, 0.02);
        let (dec, _, _) = run_cycle(seesaw, stag, &cycle).unwrap();
        assert!(matches!(dec, EvolutionDecision::Approved { .. }));
    }

    #[test]
    fn reward_hacking_edit_rejected() {
        let cycle = EvolutionCycle {
            observations: vec![],
            proposed_edit: EditType::RemoveSkill,
            metric: metric(0.6, 0.5, 1),
            params: GenreEvolutionParams::for_genre(Genre::Developer),
            gates: SafetyGateSet::all(),
        };
        let seesaw = SeesawState::new(metric(0.5, 0.5, 0));
        let stag = StagnationState::new(3, 0.02);
        let (dec, _, _) = run_cycle(seesaw, stag, &cycle).unwrap();
        assert!(matches!(dec, EvolutionDecision::Rejected { .. }));
    }
}

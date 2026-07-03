//! The 3 safety gates (ARCH §7.1, B10).
//!
//! - **Critic** — deterministic, in-loop reward-hacking defense.
//! - **Seesaw** — catastrophic-forgetting regression detection.
//! - **Stagnation** — N-session no-improvement → auto-rollback.
//!
//! ALL THREE are mandatory (R11). The compiler's static gate and the evolution
//! lifecycle both enforce their presence.

use serde::{Deserialize, Serialize};

use crate::domain::evidence::AbMetric;
use crate::domain::genre::SafetyGate;

/// The configured safety-gate set. Validates that all three are present.
#[derive(Debug, Clone, PartialEq)]
pub struct SafetyGateSet {
    pub critic: bool,
    pub seesaw: bool,
    pub stagnation: bool,
}

impl SafetyGateSet {
    pub fn all() -> Self {
        Self {
            critic: true,
            seesaw: true,
            stagnation: true,
        }
    }

    pub fn from_names(names: &[String]) -> Self {
        Self {
            critic: names.iter().any(|n| n == SafetyGate::Critic.as_str()),
            seesaw: names.iter().any(|n| n == SafetyGate::Seesaw.as_str()),
            stagnation: names.iter().any(|n| n == SafetyGate::Stagnation.as_str()),
        }
    }

    /// R11: every gate present, else error naming the missing one.
    pub fn validate_all(&self) -> crate::domain::Result<()> {
        if !self.critic {
            return Err(crate::domain::ByohError::SafetyGateMissing {
                gate: SafetyGate::Critic.as_str(),
            });
        }
        if !self.seesaw {
            return Err(crate::domain::ByohError::SafetyGateMissing {
                gate: SafetyGate::Seesaw.as_str(),
            });
        }
        if !self.stagnation {
            return Err(crate::domain::ByohError::SafetyGateMissing {
                gate: SafetyGate::Stagnation.as_str(),
            });
        }
        Ok(())
    }
}

// ── Critic ────────────────────────────────────────────────────────────────

/// A proposed edit to the harness (epic-harness EditType subset, ARCH §7.4 ref).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditType {
    AddSkill,
    ModifyInstinct,
    ModifyConfig,
    AddGuardRule,
    ModifyPrompt,
    RemoveSkill,
}

/// Critic verdict on a proposed edit. Deterministic (no LLM) per ARCH §7.1
/// "deterministic in-loop".
#[derive(Debug, Clone, PartialEq)]
pub struct CriticVerdict {
    pub approved: bool,
    pub reason: String,
    /// Detected reward-hacking pattern, if any.
    pub reward_hacking: bool,
}

/// Critic: rejects edits that look like reward hacking (e.g. weakening a guard
/// rule, deleting a skill to game a metric, or config that disables evaluation).
pub fn critic_review(edit: &EditType, metric: &AbMetric, critic_weight: f64) -> CriticVerdict {
    let weight = critic_weight.max(0.0);
    // Reward-hacking heuristics: deleting a skill to game a metric, or an
    // implausible score jump on thin evidence.
    let hacking = matches!(edit, EditType::RemoveSkill)
        || (metric.samples_with > 0
            && metric.avg_score_with > 0.99
            && metric.avg_score_without < 0.5);

    if hacking {
        return CriticVerdict {
            approved: false,
            reason: "suspected reward hacking: removing a skill or implausible metric jump".into(),
            reward_hacking: true,
        };
    }
    // High-weight critic is stricter on instinct/prompt edits.
    let strict = weight >= 1.3;
    if strict && matches!(edit, EditType::ModifyInstinct | EditType::ModifyPrompt) {
        return CriticVerdict {
            approved: false,
            reason: "high-stakes edit requires human review (critic weight elevated)".into(),
            reward_hacking: false,
        };
    }
    CriticVerdict {
        approved: true,
        reason: "no reward-hacking pattern detected".into(),
        reward_hacking: false,
    }
}

// ── Seesaw ────────────────────────────────────────────────────────────────

/// Seesaw: tracks whether a recent improvement came at the cost of a prior one
/// (catastrophic forgetting). Regression vs the `without` baseline triggers it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeesawState {
    pub last_metric: AbMetric,
    pub regressions: u32,
}

impl SeesawState {
    pub fn new(metric: AbMetric) -> Self {
        Self {
            last_metric: metric,
            regressions: 0,
        }
    }

    /// Observe a new metric; returns true if a seesaw regression is detected.
    pub fn observe(&mut self, metric: &AbMetric) -> bool {
        let regressed = metric.is_regression();
        if regressed {
            self.regressions += 1;
        }
        self.last_metric = metric.clone();
        regressed
    }

    /// After >=2 regressions we treat it as catastrophic forgetting.
    pub fn catastrophic(&self) -> bool {
        self.regressions >= 2
    }

    /// Clear the regression counter after a catastrophic rollback has fired.
    /// The rollback IS the corrective action — without this reset the counter
    /// is monotonic and every later cycle for the slug rolls back forever
    /// (evolution bricks permanently). Mirrors StagnationState, which already
    /// zeroes its counter when its rollback fires.
    pub fn reset_after_rollback(&mut self) {
        self.regressions = 0;
    }
}

// ── Stagnation ────────────────────────────────────────────────────────────

/// Stagnation: if N consecutive sessions show no improvement ≥ threshold,
/// auto-rollback to the last known-good config.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StagnationState {
    pub stagnation_limit: u32,
    pub improvement_threshold: f64,
    pub sessions_without_improvement: u32,
}

impl StagnationState {
    pub fn new(stagnation_limit: u32, improvement_threshold: f64) -> Self {
        Self {
            stagnation_limit,
            improvement_threshold,
            sessions_without_improvement: 0,
        }
    }

    /// Observe a session's A/B metric. Returns Rollback if the limit is hit.
    pub fn observe(&mut self, metric: &AbMetric) -> StagnationAction {
        if metric.is_improvement(self.improvement_threshold) {
            self.sessions_without_improvement = 0;
            StagnationAction::Continue
        } else {
            self.sessions_without_improvement += 1;
            if self.sessions_without_improvement >= self.stagnation_limit {
                self.sessions_without_improvement = 0;
                StagnationAction::Rollback
            } else {
                StagnationAction::Continue
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StagnationAction {
    Continue,
    Rollback,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metric(with: f64, without: f64, samples_with: u32) -> AbMetric {
        AbMetric {
            avg_score_with: with,
            avg_score_without: without,
            samples_with,
            samples_without: 0,
        }
    }

    #[test]
    fn safety_gate_set_requires_all_three() {
        SafetyGateSet::all().validate_all().unwrap();
        let missing = SafetyGateSet {
            critic: true,
            seesaw: true,
            stagnation: false,
        };
        assert!(missing.validate_all().is_err());
    }

    #[test]
    fn critic_rejects_skill_removal() {
        let m = metric(0.5, 0.5, 1);
        let v = critic_review(&EditType::RemoveSkill, &m, 1.0);
        assert!(!v.approved);
        assert!(v.reward_hacking);
    }

    #[test]
    fn critic_rejects_implausible_metric_jump() {
        let m = metric(0.999, 0.3, 10);
        let v = critic_review(&EditType::ModifyConfig, &m, 1.0);
        assert!(!v.approved);
    }

    #[test]
    fn critic_high_weight_blocks_instinct_edit() {
        let m = metric(0.6, 0.6, 1);
        let v = critic_review(&EditType::ModifyInstinct, &m, 1.5);
        assert!(!v.approved);
    }

    #[test]
    fn seesaw_detects_regression() {
        let mut s = SeesawState::new(metric(0.8, 0.5, 1));
        assert!(!s.observe(&metric(0.9, 0.5, 1))); // improved
        assert!(s.observe(&metric(0.4, 0.5, 1))); // regressed
        assert!(s.observe(&metric(0.3, 0.5, 1))); // regressed again
        assert!(s.catastrophic());
    }

    #[test]
    fn stagnation_rolls_back_after_limit() {
        let mut st = StagnationState::new(3, 0.02);
        assert_eq!(st.observe(&metric(0.5, 0.5, 1)), StagnationAction::Continue);
        assert_eq!(
            st.observe(&metric(0.51, 0.5, 1)),
            StagnationAction::Continue
        );
        assert_eq!(
            st.observe(&metric(0.51, 0.5, 1)),
            StagnationAction::Rollback
        );
    }

    #[test]
    fn stagnation_resets_on_improvement() {
        let mut st = StagnationState::new(3, 0.02);
        st.observe(&metric(0.5, 0.5, 1));
        st.observe(&metric(0.5, 0.5, 1));
        // big improvement resets
        assert_eq!(st.observe(&metric(0.9, 0.5, 1)), StagnationAction::Continue);
        assert_eq!(st.sessions_without_improvement, 0);
    }
}

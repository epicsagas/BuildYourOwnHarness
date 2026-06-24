//! Evolution observation records (B10 Observe phase, ARCH §7.1).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One tool-call observation. The evolution engine accumulates these and
/// analyzes at SessionEnd.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationRecord {
    pub id: String,
    pub observed_at: DateTime<Utc>,
    pub tool_name: String,
    pub outcome: ObservedOutcome,
    /// A/B slot: was the evolving skill active for this call?
    pub with_evolved: bool,
    /// 0.0..=1.0 subjective outcome score (critic computes).
    pub score: f64,
    /// Dominant error type, if any (SkillOpt pattern mining).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dominant_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObservedOutcome {
    Success,
    PartialSuccess,
    Failure,
    Skipped,
}

impl ObservedOutcome {
    pub fn to_score(self) -> f64 {
        use ObservedOutcome::*;
        match self {
            Success => 1.0,
            PartialSuccess => 0.5,
            Failure => 0.0,
            Skipped => 0.0,
        }
    }
}

/// A/B metric slots (ARCH §5.4 B10 metrics seed).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AbMetric {
    pub avg_score_with: f64,
    pub avg_score_without: f64,
    pub samples_with: u32,
    pub samples_without: u32,
}

impl AbMetric {
    /// Is the evolved variant an improvement ≥ threshold? (Stagnation gate.)
    pub fn is_improvement(&self, threshold: f64) -> bool {
        if self.samples_with == 0 {
            return false;
        }
        (self.avg_score_with - self.avg_score_without) >= threshold
    }

    /// Has performance degraded vs without? (Rollback trigger.)
    pub fn is_regression(&self) -> bool {
        if self.samples_with == 0 {
            return false;
        }
        self.avg_score_with < self.avg_score_without
    }
}

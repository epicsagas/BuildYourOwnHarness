//! Observation / state facade. The bulk of file-based state lives in
//! `deploy::state` (BuildStore) and `domain::state` (BuildState). This module
//! re-exports the recovery protocol entry points used by the CLI/orchestrator
//! and provides the observation accumulator for the evolution engine.

pub use crate::deploy::state::{crash_check, BuildStore, CrashReport};
pub use crate::domain::state::{lifecycle, BuildPhase, BuildState, PhaseEntry, Transition};

use crate::domain::evidence::ObservationRecord;

/// Accumulates observation records across a session for the evolution engine.
#[derive(Debug, Default, Clone)]
pub struct ObservationLog {
    pub records: Vec<ObservationRecord>,
}

impl ObservationLog {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record(&mut self, r: ObservationRecord) {
        self.records.push(r);
    }
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_accumulates() {
        let mut log = ObservationLog::new();
        assert!(log.is_empty());
        log.record(ObservationRecord {
            id: "1".into(),
            observed_at: chrono::Utc::now(),
            tool_name: "x".into(),
            outcome: crate::domain::evidence::ObservedOutcome::Success,
            with_evolved: true,
            score: 1.0,
            dominant_error: None,
        });
        assert_eq!(log.len(), 1);
    }
}

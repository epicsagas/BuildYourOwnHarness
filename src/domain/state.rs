//! File-based state + crash recovery (B9, ARCH §7.3).

use serde::{Deserialize, Serialize};

use super::profile::ProfileStatus;

/// A persisted build/phase checkpoint (ARCH §7.3 `BUILD-<timestamp>.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildState {
    pub slug: String,
    pub phase: BuildPhase,
    pub profile_status: ProfileStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub phase_history: Vec<PhaseEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildPhase {
    Scan,
    Interview,
    Wizard,
    Compile,
    DryRun,
    Install,
    Done,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseEntry {
    pub phase: BuildPhase,
    pub status: String,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// The full lifecycle in order.
pub fn lifecycle() -> &'static [BuildPhase] {
    use BuildPhase::*;
    &[Scan, Interview, Wizard, Compile, DryRun, Install, Done]
}

/// A legal transition descriptor (used by obs recovery).
#[derive(Debug, Clone, Copy)]
pub struct Transition {
    pub from: BuildPhase,
    pub to: BuildPhase,
}

impl BuildState {
    /// ARCH §7.3 crash-recovery threshold: updated_at older than 45 minutes
    /// ⇒ assume crash, resume from last safe phase. (Mirrors epic-harness orbit.)
    pub const CRASH_THRESHOLD_SECS: i64 = 45 * 60;

    pub fn is_stale(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        (now - self.updated_at).num_seconds() > Self::CRASH_THRESHOLD_SECS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_order() {
        let l = lifecycle();
        assert_eq!(l.first(), Some(&BuildPhase::Scan));
        assert_eq!(l.last(), Some(&BuildPhase::Done));
        assert_eq!(l.len(), 7);
    }

    #[test]
    fn crash_threshold_is_45_min() {
        assert_eq!(BuildState::CRASH_THRESHOLD_SECS, 2700);
    }
}

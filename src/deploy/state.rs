//! File-based build state + crash recovery (B9, ARCH §7.3).
//!
//! Persists `BUILD-<timestamp>.json`; the 45-minute staleness threshold detects
//! crashes so the orchestrator can resume from the last safe phase.

use std::path::{Path, PathBuf};

use crate::domain::state::{BuildPhase, BuildState, PhaseEntry};

/// On-disk store for one slug's build state.
#[derive(Debug, Clone)]
pub struct BuildStore {
    dir: PathBuf,
    slug: String,
}

impl BuildStore {
    pub fn new(root: &Path, slug: &str) -> Self {
        Self {
            dir: root.join("builds").join(slug),
            slug: slug.to_string(),
        }
    }

    fn ensure_dir(&self) -> crate::domain::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        Ok(())
    }

    /// Write the current build state, returning the path.
    pub fn checkpoint(&self, state: &BuildState) -> crate::domain::Result<PathBuf> {
        self.ensure_dir()?;
        let path = self
            .dir
            .join(format!("BUILD-{}.json", ts_suffix(&state.updated_at)));
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(&path, json)?;
        Ok(path)
    }

    /// Load the latest build state for this slug, if any.
    pub fn load_latest(&self) -> crate::domain::Result<Option<BuildState>> {
        if !self.dir.exists() {
            return Ok(None);
        }
        let mut latest: Option<(String, BuildState)> = None;
        for entry in std::fs::read_dir(&self.dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let body = std::fs::read_to_string(&path)?;
            let state: BuildState = match serde_json::from_str(&body) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            match &latest {
                Some((best, _)) if best >= &name => {}
                _ => latest = Some((name, state)),
            }
        }
        Ok(latest.map(|(_, s)| s))
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }
}

fn ts_suffix(t: &chrono::DateTime<chrono::Utc>) -> String {
    t.format("%Y%m%dT%H%M%SZ").to_string()
}

/// Determine whether a build state is stale (crashed). `now` is supplied so the
/// function is deterministic in tests (AC17).
pub fn crash_check(state: &BuildState, now: chrono::DateTime<chrono::Utc>) -> CrashReport {
    let stale = state.is_stale(now);
    CrashReport {
        stale,
        last_safe_phase: last_completed_phase(state).unwrap_or(state.phase),
        seconds_since_update: (now - state.updated_at).num_seconds().max(0),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CrashReport {
    pub stale: bool,
    pub last_safe_phase: BuildPhase,
    pub seconds_since_update: i64,
}

/// The last phase with a completed entry in phase_history — the resume point.
fn last_completed_phase(state: &BuildState) -> Option<BuildPhase> {
    state
        .phase_history
        .iter()
        .rev()
        .find(|e: &&PhaseEntry| e.status == "complete")
        .map(|e| e.phase)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::ProfileStatus;
    use chrono::TimeZone;
    use tempfile::tempdir;

    fn state_at(phase: BuildPhase, updated: chrono::DateTime<chrono::Utc>) -> BuildState {
        BuildState {
            slug: "dev1".into(),
            phase,
            profile_status: ProfileStatus::Confirmed,
            started_at: updated,
            updated_at: updated,
            phase_history: vec![],
        }
    }

    #[test]
    fn checkpoint_roundtrip() {
        let dir = tempdir().unwrap();
        let store = BuildStore::new(dir.path(), "dev1");
        let now = chrono::Utc::now();
        let st = state_at(BuildPhase::Compile, now);
        let path = store.checkpoint(&st).unwrap();
        assert!(path.exists());

        let loaded = store.load_latest().unwrap().unwrap();
        assert_eq!(loaded.phase, BuildPhase::Compile);
        assert_eq!(loaded.slug, "dev1");
    }

    #[test]
    fn crash_detected_at_46_minutes() {
        // AC17: updated_at 46 min in the past ⇒ stale (crash).
        let now = chrono::Utc.with_ymd_and_hms(2026, 6, 24, 12, 0, 0).unwrap();
        let updated = now - chrono::Duration::seconds(46 * 60);
        let st = state_at(BuildPhase::DryRun, updated);
        let report = crash_check(&st, now);
        assert!(report.stale);
    }

    #[test]
    fn no_crash_within_threshold() {
        let now = chrono::Utc.with_ymd_and_hms(2026, 6, 24, 12, 0, 0).unwrap();
        let updated = now - chrono::Duration::seconds(10 * 60);
        let st = state_at(BuildPhase::DryRun, updated);
        let report = crash_check(&st, now);
        assert!(!report.stale);
    }
}

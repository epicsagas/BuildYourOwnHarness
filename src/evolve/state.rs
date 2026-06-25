//! Evolution state persistence (Ring 3).
//!
//! `byoh evolve <slug>` runs one cycle; the seesaw + stagnation gate state must
//! carry across runs (otherwise the gates are blind — they only detect
//! regression/stagnation *over time*). This stores that state under
//! `BYOH_HOME/evolve/<slug>/state.json`.
//!
//! Guardrails (per the council Critic):
//! - **Versioned schema** (`version`) so old checkpoints load forward.
//! - **No silent reset**: a malformed/unreadable state file is *backed up* to
//!   `state.json.corrupt-<n>` and an error is returned — we never silently wipe
//!   seesaw/stagnation history, which would blind the safety gates.
//! - **Explicit baseline**: first run (`load` returns `None`) ⇒ caller seeds a
//!   fresh baseline, not "empty == approved".

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::domain::error::ByohError;
use crate::evolve::gates::{SeesawState, StagnationState};
use crate::Result;

/// Current on-disk schema version.
pub const STATE_VERSION: u32 = 1;

/// Persisted evolution state for one slug.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvolveState {
    #[serde(default = "default_version")]
    pub version: u32,
    pub seesaw: SeesawState,
    pub stagnation: StagnationState,
    /// Number of cycles run so far (monotonic).
    #[serde(default)]
    pub cycle_n: u64,
}

fn default_version() -> u32 {
    STATE_VERSION
}

impl EvolveState {
    /// A fresh baseline: no regressions, no stagnation, cycle 0.
    pub fn baseline(stagnation_limit: u32, improvement_threshold: f64) -> Self {
        Self {
            version: STATE_VERSION,
            seesaw: SeesawState::new(Default::default()),
            stagnation: StagnationState::new(stagnation_limit, improvement_threshold),
            cycle_n: 0,
        }
    }
}

/// On-disk store for one slug's evolution state.
#[derive(Debug, Clone)]
pub struct EvolveStore {
    dir: PathBuf,
}

impl EvolveStore {
    /// Store rooted at `<root>/evolve/<slug>/`. The caller is responsible for
    /// sanitizing `slug` (see `store::sanitize_slug`).
    pub fn new(root: &Path, slug: &str) -> Self {
        Self {
            dir: root.join("evolve").join(slug),
        }
    }

    fn state_path(&self) -> PathBuf {
        self.dir.join("state.json")
    }

    /// Load the persisted state, or `None` on first run. A malformed file is
    /// backed up and an error is returned (never silently reset).
    pub fn load(&self) -> Result<Option<EvolveState>> {
        let path = self.state_path();
        if !path.exists() {
            return Ok(None);
        }
        let body = std::fs::read_to_string(&path)
            .map_err(|e| ByohError::Other(format!("{}: {e}", path.display())))?;
        match serde_json::from_str::<EvolveState>(&body) {
            Ok(state) => Ok(Some(state)),
            Err(e) => {
                let backup = self.backup_corrupt(&path)?;
                Err(ByohError::Other(format!(
                    "evolve state at {} is malformed ({e}); backed up to {} — \
                     refusing to silently reset (would blind the safety gates). \
                     Inspect or delete it to start a fresh baseline.",
                    path.display(),
                    backup.display()
                )))
            }
        }
    }

    /// Persist the state atomically (temp file + rename).
    pub fn save(&self, state: &EvolveState) -> Result<PathBuf> {
        std::fs::create_dir_all(&self.dir)
            .map_err(|e| ByohError::Other(format!("{}: {e}", self.dir.display())))?;
        let path = self.state_path();
        let tmp = self.dir.join("state.json.tmp");
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(&tmp, json)
            .map_err(|e| ByohError::Other(format!("{}: {e}", tmp.display())))?;
        std::fs::rename(&tmp, &path)
            .map_err(|e| ByohError::Other(format!("{}: {e}", path.display())))?;
        Ok(path)
    }

    /// Move a corrupt state file aside to `state.json.corrupt-<n>`.
    fn backup_corrupt(&self, path: &Path) -> Result<PathBuf> {
        let mut n = 0;
        loop {
            let candidate = self.dir.join(format!("state.json.corrupt-{n}"));
            if !candidate.exists() {
                std::fs::rename(path, &candidate)
                    .map_err(|e| ByohError::Other(format!("{}: {e}", candidate.display())))?;
                return Ok(candidate);
            }
            n += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_run_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let store = EvolveStore::new(dir.path(), "dev");
        assert!(store.load().unwrap().is_none());
    }

    #[test]
    fn round_trip_save_load() {
        let dir = tempfile::tempdir().unwrap();
        let store = EvolveStore::new(dir.path(), "dev");
        let mut state = EvolveState::baseline(3, 0.02);
        state.cycle_n = 5;
        state.seesaw.regressions = 1;
        store.save(&state).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded, state);
        assert_eq!(loaded.version, STATE_VERSION);
    }

    #[test]
    fn malformed_state_is_backed_up_not_reset() {
        let dir = tempfile::tempdir().unwrap();
        let store = EvolveStore::new(dir.path(), "dev");
        std::fs::create_dir_all(&store.dir).unwrap();
        std::fs::write(store.state_path(), "{ not valid json").unwrap();
        let err = store.load().unwrap_err();
        assert!(matches!(err, ByohError::Other(_)));
        // original moved aside, not deleted
        assert!(store.dir.join("state.json.corrupt-0").exists());
        assert!(!store.state_path().exists());
    }

    #[test]
    fn baseline_is_clean() {
        let b = EvolveState::baseline(3, 0.02);
        assert_eq!(b.cycle_n, 0);
        assert_eq!(b.seesaw.regressions, 0);
        assert_eq!(b.stagnation.sessions_without_improvement, 0);
    }
}

//! Plugin installation — deploy a rendered harness to a host's plugin location.
//!
//! The most dangerous operation in BYOH: it writes into directories the user
//! cares about (`~/.claude/plugins/`, `~/.gemini/antigravity-cli/plugins/`).
//! Guardrails (per the council Critic):
//!
//! - **HOME is opt-in**: the default destination is a project-local `dist/`.
//!   Touching a host plugin dir requires an explicit target location.
//! - **BYOH-owned only**: every install drops a `.byoh-manifest` marker. We
//!   refuse to overwrite a directory that exists and lacks the marker unless
//!   `force` is set — so a user's hand-edited / third-party plugin is never
//!   silently clobbered.
//! - **Atomic**: render into a temp dir *next to* the destination (same parent
//!   ⇒ same filesystem ⇒ atomic `rename`), then swap. A crash mid-render never
//!   leaves a half-installed plugin in place. The previous install is kept as a
//!   `.bak` until the swap succeeds, then removed.
//! - **Slug-sanitized**: the plugin name is validated before any path join.

use std::path::{Path, PathBuf};

use crate::application::render_target;
use crate::domain::bundle::HarnessBundle;
use crate::domain::error::ByohError;
use crate::domain::render_target::Target;
use crate::store::sanitize_slug;
use crate::Result;

/// The on-disk marker proving a plugin directory was created by BYOH and is
/// therefore safe to overwrite on re-install.
pub const OWNED_MARKER: &str = ".byoh-manifest";

/// Resolved per-host plugin install roots. `from_env` honors overrides (used by
/// tests to redirect into a tempdir) and falls back to the host conventions.
#[derive(Debug, Clone)]
pub struct InstallLocations {
    /// Project-local default destination root (no HOME writes).
    pub dist: PathBuf,
    /// Claude Code plugins root (`~/.claude/plugins`).
    pub claude: PathBuf,
    /// Antigravity (agy) plugins root (`~/.gemini/antigravity-cli/plugins`).
    pub agy: PathBuf,
    /// Codex plugins root (`~/.codex/plugins`).
    pub codex: PathBuf,
}

impl InstallLocations {
    /// Resolve install roots. Env overrides (for tests / power users):
    /// `BYOH_DIST_DIR`, `CLAUDE_PLUGIN_DIR`, `AGY_PLUGIN_DIR`, `CODEX_PLUGIN_DIR`.
    pub fn from_env() -> Self {
        let home = home_dir();
        Self {
            dist: env_or("BYOH_DIST_DIR", PathBuf::from("dist")),
            claude: env_or("CLAUDE_PLUGIN_DIR", home.join(".claude").join("plugins")),
            agy: env_or(
                "AGY_PLUGIN_DIR",
                home.join(".gemini").join("antigravity-cli").join("plugins"),
            ),
            codex: env_or("CODEX_PLUGIN_DIR", home.join(".codex").join("plugins")),
        }
    }

    /// The install root for a given host destination.
    fn root_for(&self, dest: InstallDest, target: Target) -> PathBuf {
        match dest {
            InstallDest::Dist => self.dist.clone(),
            InstallDest::Host => match target {
                Target::Claude => self.claude.clone(),
                Target::Agy => self.agy.clone(),
                Target::Codex => self.codex.clone(),
                // `all` installed to a host falls back to the dist root (one
                // tree per concrete target is rendered under it).
                Target::All => self.dist.clone(),
            },
        }
    }
}

/// Where to install: the safe project-local `dist/` (default) or the host's
/// real plugin directory (explicit opt-in, e.g. `--host`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallDest {
    Dist,
    Host,
}

/// Install a compiled/synthesized bundle as a plugin.
///
/// Renders `bundle` for `target` into a temp dir adjacent to the destination,
/// then atomically swaps it into `<root>/byoh-<slug>/`. Returns the final path.
pub fn install_plugin(
    bundle: &HarnessBundle,
    target: Target,
    dest: InstallDest,
    loc: &InstallLocations,
    force: bool,
) -> Result<PathBuf> {
    let slug = sanitize_slug(&bundle.slug)?;
    let name = format!("byoh-{slug}");
    let root = loc.root_for(dest, target);
    let final_dir = root.join(&name);

    // Guardrail: refuse to clobber a non-BYOH directory unless forced.
    if final_dir.exists() && !is_byoh_owned(&final_dir) && !force {
        return Err(ByohError::Other(format!(
            "refusing to overwrite non-BYOH directory {} (use --force to override)",
            final_dir.display()
        )));
    }

    std::fs::create_dir_all(&root).map_err(|e| io_at(&root, e))?;

    // Render into a staging dir on the SAME parent (⇒ atomic rename).
    let staging = root.join(format!(".{name}.staging"));
    let _ = std::fs::remove_dir_all(&staging); // clean any prior aborted staging
    render_target(bundle, target, &staging)?;
    // Drop the owned-marker into the staged tree.
    write_marker(&staging, &name)?;

    // Swap: move current → .bak, staging → final, then drop .bak.
    let backup = root.join(format!(".{name}.bak"));
    let _ = std::fs::remove_dir_all(&backup);
    if final_dir.exists() {
        std::fs::rename(&final_dir, &backup).map_err(|e| io_at(&final_dir, e))?;
    }
    match std::fs::rename(&staging, &final_dir) {
        Ok(()) => {
            let _ = std::fs::remove_dir_all(&backup);
            Ok(final_dir)
        }
        Err(e) => {
            // Roll back: restore the backup, drop the staging.
            let _ = std::fs::remove_dir_all(&staging);
            if backup.exists() {
                let _ = std::fs::rename(&backup, &final_dir);
            }
            Err(io_at(&final_dir, e))
        }
    }
}

fn write_marker(dir: &Path, name: &str) -> Result<()> {
    let body = format!("{{\"tool\":\"byoh\",\"plugin\":\"{name}\",\"owned\":true}}\n");
    std::fs::write(dir.join(OWNED_MARKER), body).map_err(|e| io_at(dir, e))?;
    Ok(())
}

/// A directory is BYOH-owned iff it carries the marker file.
pub fn is_byoh_owned(dir: &Path) -> bool {
    dir.join(OWNED_MARKER).exists()
}

fn env_or(var: &str, fallback: PathBuf) -> PathBuf {
    std::env::var(var).map(PathBuf::from).unwrap_or(fallback)
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn io_at(path: &Path, e: std::io::Error) -> ByohError {
    ByohError::Other(format!("{}: {}", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::compile_profile;
    use crate::domain::genre::Genre;
    use crate::domain::profile::{GenreConfidence, ProfileStatus, UserProfile};

    fn confirmed() -> UserProfile {
        let mut p = UserProfile::new_draft("dev", "en");
        p.candidates.identity.genre = Some(GenreConfidence {
            value: Genre::Developer,
            confidence: 1.0,
            provenance: vec![],
        });
        p.status = ProfileStatus::Confirmed;
        p
    }

    fn locs(dist: &Path) -> InstallLocations {
        InstallLocations {
            dist: dist.to_path_buf(),
            claude: dist.join("claude"),
            agy: dist.join("agy"),
            codex: dist.join("codex"),
        }
    }

    #[test]
    fn install_to_dist_atomic_with_marker() {
        let bundle = compile_profile(&confirmed()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let loc = locs(dir.path());
        let out = install_plugin(&bundle, Target::Claude, InstallDest::Dist, &loc, false).unwrap();
        assert!(out.ends_with("byoh-dev"));
        assert!(out.join(OWNED_MARKER).exists(), "marker must be present");
        assert!(out.join(".claude-plugin/plugin.json").exists());
        // no staging/backup left behind
        assert!(!dir.path().join(".byoh-dev.staging").exists());
        assert!(!dir.path().join(".byoh-dev.bak").exists());
    }

    #[test]
    fn reinstall_is_idempotent_over_byoh_owned() {
        let bundle = compile_profile(&confirmed()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let loc = locs(dir.path());
        let out1 = install_plugin(&bundle, Target::Claude, InstallDest::Dist, &loc, false).unwrap();
        // second install over the byoh-owned dir succeeds without --force
        let out2 = install_plugin(&bundle, Target::Claude, InstallDest::Dist, &loc, false).unwrap();
        assert_eq!(out1, out2);
        assert!(out2.join(OWNED_MARKER).exists());
    }

    #[test]
    fn refuses_to_overwrite_non_byoh_dir() {
        let bundle = compile_profile(&confirmed()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let loc = locs(dir.path());
        // pre-create a non-BYOH dir at the target name
        let target_dir = dir.path().join("byoh-dev");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(target_dir.join("user-file.txt"), "precious").unwrap();

        let err =
            install_plugin(&bundle, Target::Claude, InstallDest::Dist, &loc, false).unwrap_err();
        assert!(matches!(err, ByohError::Other(_)));
        // user's file untouched
        assert!(target_dir.join("user-file.txt").exists());

        // with force, it installs (overwrites)
        install_plugin(&bundle, Target::Claude, InstallDest::Dist, &loc, true).unwrap();
        assert!(target_dir.join(OWNED_MARKER).exists());
    }

    #[test]
    fn rejects_bad_slug() {
        let mut p = confirmed();
        p.slug = "../evil".into();
        let bundle = compile_profile(&p).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let loc = locs(dir.path());
        assert!(install_plugin(&bundle, Target::Claude, InstallDest::Dist, &loc, false).is_err());
    }
}

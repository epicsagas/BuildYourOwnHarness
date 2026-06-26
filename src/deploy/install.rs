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
use crate::ports::command::{CommandOutcome, CommandPort};
use crate::store::{create_symlink_or_copy, sanitize_slug, write_file};
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
    /// Antigravity (agy) plugins root (`~/.gemini/config/plugins`).
    pub agy: PathBuf,
    /// Codex plugins root (`~/.codex/plugins`).
    pub codex: PathBuf,
    /// Claude Code config root (`~/.claude`), honored from `CLAUDE_CONFIG_DIR`.
    /// Plugins are activated by linking under `<this>/skills/<name>/`.
    pub claude_config: PathBuf,
}

impl InstallLocations {
    /// Resolve install roots. Env overrides (for tests / power users):
    /// `BYOH_DIST_DIR`, `CLAUDE_PLUGIN_DIR`, `AGY_PLUGIN_DIR`, `CODEX_PLUGIN_DIR`,
    /// `CLAUDE_CONFIG_DIR` (Claude config root, used for activation links).
    pub fn from_env() -> Self {
        let home = home_dir();
        Self {
            dist: env_or("BYOH_DIST_DIR", PathBuf::from("dist")),
            claude: env_or("CLAUDE_PLUGIN_DIR", home.join(".claude").join("plugins")),
            agy: env_or(
                "AGY_PLUGIN_DIR",
                home.join(".gemini").join("config").join("plugins"),
            ),
            codex: env_or("CODEX_PLUGIN_DIR", home.join(".codex").join("plugins")),
            claude_config: env_or("CLAUDE_CONFIG_DIR", home.join(".claude")),
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

// ─── Activation ──────────────────────────────────────────────────────────────
//
// `install_plugin` only drops files; it does NOT make a host discover them.
// Each host has a different discovery mechanism (verified on a live install):
//
// - Claude Code: a plugin dir under `<config>/skills/<name>/` auto-loads as
//   `<name>@skills-dir` (official — see `claude plugin init --help`). Pure file
//   op: a symlink there → the installed plugin dir. No settings.json, no CLI.
// - Codex: plugins are marketplace-sourced. Register the plugin dir as a
//   one-plugin local marketplace (`codex plugin marketplace add <dir>`), then
//   `codex plugin add <name>@<mp>`. Codex owns the `~/.codex/config.toml` edit.
// - agy (Antigravity): `agy plugin install <dir>` populates
//   `~/.gemini/config/plugins` + `import_manifest.json`.

/// Outcome of activating an installed plugin so a host tool discovers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivationReport {
    /// Which host this report is for.
    pub host: Target,
    /// What happened.
    pub status: ActivationStatus,
    /// Human-readable detail (shown to the user by the caller).
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationStatus {
    /// The host now discovers the plugin (link registered / plugin added).
    Activated,
    /// The host CLI is missing — the exact manual commands are in `message`.
    ManualStepsRequired,
    /// Activation was attempted but failed; the install itself still succeeded.
    Failed,
}

/// Activate an already-installed plugin so the host tool discovers it.
///
/// Non-fatal: an install succeeds regardless of the report; this only describes
/// the activation attempt. `Target::All` is rejected — callers must expand `All`
/// into concrete targets first (one install dir per host).
pub fn activate_plugin<C: CommandPort>(
    target: Target,
    plugin_dir: &Path,
    slug: &str,
    loc: &InstallLocations,
    commands: &C,
) -> Result<ActivationReport> {
    match target {
        Target::Claude => activate_claude(plugin_dir, slug, loc),
        Target::Codex => activate_codex(plugin_dir, slug, commands),
        Target::Agy => activate_agy(plugin_dir, slug, loc, commands),
        Target::All => Err(ByohError::Other(
            "activate_plugin does not support Target::All — expand it first".into(),
        )),
    }
}

/// Claude Code: link `<config>/skills/byoh-<slug>` → installed plugin dir. The
/// directory under `skills/` is the official auto-load mechanism
/// (`<name>@skills-dir`); no settings.json or host CLI needed.
fn activate_claude(
    plugin_dir: &Path,
    slug: &str,
    loc: &InstallLocations,
) -> Result<ActivationReport> {
    let name = format!("byoh-{slug}");
    let link = loc.claude_config.join("skills").join(&name);
    create_symlink_or_copy(plugin_dir, &link)?;
    Ok(ActivationReport {
        host: Target::Claude,
        status: ActivationStatus::Activated,
        message: format!(
            "linked @skills-dir at {}; restart Claude Code to load",
            link.display()
        ),
    })
}

/// Codex: register the plugin dir as a one-plugin local marketplace, then add
/// the plugin. Codex owns the `~/.codex/config.toml` mutation.
fn activate_codex<C: CommandPort>(
    plugin_dir: &Path,
    slug: &str,
    commands: &C,
) -> Result<ActivationReport> {
    let name = format!("byoh-{slug}");
    write_codex_marketplace(plugin_dir, &name)?;
    let dir = plugin_dir.display().to_string();
    let selector = format!("{name}@{name}");
    let manual = format!(
        "codex CLI not found. Run:\n  codex plugin marketplace add {dir}\n  codex plugin add {selector}"
    );
    if !commands.is_installed("codex") {
        return Ok(ActivationReport {
            host: Target::Codex,
            status: ActivationStatus::ManualStepsRequired,
            message: manual,
        });
    }
    // `marketplace add` is idempotent; an "already exists" failure is fine.
    match commands.run("codex", &["plugin", "marketplace", "add", &dir], None) {
        CommandOutcome::Ran { .. } | CommandOutcome::Failed { .. } => {}
        CommandOutcome::NotInstalled => {
            return Ok(ActivationReport {
                host: Target::Codex,
                status: ActivationStatus::ManualStepsRequired,
                message: manual,
            });
        }
    }
    match commands.run("codex", &["plugin", "add", &selector], None) {
        CommandOutcome::Ran { .. } => Ok(ActivationReport {
            host: Target::Codex,
            status: ActivationStatus::Activated,
            message: "marketplace registered + plugin added; restart Codex to load".into(),
        }),
        CommandOutcome::Failed { stderr, .. } if stderr.to_lowercase().contains("already") => {
            Ok(ActivationReport {
                host: Target::Codex,
                status: ActivationStatus::Activated,
                message: "plugin already added; restart Codex to load".into(),
            })
        }
        CommandOutcome::Failed { stderr, .. } => Ok(ActivationReport {
            host: Target::Codex,
            status: ActivationStatus::Failed,
            message: format!("codex plugin add failed: {stderr}"),
        }),
        CommandOutcome::NotInstalled => Ok(ActivationReport {
            host: Target::Codex,
            status: ActivationStatus::ManualStepsRequired,
            message: manual,
        }),
    }
}

/// Write a one-plugin local `marketplace.json` so Codex treats the installed
/// dir as its own marketplace via `codex plugin marketplace add <dir>`.
///
/// Codex discovers the manifest at `.agents/plugins/marketplace.json` (verified
/// against the live `codex plugin marketplace add`); the plugin source is the
/// dir itself (`source: local`, `path: "./"`), matching the `openai-curated`
/// local-plugin layout.
fn write_codex_marketplace(plugin_dir: &Path, name: &str) -> Result<()> {
    let body = serde_json::json!({
        "name": name,
        "plugins": [{
            "name": name,
            "source": { "source": "local", "path": "./" },
            "description": "Personalized BYOH harness."
        }]
    });
    write_file(
        &plugin_dir.join(".agents").join("plugins"),
        "marketplace.json",
        &body.to_string(),
    )
}

/// agy: install the plugin dir into `~/.gemini/config/plugins` via the agy CLI,
/// which records it in `import_manifest.json` and handles the claude/codex-shaped
/// markers we already render.
fn activate_agy<C: CommandPort>(
    plugin_dir: &Path,
    slug: &str,
    loc: &InstallLocations,
    commands: &C,
) -> Result<ActivationReport> {
    let name = format!("byoh-{slug}");
    let dir = plugin_dir.display().to_string();
    let manual =
        format!("agy CLI not found. Run:\n  agy plugin install {dir}\n  agy plugin enable {name}");
    if !commands.is_installed("agy") {
        return Ok(ActivationReport {
            host: Target::Agy,
            status: ActivationStatus::ManualStepsRequired,
            message: manual,
        });
    }
    match commands.run("agy", &["plugin", "install", &dir], None) {
        CommandOutcome::Ran { .. } => Ok(ActivationReport {
            host: Target::Agy,
            status: ActivationStatus::Activated,
            message: format!("installed to {}; restart agy to load", loc.agy.display()),
        }),
        CommandOutcome::Failed { stderr, .. } if stderr.to_lowercase().contains("already") => {
            Ok(ActivationReport {
                host: Target::Agy,
                status: ActivationStatus::Activated,
                message: "plugin already installed; restart agy to load".into(),
            })
        }
        CommandOutcome::Failed { stderr, .. } => Ok(ActivationReport {
            host: Target::Agy,
            status: ActivationStatus::Failed,
            message: format!("agy plugin install failed: {stderr}"),
        }),
        CommandOutcome::NotInstalled => Ok(ActivationReport {
            host: Target::Agy,
            status: ActivationStatus::ManualStepsRequired,
            message: manual,
        }),
    }
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
            claude_config: dist.join("claude-config"),
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

    /// Stub command port reporting every tool as missing — deterministic for
    /// activation tests that exercise the "manual steps" path.
    struct MissingCli;
    impl CommandPort for MissingCli {
        fn run(&self, _tool: &str, _args: &[&str], _cwd: Option<&Path>) -> CommandOutcome {
            CommandOutcome::NotInstalled
        }
        fn is_installed(&self, _tool: &str) -> bool {
            false
        }
    }

    #[test]
    fn claude_activation_creates_skills_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let plugin = dir.path().join("byoh-dev");
        std::fs::create_dir_all(&plugin).unwrap();
        let loc = locs(dir.path());
        let r = activate_plugin(Target::Claude, &plugin, "dev", &loc, &MissingCli).unwrap();
        let link = dir
            .path()
            .join("claude-config")
            .join("skills")
            .join("byoh-dev");
        assert!(link.exists(), "skills-dir link must exist");
        assert_eq!(r.status, ActivationStatus::Activated);
    }

    #[test]
    fn claude_activation_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let plugin = dir.path().join("byoh-dev");
        std::fs::create_dir_all(&plugin).unwrap();
        let loc = locs(dir.path());
        activate_plugin(Target::Claude, &plugin, "dev", &loc, &MissingCli).unwrap();
        // second activation must not error and leaves a single link.
        let r2 = activate_plugin(Target::Claude, &plugin, "dev", &loc, &MissingCli).unwrap();
        assert_eq!(r2.status, ActivationStatus::Activated);
        let link = dir
            .path()
            .join("claude-config")
            .join("skills")
            .join("byoh-dev");
        assert!(link.exists());
    }

    #[test]
    fn claude_activation_driven_by_claude_config_field() {
        // The skills link lands under loc.claude_config, not a hardcoded ~/.claude,
        // proving activation is driven by InstallLocations (testable, no env).
        let dir = tempfile::tempdir().unwrap();
        let plugin = dir.path().join("byoh-dev");
        std::fs::create_dir_all(&plugin).unwrap();
        let loc = locs(dir.path());
        let _ = activate_plugin(Target::Claude, &plugin, "dev", &loc, &MissingCli).unwrap();
        let link = dir
            .path()
            .join("claude-config")
            .join("skills")
            .join("byoh-dev");
        assert!(link.exists(), "link must be under loc.claude_config");
    }

    #[test]
    fn codex_activation_writes_marketplace_json() {
        let dir = tempfile::tempdir().unwrap();
        let plugin = dir.path().join("byoh-dev");
        std::fs::create_dir_all(&plugin).unwrap();
        let loc = locs(dir.path());
        let r = activate_plugin(Target::Codex, &plugin, "dev", &loc, &MissingCli).unwrap();
        // Codex looks for the manifest at `.agents/plugins/marketplace.json`.
        let mp: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(plugin.join(".agents/plugins/marketplace.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(mp["name"], "byoh-dev");
        assert_eq!(mp["plugins"][0]["name"], "byoh-dev");
        assert_eq!(mp["plugins"][0]["source"]["source"], "local");
        assert_eq!(mp["plugins"][0]["source"]["path"], "./");
        assert_eq!(r.status, ActivationStatus::ManualStepsRequired);
    }

    #[test]
    fn codex_activation_manual_steps_message_has_commands() {
        let dir = tempfile::tempdir().unwrap();
        let plugin = dir.path().join("byoh-dev");
        std::fs::create_dir_all(&plugin).unwrap();
        let loc = locs(dir.path());
        let r = activate_plugin(Target::Codex, &plugin, "dev", &loc, &MissingCli).unwrap();
        assert_eq!(r.status, ActivationStatus::ManualStepsRequired);
        assert!(
            r.message.contains("codex plugin marketplace add"),
            "{:?}",
            r.message
        );
        assert!(r.message.contains("codex plugin add byoh-dev@byoh-dev"));
    }

    #[test]
    fn agy_default_root_is_config_plugins() {
        // Regression: the default agy root was ~/.gemini/antigravity-cli/plugins
        // (nonexistent on disk); it must be ~/.gemini/config/plugins.
        let loc = InstallLocations::from_env();
        assert!(
            loc.agy
                .ends_with(std::path::Path::new(".gemini/config/plugins")),
            "agy root must end with .gemini/config/plugins, got {}",
            loc.agy.display()
        );
    }

    #[test]
    fn agy_activation_manual_steps_when_cli_missing() {
        let dir = tempfile::tempdir().unwrap();
        let plugin = dir.path().join("byoh-dev");
        std::fs::create_dir_all(&plugin).unwrap();
        let loc = locs(dir.path());
        let r = activate_plugin(Target::Agy, &plugin, "dev", &loc, &MissingCli).unwrap();
        assert_eq!(r.status, ActivationStatus::ManualStepsRequired);
        assert!(r.message.contains("agy plugin install"));
        assert!(r.message.contains("agy plugin enable byoh-dev"));
    }

    #[test]
    fn activate_plugin_rejects_all_target() {
        let dir = tempfile::tempdir().unwrap();
        let plugin = dir.path().join("byoh-dev");
        std::fs::create_dir_all(&plugin).unwrap();
        let loc = locs(dir.path());
        assert!(activate_plugin(Target::All, &plugin, "dev", &loc, &MissingCli).is_err());
    }
}

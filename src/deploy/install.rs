//! Plugin installation — render a polyglot harness plugin and (optionally)
//! activate it so each host tool discovers it.
//!
//! `install_plugin` renders ONE polyglot plugin tree (Claude + Codex + agy
//! manifests + shared skills/agents) into the safe project-local `dist/`. It
//! never writes into a host's own plugin root — activation does that, and only
//! when the caller opts in (`--host`). Guardrails (per the council Critic):
//!
//! - **HOME is opt-in**: the default is a project-local `dist/`. Touching a
//!   host plugin dir requires explicit `--host` activation.
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

use crate::Result;
use crate::application::render_target;
use crate::domain::bundle::HarnessBundle;
use crate::domain::error::ByohError;
use crate::domain::render_target::Target;
use crate::domain::scope::Scope;
use crate::ports::command::{CommandOutcome, CommandPort};
use crate::store::{create_symlink_or_copy, sanitize_slug, write_file};

/// The on-disk marker proving a plugin directory was created by BYOH and is
/// therefore safe to overwrite on re-install.
pub const OWNED_MARKER: &str = ".byoh-manifest";

/// Resolved install roots used by install + activation. `from_env` honors
/// overrides (used by tests to redirect into a tempdir) and falls back to the
/// host conventions.
#[derive(Debug, Clone)]
pub struct InstallLocations {
    /// Project-local default destination root (no HOME writes). The polyglot
    /// plugin tree is rendered here at `<dist>/byoh-<slug>/`.
    pub dist: PathBuf,
    /// Antigravity (agy) plugins root (`~/.gemini/config/plugins`), shown in the
    /// activation report (agy copies the tree here on `agy plugin install`).
    /// NB: agy became a standalone CLI after deprecating Gemini, but still
    /// uses `~/.gemini/` as its shared config root (verified agy 1.0.13) — this
    /// is *not* stale. agy never supported a project-local plugin scope.
    pub agy: PathBuf,
    /// Claude Code config root (`~/.claude`), honored from `CLAUDE_CONFIG_DIR`.
    /// Plugins are activated by linking under `<this>/skills/<name>/`.
    pub claude_config: PathBuf,
}

impl InstallLocations {
    /// Resolve install roots. Env overrides (for tests / power users):
    /// `BYOH_DIST_DIR`, `AGY_PLUGIN_DIR`, `CLAUDE_CONFIG_DIR`.
    ///
    /// `dist` also honors a thread-local override (`set_dist_override`) so tests
    /// can redirect into a tempdir without `set_var("BYOH_DIST_DIR", …)`, which
    /// became `unsafe` in the Rust 2024 edition and is incompatible with this
    /// crate's `#![forbid(unsafe_code)]`.
    pub fn from_env() -> Self {
        let home = home_dir();
        let dist = DIST_OVERRIDE
            .with(|c| c.borrow().clone())
            .unwrap_or_else(|| env_or("BYOH_DIST_DIR", PathBuf::from("dist")));
        Self {
            dist,
            agy: env_or(
                "AGY_PLUGIN_DIR",
                home.join(".gemini").join("config").join("plugins"),
            ),
            claude_config: env_or("CLAUDE_CONFIG_DIR", home.join(".claude")),
        }
    }

    /// Same as [`from_env`](Self::from_env) but with `dist` supplied explicitly.
    /// Use this from `spawn_blocking` worker threads where the thread-local dist
    /// override is not visible (capture it on the originating thread first).
    pub fn from_env_with_dist(dist: PathBuf) -> Self {
        let home = home_dir();
        Self {
            dist,
            agy: env_or(
                "AGY_PLUGIN_DIR",
                home.join(".gemini").join("config").join("plugins"),
            ),
            claude_config: env_or("CLAUDE_CONFIG_DIR", home.join(".claude")),
        }
    }
}

thread_local! {
    static DIST_OVERRIDE: std::cell::RefCell<Option<PathBuf>> =
        const { std::cell::RefCell::new(None) };
}

/// Override the `dist` root for the current thread (tests). `None` clears it.
/// Use this instead of `set_var("BYOH_DIST_DIR", …)`: `unsafe`-free, thread-scoped.
pub fn set_dist_override(path: Option<PathBuf>) {
    DIST_OVERRIDE.with(|c| *c.borrow_mut() = path);
}

impl InstallLocations {
    /// Return a copy with `claude_config` swapped — used by `Scope::Local` to
    /// activate into the project's `./.claude/` instead of the user's HOME.
    /// The other roots are unchanged.
    pub fn with_claude_config(&self, claude_config: PathBuf) -> Self {
        let mut c = self.clone();
        c.claude_config = claude_config;
        c
    }
}

/// Resolve the user's `--scope`/`--host` choice into a single [`Scope`].
///
/// Precedence (back-compat preserving):
/// - `(None, false)` → `DistOnly` (the legacy no-activation default).
/// - `(None, true)`  → `Global` (the legacy `--host` behavior).
/// - `(Some(s), false)` → parse `s`.
/// - `(Some(s), true)`  → `Global` if `s` is `global`/`host`; otherwise an
///   **error** — `--host` and `--scope <other>` conflict and silently picking
///   one could activate into the wrong root.
pub fn resolve_scope(scope: Option<String>, host: bool) -> Result<Scope> {
    match (scope, host) {
        (None, false) => Ok(Scope::DistOnly),
        (None, true) => Ok(Scope::Global),
        (Some(raw), false) => raw.parse::<Scope>(),
        (Some(raw), true) => {
            let parsed: Scope = raw.parse::<Scope>()?;
            if parsed == Scope::Global {
                Ok(Scope::Global)
            } else {
                Err(ByohError::Other(format!(
                    "--host conflicts with --scope {}; --host means global activation, \
                     so use exactly one of --host or --scope <local|global|publish>",
                    parsed.as_str()
                )))
            }
        }
    }
}

/// Install a compiled/synthesized bundle as a **polyglot** plugin.
///
/// Renders one tree carrying all three hosts' manifests + shared skills/agents
/// into a temp dir adjacent to `<dist>/`, then atomically swaps it into
/// `<dist>/byoh-<slug>/`. Returns the final path. This never writes into a
/// host's own plugin root — that is activation's job (opt-in via `--host`).
pub fn install_plugin(
    bundle: &HarnessBundle,
    loc: &InstallLocations,
    force: bool,
) -> Result<PathBuf> {
    let slug = sanitize_slug(&bundle.slug)?;
    let name = format!("byoh-{slug}");
    let root = loc.dist.clone();
    let final_dir = root.join(&name);

    // Guardrail: refuse to clobber a non-BYOH directory unless forced.
    if final_dir.exists() && !is_byoh_owned(&final_dir) && !force {
        return Err(ByohError::Other(format!(
            "refusing to overwrite non-BYOH directory {} (use --force to override)",
            final_dir.display()
        )));
    }

    std::fs::create_dir_all(&root).map_err(|e| io_at(&root, e))?;

    // Render into a staging dir on the SAME parent (⇒ atomic rename). Resolve
    // the home dir here so doc overrides authored via `author_doc` are applied;
    // tests isolate this via `set_home_override`.
    let staging = root.join(format!(".{name}.staging"));
    let _ = std::fs::remove_dir_all(&staging); // clean any prior aborted staging
    let home = crate::store::byoh_home();
    render_target(bundle, Target::All, &staging, &home)?;
    // Drop the owned-marker into the staged tree.
    write_owned_marker(&staging, &name)?;

    // Swap: move current → .bak, staging → final, then drop .bak.
    let backup = root.join(format!(".{name}.bak"));
    let _ = std::fs::remove_dir_all(&backup);
    if final_dir.exists() {
        std::fs::rename(&final_dir, &backup).map_err(|e| io_at(&final_dir, e))?;
    }
    match std::fs::rename(&staging, &final_dir) {
        Ok(()) => {
            let _ = std::fs::remove_dir_all(&backup);
            // Return an absolute, symlink-safe path: the dist root may be a
            // relative dir (default "dist"), and activation links/registers this
            // path from a different cwd (e.g. ~/.claude/skills/<name>), where a
            // relative target would dangle.
            Ok(std::fs::canonicalize(&final_dir).unwrap_or(final_dir))
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

/// Drop the BYOH ownership marker into `dir`. Shared with the renderer so a
/// plain `byoh render` output is also recognizably BYOH-owned (re-render and
/// install guards both key off this marker).
pub fn write_owned_marker(dir: &Path, name: &str) -> Result<()> {
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

    // Phase 1: `agy plugin install` imports the plugin. It does NOT enable it
    // (agy keeps install + enable separate). Idempotent — "already" is fine.
    match commands.run("agy", &["plugin", "install", &dir], None) {
        CommandOutcome::Ran { .. } => {}
        CommandOutcome::Failed { stderr, .. } if stderr.to_lowercase().contains("already") => {}
        CommandOutcome::Failed { stderr, .. } => {
            return Ok(ActivationReport {
                host: Target::Agy,
                status: ActivationStatus::Failed,
                message: format!("agy plugin install failed: {stderr}"),
            });
        }
        CommandOutcome::NotInstalled => {
            return Ok(ActivationReport {
                host: Target::Agy,
                status: ActivationStatus::ManualStepsRequired,
                message: manual,
            });
        }
    }

    // Phase 2: enable so agy actually loads the plugin. Idempotent — an
    // "already enabled" failure is success.
    match commands.run("agy", &["plugin", "enable", &name], None) {
        CommandOutcome::Ran { .. } => Ok(ActivationReport {
            host: Target::Agy,
            status: ActivationStatus::Activated,
            message: format!(
                "installed + enabled; restart agy to load (copies to {})",
                loc.agy.display()
            ),
        }),
        CommandOutcome::Failed { stderr, .. }
            if stderr.to_lowercase().contains("already")
                || stderr.to_lowercase().contains("enabled") =>
        {
            Ok(ActivationReport {
                host: Target::Agy,
                status: ActivationStatus::Activated,
                message: "plugin already enabled; restart agy to load".into(),
            })
        }
        CommandOutcome::Failed { stderr, .. } => Ok(ActivationReport {
            host: Target::Agy,
            status: ActivationStatus::Failed,
            message: format!("agy plugin enable failed: {stderr}"),
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
            agy: dist.join("agy"),
            claude_config: dist.join("claude-config"),
        }
    }

    #[test]
    fn install_renders_polyglot_to_dist_atomic() {
        let bundle = compile_profile(&confirmed()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let loc = locs(dir.path());
        let out = install_plugin(&bundle, &loc, false).unwrap();
        assert!(out.ends_with("byoh-dev"));
        assert!(out.join(OWNED_MARKER).exists(), "marker must be present");
        // polyglot: all three hosts' manifests in one tree.
        assert!(out.join(".claude-plugin/plugin.json").exists());
        assert!(out.join(".codex-plugin/plugin.json").exists());
        assert!(out.join("plugin.json").exists(), "agy root plugin.json");
        // no staging/backup left behind
        assert!(!dir.path().join(".byoh-dev.staging").exists());
        assert!(!dir.path().join(".byoh-dev.bak").exists());
    }

    #[test]
    fn reinstall_is_idempotent_over_byoh_owned() {
        let bundle = compile_profile(&confirmed()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let loc = locs(dir.path());
        let out1 = install_plugin(&bundle, &loc, false).unwrap();
        // second install over the byoh-owned dir succeeds without --force
        let out2 = install_plugin(&bundle, &loc, false).unwrap();
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

        let err = install_plugin(&bundle, &loc, false).unwrap_err();
        assert!(matches!(err, ByohError::Other(_)));
        // user's file untouched
        assert!(target_dir.join("user-file.txt").exists());

        // with force, it installs (overwrites)
        install_plugin(&bundle, &loc, true).unwrap();
        assert!(target_dir.join(OWNED_MARKER).exists());
    }

    #[test]
    fn rejects_bad_slug() {
        let mut p = confirmed();
        p.slug = "../evil".into();
        let bundle = compile_profile(&p).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let loc = locs(dir.path());
        assert!(install_plugin(&bundle, &loc, false).is_err());
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

    /// Records every `run()` call and reports the configured tools as installed,
    /// returning `Ran`. For activation success-path tests (proves which host CLI
    /// subcommands were actually invoked).
    struct RecordingCli {
        installed: Vec<String>,
        calls: std::cell::RefCell<Vec<(String, Vec<String>)>>,
    }
    impl RecordingCli {
        fn new(installed: &[&str]) -> Self {
            Self {
                installed: installed.iter().map(|s| s.to_string()).collect(),
                calls: std::cell::RefCell::new(Vec::new()),
            }
        }
    }
    impl CommandPort for RecordingCli {
        fn run(&self, tool: &str, args: &[&str], _cwd: Option<&Path>) -> CommandOutcome {
            self.calls.borrow_mut().push((
                tool.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            ));
            CommandOutcome::Ran {
                stdout: String::new(),
            }
        }
        fn is_installed(&self, tool: &str) -> bool {
            self.installed.iter().any(|s| s == tool)
        }
    }

    #[test]
    fn agy_activation_runs_enable_after_install() {
        let dir = tempfile::tempdir().unwrap();
        let plugin = dir.path().join("byoh-dev");
        std::fs::create_dir_all(&plugin).unwrap();
        let loc = locs(dir.path());
        let cli = RecordingCli::new(&["agy"]);
        let r = activate_plugin(Target::Agy, &plugin, "dev", &loc, &cli).unwrap();
        assert_eq!(r.status, ActivationStatus::Activated);

        // install imports; a SEPARATE enable makes agy load it. Assert both ran.
        let calls = cli.calls.borrow();
        let ran = |sub: &str, name: Option<&str>| {
            calls.iter().any(|(t, a)| {
                t == "agy"
                    && a.get(1).map(|s| s.as_str()) == Some(sub)
                    && name.is_none_or(|n| a.get(2).map(|s| s.as_str()) == Some(n))
            })
        };
        assert!(
            ran("install", None),
            "must run agy plugin install, got {calls:?}"
        );
        assert!(
            ran("enable", Some("byoh-dev")),
            "must run agy plugin enable byoh-dev, got {calls:?}"
        );
    }

    /// install succeeds; enable reports "already enabled" (re-install idempotency).
    struct AgyAlreadyEnabled;
    impl CommandPort for AgyAlreadyEnabled {
        fn run(&self, _tool: &str, args: &[&str], _cwd: Option<&Path>) -> CommandOutcome {
            if args.get(1).copied() == Some("enable") {
                CommandOutcome::Failed {
                    code: 1,
                    stderr: "already enabled".into(),
                }
            } else {
                CommandOutcome::Ran {
                    stdout: String::new(),
                }
            }
        }
        fn is_installed(&self, _tool: &str) -> bool {
            true
        }
    }

    #[test]
    fn agy_enable_already_enabled_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let plugin = dir.path().join("byoh-dev");
        std::fs::create_dir_all(&plugin).unwrap();
        let loc = locs(dir.path());
        let r = activate_plugin(Target::Agy, &plugin, "dev", &loc, &AgyAlreadyEnabled).unwrap();
        assert_eq!(r.status, ActivationStatus::Activated);
    }

    #[test]
    fn with_claude_config_is_immutable_copy() {
        let dir = tempfile::tempdir().unwrap();
        let loc = locs(dir.path());
        let original_claude = loc.claude_config.clone();
        let local = loc.with_claude_config(PathBuf::from("./.claude"));
        // copy got the new root...
        assert_eq!(local.claude_config, PathBuf::from("./.claude"));
        // ...original is untouched, and other roots are carried over.
        assert_eq!(loc.claude_config, original_claude);
        assert_eq!(local.agy, loc.agy);
        assert_eq!(local.dist, loc.dist);
    }

    #[test]
    fn local_scope_activates_into_project_claude_root() {
        // `Scope::Local` routes Claude activation into a project-local
        // .claude/ instead of HOME: simulate by pointing claude_config at a
        // tempdir "project root" and activating normally.
        let dir = tempfile::tempdir().unwrap();
        let plugin = dir.path().join("byoh-dev");
        std::fs::create_dir_all(&plugin).unwrap();
        let project_root = dir.path().join("project");
        let loc = locs(dir.path()).with_claude_config(project_root.join(".claude"));
        let r = activate_plugin(Target::Claude, &plugin, "dev", &loc, &MissingCli).unwrap();
        let link = project_root.join(".claude").join("skills").join("byoh-dev");
        assert!(
            link.exists(),
            "local skills-dir link must exist under project"
        );
        assert_eq!(r.status, ActivationStatus::Activated);
    }

    #[test]
    fn resolve_scope_back_compat_and_conflicts() {
        use crate::domain::scope::Scope;
        // No flags → legacy no-activation default.
        assert_eq!(resolve_scope(None, false).unwrap(), Scope::DistOnly);
        // --host alone → legacy global.
        assert_eq!(resolve_scope(None, true).unwrap(), Scope::Global);
        // --scope alone parses.
        assert_eq!(
            resolve_scope(Some("local".into()), false).unwrap(),
            Scope::Local
        );
        assert_eq!(
            resolve_scope(Some("publish".into()), false).unwrap(),
            Scope::Publish
        );
        // --host + --scope global → same intent, OK.
        assert_eq!(
            resolve_scope(Some("global".into()), true).unwrap(),
            Scope::Global
        );
        // --host + --scope <other> → conflict, must error.
        assert!(resolve_scope(Some("local".into()), true).is_err());
        assert!(resolve_scope(Some("publish".into()), true).is_err());
        // Bad scope string → error.
        assert!(resolve_scope(Some("nope".into()), false).is_err());
    }
}

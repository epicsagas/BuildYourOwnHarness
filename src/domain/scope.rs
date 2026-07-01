//! Install scope — *where* a compiled harness goes after `install_plugin`
//! writes the polyglot tree to `dist/`.
//!
//! Pure enum + parsing, mirroring [`crate::domain::render_target::Target`]. The
//! activation/dispatch logic lives in `deploy::install` and `main::run_install`.

use crate::domain::error::ByohError;

/// Where an installed harness should land. `install_plugin` always writes the
/// polyglot tree to `dist/` first; the scope decides what happens next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Write `dist/` only, no activation. The default when neither `--host` nor
    /// `--scope` is given (back-compat with the pre-scope `install` behavior).
    DistOnly,
    /// Activate into the *current project's* host roots (e.g. `./.claude/skills/`),
    /// not the user's HOME. Only Claude has a project-local mode; codex/agy are
    /// HOME-based CLIs and are skipped with a notice.
    Local,
    /// Activate into the user's HOME host roots (`~/.claude`, `~/.codex`,
    /// `~/.gemini`). Equivalent to the legacy `--host` flag.
    Global,
    /// Add `LICENSE` + `.gitignore` to the `dist/` tree so it is ready to
    /// `git init && git push`. No activation; git operations are the user's.
    Publish,
}

impl Scope {
    pub fn as_str(self) -> &'static str {
        match self {
            Scope::DistOnly => "dist-only",
            Scope::Local => "local",
            Scope::Global => "global",
            Scope::Publish => "publish",
        }
    }

    /// Whether this scope activates the harness on a host (Local/Global).
    /// DistOnly and Publish only touch the `dist/` tree.
    pub fn activates(self) -> bool {
        matches!(self, Scope::Local | Scope::Global)
    }
}

impl std::str::FromStr for Scope {
    type Err = ByohError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "local" | "project" => Scope::Local,
            "global" | "host" => Scope::Global,
            "publish" | "release" => Scope::Publish,
            other => {
                return Err(ByohError::Schema(format!(
                    "unknown scope '{other}' (local|global|publish)"
                )));
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_variants() {
        assert_eq!("local".parse::<Scope>().unwrap(), Scope::Local);
        assert_eq!("GLOBAL".parse::<Scope>().unwrap(), Scope::Global);
        assert_eq!("publish".parse::<Scope>().unwrap(), Scope::Publish);
        // Aliases map onto the canonical variants.
        assert_eq!("project".parse::<Scope>().unwrap(), Scope::Local);
        assert_eq!("host".parse::<Scope>().unwrap(), Scope::Global);
        assert_eq!("release".parse::<Scope>().unwrap(), Scope::Publish);
    }

    #[test]
    fn rejects_unknown() {
        assert!("nope".parse::<Scope>().is_err());
        // DistOnly is never produced from a string — it is programmatic only.
        assert!("dist-only".parse::<Scope>().is_err());
    }

    #[test]
    fn activates_flag() {
        assert!(Scope::Local.activates());
        assert!(Scope::Global.activates());
        assert!(!Scope::DistOnly.activates());
        assert!(!Scope::Publish.activates());
    }
}

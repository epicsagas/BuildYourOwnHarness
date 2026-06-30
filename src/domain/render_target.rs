//! Render targets — which host tool a rendered plugin is shaped for.
//!
//! Pure enum + parsing. The actual file emission lives in
//! [`crate::application::render_plugin`].

use crate::domain::error::ByohError;

/// A plugin target host. `All` renders every concrete target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Claude Code — `.claude-plugin/` + root `skills/`/`agents/`/`commands/`.
    Claude,
    /// Codex — `.codex-plugin/` (TOML agents) + optional `.codex/`.
    Codex,
    /// Antigravity (agy) — root `agents/`/`skills/`/`hooks/` (no plugin dir).
    Agy,
    /// Render every concrete target into its own subdir of `out/`.
    All,
}

impl Target {
    /// The concrete targets a given Target expands to (`All` → all three).
    pub fn concrete(self) -> &'static [Target] {
        match self {
            Target::All => &[Target::Claude, Target::Codex, Target::Agy],
            Target::Claude => &[Target::Claude],
            Target::Codex => &[Target::Codex],
            Target::Agy => &[Target::Agy],
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Target::Claude => "claude",
            Target::Codex => "codex",
            Target::Agy => "agy",
            Target::All => "all",
        }
    }
}

impl std::str::FromStr for Target {
    type Err = ByohError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "claude" | "claude-code" => Target::Claude,
            "codex" => Target::Codex,
            "agy" | "antigravity" => Target::Agy,
            "all" => Target::All,
            other => {
                return Err(ByohError::Schema(format!(
                    "unknown render target '{other}' (claude|codex|agy|all)"
                )));
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_expands_to_three() {
        assert_eq!(
            Target::All.concrete(),
            &[Target::Claude, Target::Codex, Target::Agy]
        );
    }

    #[test]
    fn single_expands_to_itself() {
        assert_eq!(Target::Codex.concrete(), &[Target::Codex]);
    }

    #[test]
    fn parse_variants() {
        assert_eq!("claude".parse::<Target>().unwrap(), Target::Claude);
        assert_eq!("AGY".parse::<Target>().unwrap(), Target::Agy);
        assert_eq!("all".parse::<Target>().unwrap(), Target::All);
        assert!("nope".parse::<Target>().is_err());
    }
}

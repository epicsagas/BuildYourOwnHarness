//! `HarnessBundle` — the compiled output (ARCH §5).
//!
//! A bundle is the 4-Ring skeleton materialized as on-disk files:
//!   - `config/harness.toml`
//!   - `skills/<ring>/<name>.md`
//!   - `mcp/tools/<name>.json`
//!   - `hooks/hooks.json`
//!   - `evolution_policy.toml`

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::genre::{Genre, SafetyGates};

/// Which 4-Ring tier a skill/hook belongs to (ARCH §5.2, B8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ring {
    /// Auto hooks (SessionStart / PreToolUse / PostToolUse / SessionEnd).
    Ring0,
    /// Pipeline (spec → go → check → ship).
    Ring1,
    /// Quality (debug / secure / perf / genre-specific).
    Ring2,
    /// Evolution (Critic / Seesaw / Stagnation + skills).
    Ring3,
}

impl Ring {
    pub fn as_str(self) -> &'static str {
        use Ring::*;
        match self {
            Ring0 => "ring0",
            Ring1 => "ring1",
            Ring2 => "ring2",
            Ring3 => "ring3",
        }
    }
    pub fn all() -> [Ring; 4] {
        [Ring::Ring0, Ring::Ring1, Ring::Ring2, Ring::Ring3]
    }
}

/// A rendered skill (ARCH §5 step 2). Body is markdown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillSpec {
    pub id: String,
    pub ring: Ring,
    pub name: String,
    pub description: String,
    pub body_markdown: String,
    /// Pipeline membership: `None` = standalone skill (default, backward-compatible);
    /// `Some(id)` = this skill belongs to an ordered pipeline chain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline: Option<String>,
    /// Position within the pipeline (1-based). `None` = order-independent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>,
}

/// A rendered agent definition (target-renderer input). Becomes
/// `agents/<name>.md` (Claude Code / agy) or `.codex-plugin/agents/<name>.toml`
/// (Codex) when rendered to a plugin. `tools` is Claude-only — Codex drops it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSpec {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Markdown body WITHOUT frontmatter (frontmatter is added per-target at
    /// render time).
    pub body_markdown: String,
    /// Claude Code agent tools (e.g. `["Read","Write","Bash"]`). None/empty on
    /// Codex (its TOML agent schema has no tools key — verified).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
}

/// HookInput required fields (research §2.4.2, ARCH §5.4). A bundle's hooks.json
/// must declare these — the static gate enforces it (R8/AC7).
pub const HOOK_REQUIRED_FIELDS: &[&str] = &[
    "tool_name",
    "tool_input",
    "hook_event_name",
    "context_usage",
];

/// A hook entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HookSpec {
    pub event: String,
    pub command: String,
    /// Declares which HookInput fields this hook reads.
    pub reads: Vec<String>,
}

/// An MCP tool blueprint rendered to JSON (B4 self-describing). `description`
/// embeds agent-dispatch logic (ARCH §5.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    /// Raw JSON Schema for input. Validated by the static gate (R8).
    pub input_schema: serde_json::Value,
}

impl McpTool {
    /// B4 check: must have non-empty name + description + a JSON-schema input.
    pub fn is_well_formed(&self) -> bool {
        !self.name.trim().is_empty()
            && !self.description.trim().is_empty()
            && self.input_schema.get("type").is_some()
    }
}

/// `config/harness.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BundleConfig {
    pub slug: String,
    pub genre: Genre,
    pub profile_version: String,
    pub depends_on: Vec<DependencyPin>,
    #[serde(default)]
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencyPin {
    pub id: String,
    pub min_version: String,
}

/// Semantic bundle version (ARCH §9.1 `bundle_version`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BundleVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl BundleVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
    pub fn as_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl std::fmt::Display for BundleVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// The full compiled harness bundle (ARCH §5.1 Bundle subgraph).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HarnessBundle {
    pub config: BundleConfig,
    pub version: BundleVersion,
    pub genre: Genre,
    pub slug: String,
    pub skills: Vec<SkillSpec>,
    pub hooks: Vec<HookSpec>,
    pub mcp_tools: Vec<McpTool>,
    /// Agent definitions rendered to agents/<name>.md (Claude/agy) or
    /// .codex-plugin/agents/<name>.toml (Codex). Empty by default for
    /// backward compatibility with pre-agent bundles.
    #[serde(default)]
    pub agents: Vec<AgentSpec>,
    /// evolution_policy.toml content (B10). safety_gates MUST be all three.
    pub safety_gates: SafetyGates,
    pub stagnation_limit: u32,
    pub improvement_threshold: f64,
    /// sha256 of the source ConfirmedProfile (ARCH §9.1 source_profile_hash).
    pub source_profile_hash: String,
    /// Profile language code (e.g. "en", "ko"). Drives user-facing doc
    /// localization (README); AI-facing instructions (skills/agents/AGENTS.md)
    /// stay English. Defaults to "en" for pre-language bundles.
    #[serde(default = "default_language")]
    pub language: String,
    /// The skill id a session starts at — the first step of the Ring 1
    /// pipeline. Drives the "Entry rule" line in the getting-started doc so it
    /// reflects the genre's actual entry (`spec` for dev/researcher, `goal` for
    /// business, `draft` for creator) instead of a hardcoded `spec`. `None` for
    /// pre-entry-skill bundles → the renderer falls back to `spec`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_skill: Option<String>,
}

/// Serde default for [`HarnessBundle::language`]. Old bundles deserialize
/// without a language field → treated as English.
fn default_language() -> String {
    "en".into()
}

impl HarnessBundle {
    pub fn skills_for(&self, ring: Ring) -> impl Iterator<Item = &SkillSpec> {
        self.skills.iter().filter(move |s| s.ring == ring)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_string() {
        assert_eq!(BundleVersion::new(1, 2, 0).as_string(), "1.2.0");
    }

    #[test]
    fn hook_required_fields_present() {
        assert!(HOOK_REQUIRED_FIELDS.contains(&"tool_name"));
        assert_eq!(HOOK_REQUIRED_FIELDS.len(), 4);
    }
}

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
    /// evolution_policy.toml content (B10). safety_gates MUST be all three.
    pub safety_gates: SafetyGates,
    pub stagnation_limit: u32,
    pub improvement_threshold: f64,
    /// sha256 of the source ConfirmedProfile (ARCH §9.1 source_profile_hash).
    pub source_profile_hash: String,
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

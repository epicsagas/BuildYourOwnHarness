//! MCP tool parameter structs.
//!
//! Each derives `serde::Deserialize + schemars::JsonSchema` (required by rmcp so
//! the tool's `inputSchema` is derived automatically). Only **primitive** types
//! are used here — BYOH domain types (which derive `Serialize` but **not**
//! `JsonSchema`) are returned from tools as opaque `serde_json::Value` instead,
//! so we never touch the domain derives.

use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProfileReadParams {
    /// Profile slug (the `<slug>` in `<BYOH_HOME>/profiles/<slug>.yaml`).
    pub slug: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProfileCreateParams {
    pub slug: String,
    /// Optional paths to autoscan (S1 non-destructive collection).
    #[serde(default)]
    pub scan_paths: Vec<String>,
    /// Language code, "ko" or "en" (default "ko").
    #[serde(default)]
    pub language: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProfileScanParams {
    pub slug: String,
    /// Filesystem paths to scan non-destructively.
    pub paths: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProfileInterviewParams {
    pub slug: String,
    /// Answers keyed by question id: (answer_text, confidence 0.0–1.0).
    /// Empty (default) ⇒ auto-accept rule-based suggestions.
    #[serde(default)]
    pub answers: HashMap<String, (String, f64)>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProfileConfirmParams {
    pub slug: String,
    /// Genre: "developer" | "creator" | "researcher" | "business".
    pub genre: String,
    /// Optional 30-day goal string.
    #[serde(default)]
    pub goal_30d: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompileParams {
    pub slug: String,
    /// If true, also run the static gate and include its report.
    #[serde(default = "default_true")]
    pub run_static_gate: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompileDryRunParams {
    pub slug: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EvolveCycleParams {
    /// Profile slug — keys the persisted seesaw/stagnation state across runs.
    pub slug: String,
    pub genre: String,
    /// Edit type: "AddSkill" | "ModifyInstinct" | "ModifyConfig" | "AddGuardRule"
    /// | "ModifyPrompt" | "RemoveSkill".
    pub edit_type: String,
    pub metric: EvolveMetricParams,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EvolveMetricParams {
    /// Avg score with the proposed edit.
    #[serde(rename = "with")]
    pub with_: f64,
    /// Avg score without the proposed edit.
    pub without: f64,
    pub samples_with: u32,
    pub samples_without: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RegistryCloneSkillParams {
    pub genre: String,
    /// Preset skill id, e.g. "tdd" or "debug".
    pub skill_id: String,
    pub slug: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenderPluginParams {
    pub slug: String,
    /// Target host: "claude" | "codex" | "agy" | "all" (default "all").
    #[serde(default = "default_target")]
    pub target: String,
    /// Output directory for the deployable plugin tree.
    pub out: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InstallPluginParams {
    pub slug: String,
    /// Target host: "claude" | "codex" | "agy" | "all" (default "all").
    #[serde(default = "default_target")]
    pub target: String,
    /// Install into the host's real plugin dir (~/.claude etc.) instead of the
    /// safe project-local `dist/`. Default false.
    #[serde(default)]
    pub host: bool,
    /// Overwrite a non-BYOH directory of the same name. Default false.
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CatalogSearchParams {
    /// Natural-language query (searched in name, id, keywords, description).
    pub query: String,
    /// Optional genre filter: "developer" | "creator" | "researcher" | "business".
    #[serde(default)]
    pub genre: Option<String>,
    /// AND-filter tags. Entry must contain ALL of these.
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_k")]
    pub limit: usize,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CatalogVendorParams {
    /// Plugin id as listed in the catalog ("owner/repo" slug).
    pub plugin_id: String,
    /// Genre override: "developer" | "creator" | "researcher" | "business".
    /// Required when the catalog entry has no inferred genre.
    #[serde(default)]
    pub genre: Option<String>,
    /// Extra keywords to merge into the vendored entry.
    #[serde(default)]
    pub extra_keywords: Vec<String>,
}

fn default_target() -> String {
    "all".to_string()
}

fn default_k() -> usize {
    5
}
fn default_true() -> bool {
    true
}

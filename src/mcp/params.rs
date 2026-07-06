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
pub struct BuildParams {
    pub slug: String,
    /// If true, also run the dry-run gate (deps missing → graceful fallback).
    /// The static gate always runs (built into `synthesize`).
    #[serde(default)]
    pub run_dry_run: bool,
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
    /// Where the installed harness goes: "local" (this project's .claude/),
    /// "global" (HOME, same as host=true), or "publish" (add LICENSE + .gitignore,
    /// no activation, return git instructions). Conflicts with host unless
    /// "global". Omit to write dist/ only.
    #[serde(default)]
    pub scope: Option<String>,
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

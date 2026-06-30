//! The BYOH stdio MCP server (`byoh serve`).
//!
//! Wraps BYOH's synchronous lib APIs as MCP tools so an LLM agent can drive the
//! whole profile → compile → evolve flow. The `#[tool_router(server_handler)]`
//! macro generates the `ServerHandler` impl; each `#[tool]` method is a plain
//! sync `fn` that calls BYOH directly (the lib has no async surface). Domain
//! data is returned as opaque `serde_json::Value`.

use std::path::PathBuf;
use std::sync::Arc;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::tool;
use rmcp::tool_handler;
use rmcp::tool_router;
use rmcp::transport::stdio;
use rmcp::{ServerHandler, ServiceExt};
use serde_json::{Value, json};

use crate::adapters::{FilesystemSource, RuleInterview, RuleLlm, StaticWizard};
use crate::application::ProfileOrchestrator;
use crate::domain::bundle::HarnessBundle;
use crate::domain::error::ByohError;
use crate::domain::evidence::AbMetric;
use crate::domain::genre::Genre;
use crate::domain::profile::UserProfile;
use crate::evolve::EvolutionDecision;

use super::params::*;

/// Fixed-at-startup runtime context shared (immutable) across all tool calls.
pub struct ByohContext {
    /// `$BYOH_HOME` root (profiles under `<home>/profiles/`).
    pub home: PathBuf,
    /// "ko" or "en".
    pub language: String,
}

/// The MCP server. Holds an immutable `Arc<ByohContext>`; tool methods borrow it.
#[derive(Clone)]
pub struct ByohServer {
    ctx: Arc<ByohContext>,
}

impl ByohServer {
    pub fn new(ctx: ByohContext) -> Self {
        Self { ctx: Arc::new(ctx) }
    }

    /// Run the stdio MCP server until the client disconnects.
    ///
    /// Returns a `String` error (Send + Sync) so the synchronous binary entry
    /// point can surface it via `anyhow`.
    pub async fn serve_stdio(self) -> Result<(), String> {
        let service = self
            .serve(stdio())
            .await
            .map_err(|e| format!("MCP serve init failed: {e}"))?;
        service
            .waiting()
            .await
            .map_err(|e| format!("MCP serve stopped: {e}"))?;
        Ok(())
    }
}

// ─── helpers ────────────────────────────────────────────────────────────────

/// Build a rule-based orchestrator (no network, no model). Owned locally.
fn orchestrator() -> (
    FilesystemSource,
    RuleLlm,
    RuleInterview<RuleLlm>,
    StaticWizard,
) {
    (
        FilesystemSource::new(),
        RuleLlm::new(),
        RuleInterview::new(RuleLlm::new()),
        StaticWizard::new(),
    )
}

fn parse_genre(s: &str) -> Result<Genre, ByohError> {
    s.parse::<Genre>()
}

/// Successful tool result carrying a JSON value as a text content block.
fn ok_value(v: Value) -> CallToolResult {
    CallToolResult::success(vec![rmcp::model::Content::text(v.to_string())])
}

/// Error tool result. Maps BYOH errors to human-readable text; the MCP layer
/// surfaces this as an `is_error` result the agent can read.
fn err_result(e: ByohError) -> CallToolResult {
    let kind = match &e {
        ByohError::InvalidTransition { .. }
        | ByohError::Schema(_)
        | ByohError::MissingTruth { .. }
        | ByohError::ValidationGateFailed { .. }
        | ByohError::SafetyGateMissing { .. } => "invalid_params",
        ByohError::DependencyMissing { .. } => "dependency_missing",
        _ => "internal_error",
    };
    CallToolResult::error(vec![rmcp::model::Content::text(format!("[{kind}] {e}"))])
}

/// Convert a `byoh::Result<T>` (where T: Serialize) into a tool result.
macro_rules! tool_result {
    ($expr:expr) => {
        match $expr {
            Ok(v) => ok_value(serde_json::to_value(&v).unwrap_or(Value::Null)),
            Err(e) => err_result(e),
        }
    };
}

/// Project a `StaticGateReport` (non-Serialize) into JSON.
fn static_gate_json(r: &crate::compiler::StaticGateReport) -> Value {
    json!({
        "mcp_schema_ok": r.mcp_schema_ok,
        "hook_input_ok": r.hook_input_ok,
        "safety_gates_ok": r.safety_gates_ok,
        "errors": r.errors,
        "passed": r.passed(),
    })
}

/// Project a `DryRunReport` (non-Serialize) into JSON.
fn dry_run_json(r: &crate::compiler::DryRunReport) -> Value {
    json!({
        "spec_ok": r.spec_ok,
        "go_ok": r.go_ok,
        "check_ok": r.check_ok,
        "tools_list_ok": r.tools_list_ok,
        "fallbacks": r.fallbacks,
        "passed": r.passed(),
    })
}

/// Compact profile snapshot — the focused status an agent needs to drive the
/// wizard, instead of echoing the full `UserProfile` on every call. Each wizard
/// step (create/scan/interview/confirm) returns this; the full profile is
/// available on demand via `profile_read`. This keeps the conversation context
/// from re-growing by the profile size on every turn (token optimization).
fn compact_status(profile: &UserProfile) -> Value {
    let ac = &profile.interview_meta.axis_completion;
    json!({
        "slug": profile.slug,
        "status": format!("{:?}", profile.status),
        "language": profile.language,
        "interview_complete": ac.all_above_threshold(),
        "axis_completion": {
            "identity": ac.tacit,
            "goals": ac.goals,
            "genre": ac.genre,
            "data": ac.data,
        },
        "genre": profile.candidates.identity.genre.as_ref().map(|g| json!({
            "value": g.value.as_str(),
            "confidence": g.confidence,
        })),
        "domain": profile.truth.identity.domain,
        "goal_30d": profile.truth.goals.goal_30d,
        "primary_expertise": profile
            .candidates
            .identity
            .primary_expertise
            .iter()
            .map(|f| f.value.clone())
            .collect::<Vec<_>>(),
        "data_sources": profile.data_sources.sources.len(),
    })
}

// ─── tools ──────────────────────────────────────────────────────────────────

#[tool_router]
impl ByohServer {
    #[tool(description = "Read a BYOH user profile by slug. Returns the full profile as JSON.")]
    pub fn profile_read(&self, Parameters(p): Parameters<ProfileReadParams>) -> CallToolResult {
        tool_result!(crate::store::load_profile(&p.slug))
    }

    #[tool(
        description = "Create a draft profile (and optionally autoscan paths). Returns the created profile."
    )]
    pub fn profile_create(&self, Parameters(p): Parameters<ProfileCreateParams>) -> CallToolResult {
        let lang = p.language.as_deref().unwrap_or("ko");
        let mut profile = UserProfile::new_draft(&p.slug, lang);
        if !p.scan_paths.is_empty() {
            let (src, llm, iv, wz) = orchestrator();
            let orch = ProfileOrchestrator::new(&src, &llm, &iv, &wz);
            let path_refs: Vec<&std::path::Path> =
                p.scan_paths.iter().map(|s| s.as_str().as_ref()).collect();
            if let Err(e) = orch.stage1_scan(&mut profile, &path_refs) {
                return err_result(e);
            }
        }
        match crate::store::write_profile(&profile) {
            Ok(()) => ok_value(compact_status(&profile)),
            Err(e) => err_result(e),
        }
    }

    #[tool(
        description = "Run non-destructive autoscan (S1) over the given paths, merging derived candidates into the profile. Filesystem-walk heavy: runs off the async runtime via spawn_blocking."
    )]
    pub async fn profile_scan(
        &self,
        Parameters(p): Parameters<ProfileScanParams>,
    ) -> CallToolResult {
        // Move owned data into the blocking task (no borrowed refs).
        let res = tokio::task::spawn_blocking(move || profile_scan_sync(&p.slug, &p.paths)).await;
        match res {
            Ok(r) => r,
            Err(join_err) => err_result(crate::domain::error::ByohError::Other(format!(
                "scan task join failed: {join_err}"
            ))),
        }
    }

    #[tool(
        description = "Run the S2 interview step. Empty answers auto-accept rule-based suggestions. Returns the updated profile + any surfaced council questions."
    )]
    pub fn profile_interview(
        &self,
        Parameters(p): Parameters<ProfileInterviewParams>,
    ) -> CallToolResult {
        let mut profile = match crate::store::load_profile(&p.slug) {
            Ok(p) => p,
            Err(e) => return err_result(e),
        };
        let (src, llm, iv, wz) = orchestrator();
        let orch = ProfileOrchestrator::new(&src, &llm, &iv, &wz);
        match orch.stage2_interview(&mut profile, &p.answers) {
            Ok(council) => {
                let _ = crate::store::write_profile(&profile);
                // Enrich with catalog suggestions so the LLM can recommend plugins
                // mid-interview without a separate tool call.
                let tags = crate::application::synthesis::profile_tags(&profile);
                let catalog_suggestions = match crate::catalog::search::catalog_search(
                    &self.ctx.home,
                    &crate::catalog::search::SearchOptions {
                        query: &tags.join(" "),
                        genre: profile.candidates.identity.genre.as_ref().map(|g| g.value),
                        tags: &[],
                        limit: 5,
                    },
                ) {
                    Ok(entries) => entries
                        .into_iter()
                        .map(|e| json!({ "id": e.id, "name": e.name }))
                        .collect::<Vec<_>>(),
                    Err(_) => vec![],
                };
                ok_value(json!({
                    "status": compact_status(&profile),
                    "council_questions": council
                        .iter()
                        .map(|q| json!({ "id": q.id, "text": q.text }))
                        .collect::<Vec<_>>(),
                    "catalog_suggestions": catalog_suggestions,
                }))
            }
            Err(e) => err_result(e),
        }
    }

    #[tool(
        description = "Confirm the profile (S3): freeze genre + goal and transition to Confirmed. Requires genre."
    )]
    pub fn profile_confirm(
        &self,
        Parameters(p): Parameters<ProfileConfirmParams>,
    ) -> CallToolResult {
        let genre = match parse_genre(&p.genre) {
            Ok(g) => g,
            Err(e) => return err_result(e),
        };
        let mut profile = match crate::store::load_profile(&p.slug) {
            Ok(p) => p,
            Err(e) => return err_result(e),
        };
        let (src, llm, iv, wz) = orchestrator();
        let orch = ProfileOrchestrator::new(&src, &llm, &iv, &wz);
        match orch.stage3_confirm(&mut profile, genre, p.goal_30d.as_deref()) {
            Ok(()) => match crate::store::write_profile(&profile) {
                Ok(()) => ok_value(compact_status(&profile)),
                Err(e) => err_result(e),
            },
            Err(e) => err_result(e),
        }
    }

    #[tool(
        description = "List the four BYOH genres (developer/creator/researcher/business) with MVP flags."
    )]
    pub fn genre_list(&self) -> CallToolResult {
        let lib = crate::templates::TemplateLibrary::new();
        let out: Vec<Value> = lib
            .all()
            .into_iter()
            .map(|t| {
                json!({
                    "genre": t.genre.as_str(),
                    "mvp": t.mvp,
                })
            })
            .collect();
        ok_value(json!(out))
    }

    #[tool(
        description = "Compile a confirmed profile into a HarnessBundle (4-Ring). Optionally run the static gate. Returns the bundle (and gate report)."
    )]
    pub fn compile(&self, Parameters(p): Parameters<CompileParams>) -> CallToolResult {
        let profile = match crate::store::load_profile(&p.slug) {
            Ok(p) => p,
            Err(e) => return err_result(e),
        };
        match crate::compiler::compile_profile(&profile) {
            Ok(bundle) => {
                if p.run_static_gate {
                    match crate::compiler::static_gate(&bundle) {
                        Ok(report) => ok_value(json!({
                            "bundle": serde_json::to_value(&bundle).unwrap_or(Value::Null),
                            "static_gate": static_gate_json(&report),
                        })),
                        Err(e) => err_result(e),
                    }
                } else {
                    ok_value(serde_json::to_value(&bundle).unwrap_or(Value::Null))
                }
            }
            Err(e) => err_result(e),
        }
    }

    #[tool(
        description = "Compile + static gate + dry-run a profile. Returns whether each gate passed (deps missing are a graceful fallback, not an error). Dry-run shells out to dependency tools: runs off the async runtime via spawn_blocking."
    )]
    pub async fn compile_dry_run(
        &self,
        Parameters(p): Parameters<CompileDryRunParams>,
    ) -> CallToolResult {
        let home = self.ctx.home.clone();
        let res = tokio::task::spawn_blocking(move || compile_dry_run_sync(&home, &p.slug)).await;
        match res {
            Ok(r) => r,
            Err(join_err) => err_result(crate::domain::error::ByohError::Other(format!(
                "dry_run task join failed: {join_err}"
            ))),
        }
    }

    #[tool(
        description = "Run one Ring-3 evolution cycle (Observe→Analyze→Evolve→Gate) under the 3 safety gates, PERSISTING seesaw/stagnation state across runs by slug. Returns the honest decision (Approved/Rejected/RolledBack/AutoTuned) + reason + cycle number."
    )]
    pub fn evolve_cycle(&self, Parameters(p): Parameters<EvolveCycleParams>) -> CallToolResult {
        let genre = match parse_genre(&p.genre) {
            Ok(g) => g,
            Err(e) => return err_result(e),
        };
        let edit = match crate::application::evolve_run::parse_edit_type(&p.edit_type) {
            Ok(e) => e,
            Err(e) => return err_result(e),
        };
        let metric = AbMetric {
            avg_score_with: p.metric.with_,
            avg_score_without: p.metric.without,
            samples_with: p.metric.samples_with,
            samples_without: p.metric.samples_without,
        };
        match crate::application::evolve_one_cycle(&self.ctx.home, &p.slug, genre, edit, metric) {
            Ok((decision, state)) => {
                let label = crate::application::evolve_run::decision_label(&decision);
                let reason = match &decision {
                    EvolutionDecision::Rejected { reason }
                    | EvolutionDecision::RolledBack { reason } => reason.clone(),
                    _ => String::new(),
                };
                ok_value(json!({
                    "decision": label,
                    "reason": reason,
                    "cycle_n": state.cycle_n,
                    "negative": crate::application::evolve_run::decision_is_negative(&decision),
                }))
            }
            Err(e) => err_result(e),
        }
    }

    #[tool(
        description = "Clone a vetted preset skill (e.g. 'tdd', 'debug') into a compiled genre bundle. Compiles by slug first, then injects. Returns the enriched bundle. Generate + clone coexist (deduped by skill id)."
    )]
    pub fn registry_clone_skill(
        &self,
        Parameters(p): Parameters<RegistryCloneSkillParams>,
    ) -> CallToolResult {
        let genre = match parse_genre(&p.genre) {
            Ok(g) => g,
            Err(e) => return err_result(e),
        };
        let profile = match crate::store::load_profile(&p.slug) {
            Ok(p) => p,
            Err(e) => return err_result(e),
        };
        let mut bundle: HarnessBundle = match crate::compiler::compile_profile(&profile) {
            Ok(b) => b,
            Err(e) => return err_result(e),
        };
        match crate::deploy::presets::inject_preset(&mut bundle, genre, &p.skill_id) {
            Ok(()) => ok_value(serde_json::to_value(&bundle).unwrap_or(Value::Null)),
            Err(e) => err_result(e),
        }
    }

    #[tool(
        description = "Render a synthesized harness into a deployable plugin tree at `out`. target = claude|codex|agy|all. The output dir is git-ready — push it and others can use the plugin. Filesystem-heavy: runs off the async runtime via spawn_blocking."
    )]
    pub async fn render_plugin(
        &self,
        Parameters(p): Parameters<RenderPluginParams>,
    ) -> CallToolResult {
        let home = self.ctx.home.clone();
        let res = tokio::task::spawn_blocking(move || render_plugin_blocking(&home, &p)).await;
        match res {
            Ok(r) => r,
            Err(join_err) => err_result(crate::domain::error::ByohError::Other(format!(
                "render task join failed: {join_err}"
            ))),
        }
    }

    #[tool(
        description = "Render a synthesized harness plugin for a slug as a polyglot tree into the SAFE project-local dist/. Set host=true to additionally activate it so each host (claude/codex/agy) discovers the tree. Refuses to overwrite a non-BYOH dir unless force=true. Atomic. Filesystem-heavy: runs via spawn_blocking."
    )]
    pub async fn install_plugin(
        &self,
        Parameters(p): Parameters<InstallPluginParams>,
    ) -> CallToolResult {
        // Capture the resolved home + dist override on THIS thread before moving
        // into spawn_blocking: the worker thread can't see the thread-local
        // overrides (Rust 2024 + forbid(unsafe_code) means no set_var).
        let home = self.ctx.home.clone();
        let dist = crate::deploy::InstallLocations::from_env().dist;
        let res =
            tokio::task::spawn_blocking(move || install_plugin_blocking(&home, &dist, &p)).await;
        match res {
            Ok(r) => r,
            Err(join_err) => err_result(crate::domain::error::ByohError::Other(format!(
                "install task join failed: {join_err}"
            ))),
        }
    }

    #[tool(
        description = "Search the local plugin catalog cache (~/.byoh/catalog.json, top 100 by stars). \
                       Returns ranked plugin entries (id, name, description, github_url, stars, genre). \
                       Offline — no network. Run `byoh catalog index` to populate the cache first. \
                       Use this during the wizard to recommend external plugins the user can vendor."
    )]
    pub fn catalog_search(&self, Parameters(p): Parameters<CatalogSearchParams>) -> CallToolResult {
        let genre = match p.genre.as_deref().map(parse_genre).transpose() {
            Ok(g) => g,
            Err(e) => return err_result(e),
        };
        let opts = crate::catalog::search::SearchOptions {
            query: &p.query,
            genre,
            tags: &p.tags,
            limit: p.limit,
        };
        match crate::catalog::search::catalog_search(&self.ctx.home, &opts) {
            Ok(entries) => {
                let out: Vec<Value> = entries
                    .into_iter()
                    .map(|e| {
                        json!({
                            "id": e.id,
                            "name": e.name,
                            "description": e.description,
                            "github_url": e.github_url,
                            "stars": e.stars,
                            "genre": e.byoh_genre.map(|g| g.as_str()),
                            "keywords": e.keywords,
                        })
                    })
                    .collect();
                ok_value(json!(out))
            }
            Err(e) => err_result(e),
        }
    }

    #[tool(
        description = "Vendor a plugin from the catalog into registry/vendored/. \
                       Looks up `plugin_id` (owner/repo) in the local cache, shallow-clones its GitHub repo, \
                       and delegates to vendor_add. Run catalog_search first to find the right plugin_id. \
                       Returns the vendored entry (skill_id, genre, sha256). \
                       Filesystem + git heavy: runs via spawn_blocking."
    )]
    pub async fn catalog_vendor(
        &self,
        Parameters(p): Parameters<CatalogVendorParams>,
    ) -> CallToolResult {
        let home = self.ctx.home.clone();
        let res = tokio::task::spawn_blocking(move || catalog_vendor_blocking(&home, &p)).await;
        match res {
            Ok(r) => r,
            Err(join_err) => err_result(crate::domain::error::ByohError::Other(format!(
                "catalog_vendor task join failed: {join_err}"
            ))),
        }
    }
}

#[tool_handler]
impl ServerHandler for ByohServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        // Use Defaults + field assignment to stay forward-compatible with rmcp's
        // non-exhaustive model structs.
        let mut info = rmcp::model::ServerInfo::default();
        info.server_info = rmcp::model::Implementation::new("byoh", env!("CARGO_PKG_VERSION"));
        info.instructions = Some(
            "BYOH: build a personalized AI agent harness. Drive the flow: \
             profile_create → profile_scan → profile_interview → profile_confirm \
             → compile → compile_dry_run → registry_clone_skill. The conversation \
             IS the interview/wizard."
                .into(),
        );
        info
    }
}

// ─── blocking bodies for the heavy tools (run via spawn_blocking) ───────────

/// Synchronous body of `render_plugin`. Synthesizes the bundle then renders it
/// to the target host(s), writing a deployable plugin tree at `out`.
fn render_plugin_blocking(home: &std::path::Path, p: &RenderPluginParams) -> CallToolResult {
    let target: crate::domain::render_target::Target = match p.target.parse() {
        Ok(t) => t,
        Err(e) => return err_result(e),
    };
    let profile = match crate::store::load_profile_in(home, &p.slug) {
        Ok(pr) => pr,
        Err(e) => return err_result(e),
    };
    let (bundle, _plan) = match crate::application::synthesize(&profile) {
        Ok(b) => b,
        Err(e) => return err_result(e),
    };
    match crate::application::render_target(&bundle, target, std::path::Path::new(&p.out)) {
        Ok(root) => ok_value(json!({
            "rendered_to": root.to_string_lossy(),
            "target": target.as_str(),
            "skills": bundle.skills.len(),
            "agents": bundle.agents.len(),
            "git_ready": true,
        })),
        Err(e) => err_result(e),
    }
}

/// Synchronous body of `install_plugin`. Renders the polyglot tree to dist/;
/// if `host`, activates each selected host against it.
fn install_plugin_blocking(
    home: &std::path::Path,
    dist: &std::path::Path,
    p: &InstallPluginParams,
) -> CallToolResult {
    let target: crate::domain::render_target::Target = match p.target.parse() {
        Ok(t) => t,
        Err(e) => return err_result(e),
    };
    let profile = match crate::store::load_profile_in(home, &p.slug) {
        Ok(pr) => pr,
        Err(e) => return err_result(e),
    };
    let (bundle, _plan) = match crate::application::synthesize(&profile) {
        Ok(b) => b,
        Err(e) => return err_result(e),
    };
    let loc = crate::deploy::InstallLocations::from_env_with_dist(dist.to_path_buf());
    let path = match crate::deploy::install_plugin(&bundle, &loc, p.force) {
        Ok(path) => path,
        Err(e) => return err_result(e),
    };
    let mut result = json!({
        "installed_to": path.to_string_lossy(),
        "skills": bundle.skills.len(),
        "agents": bundle.agents.len(),
    });
    if p.host {
        let commands = crate::adapters::StdCommand::new();
        let mut activations = Vec::new();
        for t in target.concrete() {
            let entry = match crate::deploy::activate_plugin(*t, &path, &p.slug, &loc, &commands) {
                Ok(r) => json!({
                    "host": t.as_str(),
                    "status": format!("{:?}", r.status),
                    "message": r.message,
                }),
                Err(e) => json!({ "host": t.as_str(), "error": e.to_string() }),
            };
            activations.push(entry);
        }
        result["activations"] = json!(activations);
    } else {
        result["host"] = json!(false);
    }
    ok_value(result)
}

/// Synchronous body of `profile_scan`. Owned inputs so it can move into a
/// `spawn_blocking` task.
fn profile_scan_sync(slug: &str, paths: &[String]) -> CallToolResult {
    let mut profile = match crate::store::load_profile(slug) {
        Ok(p) => p,
        Err(e) => return err_result(e),
    };
    let (src, llm, iv, wz) = orchestrator();
    let orch = ProfileOrchestrator::new(&src, &llm, &iv, &wz);
    let path_refs: Vec<&std::path::Path> = paths.iter().map(|s| s.as_str().as_ref()).collect();
    match orch.stage1_scan(&mut profile, &path_refs) {
        Ok(()) => match crate::store::write_profile(&profile) {
            Ok(()) => ok_value(compact_status(&profile)),
            Err(e) => err_result(e),
        },
        Err(e) => err_result(e),
    }
}

/// Synchronous body of `compile_dry_run`.
fn compile_dry_run_sync(home: &std::path::Path, slug: &str) -> CallToolResult {
    let profile = match crate::store::load_profile_in(home, slug) {
        Ok(p) => p,
        Err(e) => return err_result(e),
    };
    let bundle = match crate::compiler::compile_profile(&profile) {
        Ok(b) => b,
        Err(e) => return err_result(e),
    };
    let static_report = match crate::compiler::static_gate(&bundle) {
        Ok(r) => r,
        Err(e) => return err_result(e),
    };
    // StdCommand shells out to dependency tools; missing tools are a graceful
    // fallback inside dry_run, surfaced in the report.
    let cmds = crate::adapters::StdCommand::new();
    match crate::compiler::dry_run(&bundle, &cmds) {
        Ok(dry_report) => ok_value(json!({
            "static_gate_passed": static_report.passed(),
            "dry_run_passed": dry_report.passed(),
            "static_gate": static_gate_json(&static_report),
            "dry_run": dry_run_json(&dry_report),
        })),
        Err(e) => err_result(e),
    }
}

/// Synchronous body of `catalog_vendor`.
fn catalog_vendor_blocking(home: &std::path::Path, p: &CatalogVendorParams) -> CallToolResult {
    let genre = match p.genre.as_deref().map(parse_genre).transpose() {
        Ok(g) => g,
        Err(e) => return err_result(e),
    };
    let mut cache = match crate::catalog::load_cache(home) {
        Ok(c) => c,
        Err(e) => return err_result(e),
    };
    let entry = match cache.entries.iter().find(|e| e.id == p.plugin_id) {
        Some(e) => e.clone(),
        None => {
            return err_result(crate::domain::ByohError::Schema(format!(
                "plugin '{}' not in catalog cache — run `byoh catalog index` first",
                p.plugin_id
            )));
        }
    };
    let repo_root = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return err_result(crate::domain::ByohError::Other(format!("cwd: {e}")));
        }
    };
    match crate::catalog::vendor_from_catalog::catalog_vendor(
        &entry,
        genre,
        &p.extra_keywords,
        &repo_root,
    ) {
        Ok((v, enrichment)) => {
            if let Some(cached) = cache.entries.iter_mut().find(|e| e.id == p.plugin_id) {
                if cached.license == "unknown" || cached.license.is_empty() {
                    cached.license = enrichment.license.clone();
                }
                if cached.keywords.is_empty() && !enrichment.keywords.is_empty() {
                    cached.keywords = enrichment.keywords.clone();
                }
                if cached.byoh_genre.is_none() {
                    cached.byoh_genre = Some(enrichment.genre);
                }
            }
            let _ = crate::catalog::save_cache(home, &cache);
            ok_value(json!({
                "skill_id": v.skill_id,
                "genre": v.genre,
                "sha256": v.sha256,
                "license": enrichment.license,
            }))
        }
        Err(e) => err_result(e),
    }
}

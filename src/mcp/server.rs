//! The BYOH stdio MCP server (`byoh serve`).
//!
//! Wraps BYOH's synchronous lib APIs as MCP tools so an LLM agent can drive the
//! whole profile → rag → compile → evolve flow. The `#[tool_router(server_handler)]`
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
use serde_json::{json, Value};

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
    /// `$BYOH_HOME` root (profiles under `<home>/profiles/`, indexes under
    /// `<home>/indexes/`).
    pub home: PathBuf,
    /// "ko" or "en".
    pub language: String,
    /// Whether the build was compiled with `native-rag` (selects the embedder
    /// backend for rag_index/rag_search).
    pub native_rag: bool,
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
            Ok(()) => ok_value(serde_json::to_value(&profile).unwrap_or(Value::Null)),
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
                ok_value(json!({
                    "profile": serde_json::to_value(&profile).unwrap_or(Value::Null),
                    "council_questions": council.len(),
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
                Ok(()) => ok_value(serde_json::to_value(&profile).unwrap_or(Value::Null)),
                Err(e) => err_result(e),
            },
            Err(e) => err_result(e),
        }
    }

    #[tool(
        description = "Build + persist a genre index over a corpus (S2). Returns a build report (docs/chunks/dim/backend). Embedding-heavy: runs off the async runtime via spawn_blocking."
    )]
    pub async fn rag_index(&self, Parameters(p): Parameters<RagIndexParams>) -> CallToolResult {
        let home = self.ctx.home.clone();
        let res = tokio::task::spawn_blocking(move || rag_index_blocking(&home, &p)).await;
        match res {
            Ok(r) => r,
            Err(join_err) => err_result(crate::domain::error::ByohError::Other(format!(
                "rag_index task join failed: {join_err}"
            ))),
        }
    }

    #[tool(
        description = "Hybrid search a query against a corpus (or grep-only if no corpus). Returns ranked hits with mode/score. Secrets in results are masked. Build/search-heavy: runs off the async runtime via spawn_blocking."
    )]
    pub async fn rag_search(&self, Parameters(p): Parameters<RagSearchParams>) -> CallToolResult {
        let res = tokio::task::spawn_blocking(move || rag_search_blocking(&p)).await;
        match res {
            Ok(r) => r,
            Err(join_err) => err_result(crate::domain::error::ByohError::Other(format!(
                "rag_search task join failed: {join_err}"
            ))),
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
        let res = tokio::task::spawn_blocking(move || compile_dry_run_sync(&p.slug)).await;
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
        match crate::application::evolve_one_cycle(
            &crate::store::byoh_home(),
            &p.slug,
            genre,
            edit,
            metric,
        ) {
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
        let res = tokio::task::spawn_blocking(move || render_plugin_blocking(&p)).await;
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
        let res = tokio::task::spawn_blocking(move || install_plugin_blocking(&p)).await;
        match res {
            Ok(r) => r,
            Err(join_err) => err_result(crate::domain::error::ByohError::Other(format!(
                "install task join failed: {join_err}"
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
             → rag_index/rag_search → compile → compile_dry_run → \
             registry_clone_skill. The conversation IS the interview/wizard."
                .into(),
        );
        info
    }
}

// ─── blocking bodies for the heavy tools (run via spawn_blocking) ───────────

/// Synchronous body of `render_plugin`. Synthesizes the bundle then renders it
/// to the target host(s), writing a deployable plugin tree at `out`.
fn render_plugin_blocking(p: &RenderPluginParams) -> CallToolResult {
    let target: crate::domain::render_target::Target = match p.target.parse() {
        Ok(t) => t,
        Err(e) => return err_result(e),
    };
    let profile = match crate::store::load_profile(&p.slug) {
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
fn install_plugin_blocking(p: &InstallPluginParams) -> CallToolResult {
    let target: crate::domain::render_target::Target = match p.target.parse() {
        Ok(t) => t,
        Err(e) => return err_result(e),
    };
    let profile = match crate::store::load_profile(&p.slug) {
        Ok(pr) => pr,
        Err(e) => return err_result(e),
    };
    let (bundle, _plan) = match crate::application::synthesize(&profile) {
        Ok(b) => b,
        Err(e) => return err_result(e),
    };
    let loc = crate::deploy::InstallLocations::from_env();
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
            Ok(()) => ok_value(serde_json::to_value(&profile).unwrap_or(Value::Null)),
            Err(e) => err_result(e),
        },
        Err(e) => err_result(e),
    }
}

/// Synchronous body of `rag_index`.
fn rag_index_blocking(home: &std::path::Path, p: &RagIndexParams) -> CallToolResult {
    let genre = match parse_genre(&p.genre) {
        Ok(g) => g,
        Err(e) => return err_result(e),
    };
    match rag_index_impl(home, genre, &p.corpus, p.max_tokens, p.overlap) {
        Ok(report) => ok_value(json!({
            "docs": report.docs,
            "chunks": report.chunks,
            "dim": report.dim,
            "backend": report.backend,
        })),
        Err(e) => err_result(e),
    }
}

/// Synchronous body of `rag_search`.
fn rag_search_blocking(p: &RagSearchParams) -> CallToolResult {
    let genre = match parse_genre(&p.genre) {
        Ok(g) => g,
        Err(e) => return err_result(e),
    };
    match rag_search_impl(genre, &p.query, p.corpus.as_deref(), p.k) {
        Ok(hits) => {
            let masked: Vec<Value> = hits
                .into_iter()
                .map(|h| {
                    json!({
                        "id": h.id,
                        "score": h.score,
                        "mode": h.mode,
                        "text": crate::security::mask(&h.text),
                    })
                })
                .collect();
            ok_value(json!(masked))
        }
        Err(e) => err_result(e),
    }
}

/// Synchronous body of `compile_dry_run`.
fn compile_dry_run_sync(slug: &str) -> CallToolResult {
    let profile = match crate::store::load_profile(slug) {
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

// ─── rag helpers (feature-aware) ────────────────────────────────────────────

#[cfg(feature = "native-rag")]
fn rag_index_impl(
    home: &std::path::Path,
    genre: Genre,
    corpus: &str,
    max_tokens: usize,
    overlap: usize,
) -> crate::Result<crate::rag::BuildReport> {
    let docs = crate::store::collect_corpus(std::path::Path::new(corpus))?;
    let opts = crate::rag::ChunkOptions::new(max_tokens, overlap);
    let embedder = crate::store::make_embedder_native()?;
    let (report, handle) =
        crate::rag::pipeline::native::build_index_native(&*embedder, genre, &docs, &opts, 4)?;
    crate::rag::pipeline::native::save_index_native(&handle, home)?;
    Ok(report)
}

#[cfg(not(feature = "native-rag"))]
fn rag_index_impl(
    home: &std::path::Path,
    genre: Genre,
    corpus: &str,
    max_tokens: usize,
    overlap: usize,
) -> crate::Result<crate::rag::BuildReport> {
    let docs = crate::store::collect_corpus(std::path::Path::new(corpus))?;
    let opts = crate::rag::ChunkOptions::new(max_tokens, overlap);
    let embedder = crate::store::make_embedder()?;
    // Incremental: re-embed only changed/added docs against the persisted index.
    let (report, _delta) =
        crate::rag::build_index_incremental(&*embedder, home, genre, &docs, &opts)?;
    Ok(report)
}

/// Returns `HybridHit`s. Builds an ephemeral index from a corpus when given,
/// otherwise runs the grep-only tier against an empty corpus.
fn rag_search_impl(
    genre: Genre,
    query: &str,
    corpus: Option<&str>,
    k: usize,
) -> crate::Result<Vec<crate::rag::SearchHit>> {
    let embedder = crate::store::make_embedder()?;
    if let Some(corpus_path) = corpus {
        let docs = crate::store::collect_corpus(std::path::Path::new(corpus_path))?;
        let opts = crate::rag::ChunkOptions::default();
        let (_report, handle) = crate::rag::build_index(&*embedder, genre, &docs, &opts)?;
        return handle.search(&*embedder, query, k);
    }
    // No corpus supplied: reuse a previously-persisted index if one exists
    // (the "persistent knowledge base" — no re-embedding needed).
    if let Some(handle) = crate::rag::load_index(&crate::store::byoh_home(), genre)? {
        return handle.search(&*embedder, query, k);
    }
    // Otherwise: grep-only tier against an empty corpus via hybrid_search.
    let empty: Vec<(String, String)> = Vec::new();
    let qe = embedder.embed(query)?;
    let hits = crate::rag::hybrid_search(None, Some(&qe), &empty, query, k, genre);
    Ok(hits
        .into_iter()
        .map(|h| crate::rag::SearchHit {
            id: h.id,
            text: h.text,
            score: h.score,
            mode: h.mode.as_str(),
        })
        .collect())
}

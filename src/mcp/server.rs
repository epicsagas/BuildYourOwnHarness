//! The BYOH stdio MCP server (`byoh serve`).
//!
//! Wraps BYOH's synchronous lib APIs as MCP tools so an LLM agent can drive the
//! whole profile → build → install flow. The `#[tool_router(server_handler)]`
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

use crate::adapters::{FilesystemSource, RuleInterview, RuleLlm, StdCommand};
use crate::application::ProfileOrchestrator;
use crate::compiler::{dry_run, is_skeleton_body, static_gate};
use crate::domain::error::ByohError;
use crate::domain::genre::Genre;
use crate::domain::profile::{ProfileStatus, UserProfile};

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
fn orchestrator() -> (FilesystemSource, RuleLlm, RuleInterview) {
    (
        FilesystemSource::new(),
        RuleLlm::new(),
        RuleInterview::new(),
    )
}

/// Guard: the compile/render/install surface requires a Confirmed profile —
/// the same state-machine rule the CLI enforces. Without this, an agent could
/// compile and install a Draft profile and the 4-state machine would be
/// decorative at the API boundary agents actually use.
fn require_confirmed(profile: &UserProfile) -> Result<(), ByohError> {
    if profile.status != ProfileStatus::Confirmed {
        return Err(ByohError::ValidationGateFailed {
            gate: "profile_status",
            reason: format!(
                "profile '{}' is {:?}, not Confirmed — call profile_confirm first",
                profile.slug, profile.status
            ),
        });
    }
    Ok(())
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
        "pipeline_ok": r.pipeline_ok,
        "skills_ok": r.skills_ok,
        "agents_ok": r.agents_ok,
        "tools_list_ok": r.tools_list_ok,
        "errors": r.errors,
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
        tool_result!(crate::store::load_profile_in(&self.ctx.home, &p.slug))
    }

    #[tool(
        description = "Create a draft profile (and optionally autoscan paths). Returns the created profile."
    )]
    pub fn profile_create(&self, Parameters(p): Parameters<ProfileCreateParams>) -> CallToolResult {
        let lang = p.language.as_deref().unwrap_or("ko");
        let mut profile = UserProfile::new_draft(&p.slug, lang);
        if !p.scan_paths.is_empty() {
            let (src, llm, iv) = orchestrator();
            let orch = ProfileOrchestrator::new(&src, &llm, &iv);
            let path_refs: Vec<&std::path::Path> =
                p.scan_paths.iter().map(|s| s.as_str().as_ref()).collect();
            if let Err(e) = orch.stage1_scan(&mut profile, &path_refs) {
                return err_result(e);
            }
        }
        match crate::store::write_profile_in(&self.ctx.home, &profile) {
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
        // Move owned data into the blocking task (no borrowed refs). The home is
        // captured on THIS thread so the worker never consults the thread-local
        // override it cannot see.
        let home = self.ctx.home.clone();
        let res =
            tokio::task::spawn_blocking(move || profile_scan_sync(&home, &p.slug, &p.paths)).await;
        match res {
            Ok(r) => r,
            Err(join_err) => err_result(crate::domain::error::ByohError::Other(format!(
                "scan task join failed: {join_err}"
            ))),
        }
    }

    #[tool(
        description = "Run the S2 interview step. Only explicit answers are applied; unanswered questions stay open for the next call (ask the user, then call again). Returns the updated status + any surfaced council questions."
    )]
    pub fn profile_interview(
        &self,
        Parameters(p): Parameters<ProfileInterviewParams>,
    ) -> CallToolResult {
        let mut profile = match crate::store::load_profile_in(&self.ctx.home, &p.slug) {
            Ok(p) => p,
            Err(e) => return err_result(e),
        };
        let (src, llm, iv) = orchestrator();
        let orch = ProfileOrchestrator::new(&src, &llm, &iv);
        match orch.stage2_interview(&mut profile, &p.answers) {
            Ok(council) => {
                // A failed persist must be an error, not a silent success —
                // otherwise the next profile_confirm operates on stale state.
                if let Err(e) = crate::store::write_profile_in(&self.ctx.home, &profile) {
                    return err_result(e);
                }
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
        let mut profile = match crate::store::load_profile_in(&self.ctx.home, &p.slug) {
            Ok(p) => p,
            Err(e) => return err_result(e),
        };
        // Allow the minimal create → confirm path (Draft → Interviewed is a
        // formality when the caller already knows genre/goal), matching the CLI.
        if profile.status == ProfileStatus::Draft {
            if let Err(e) = profile.advance(ProfileStatus::Interviewed) {
                return err_result(e);
            }
        }
        let (src, llm, iv) = orchestrator();
        let orch = ProfileOrchestrator::new(&src, &llm, &iv);
        match orch.stage3_confirm(&mut profile, genre, p.goal_30d.as_deref()) {
            Ok(()) => match crate::store::write_profile_in(&self.ctx.home, &profile) {
                Ok(()) => ok_value(compact_status(&profile)),
                Err(e) => err_result(e),
            },
            Err(e) => err_result(e),
        }
    }

    #[tool(
        description = "Build a harness bundle from a confirmed profile: synthesize (compile + preset injection + static gate), optionally dry-run. Returns the bundle, synthesis plan, gate status, and matched vs skeleton skill classification. The agent decides whether to install or iterate the profile first based on this. Runs off the async runtime via spawn_blocking (dry-run shells out to dependency tools)."
    )]
    pub async fn build(&self, Parameters(p): Parameters<BuildParams>) -> CallToolResult {
        let home = self.ctx.home.clone();
        let res = tokio::task::spawn_blocking(move || build_sync(&home, &p)).await;
        match res {
            Ok(r) => r,
            Err(join_err) => err_result(ByohError::Other(format!(
                "build task join failed: {join_err}"
            ))),
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

    #[tool(
        description = "Author (or replace) a skill body via the LLM-authored overlay. The body \
                       persists across rebuilds — the next `build` reads it and replaces the \
                       skeleton. Safety-gate skill ids (critic/seesaw/stagnation) are refused. \
                       `body_markdown` is the full SKILL.md content (frontmatter + 4-section \
                       Process/Anti-Rationalization/Evidence/Red Flags body). Masked on write."
    )]
    pub async fn author_skill(
        &self,
        Parameters(p): Parameters<AuthorSkillParams>,
    ) -> CallToolResult {
        let home = self.ctx.home.clone();
        let res = tokio::task::spawn_blocking(move || author_skill_blocking(&home, &p)).await;
        match res {
            Ok(r) => r,
            Err(join_err) => err_result(crate::domain::error::ByohError::Other(format!(
                "author_skill task join failed: {join_err}"
            ))),
        }
    }

    #[tool(
        description = "Author (or replace) a doc (README / getting-started / AGENTS) for a \
                       profile via the overlay. `language` sets the file suffix \
                       (README.en.md, getting-started.ko.md). The renderer prefers an authored \
                       doc over the Rust skeleton. Masked on write."
    )]
    pub async fn author_doc(&self, Parameters(p): Parameters<AuthorDocParams>) -> CallToolResult {
        let home = self.ctx.home.clone();
        let res = tokio::task::spawn_blocking(move || author_doc_blocking(&home, &p)).await;
        match res {
            Ok(r) => r,
            Err(join_err) => err_result(crate::domain::error::ByohError::Other(format!(
                "author_doc task join failed: {join_err}"
            ))),
        }
    }

    #[tool(
        description = "List the skill/agent/doc overrides currently authored for a profile. \
                       Read-only — use to see what the next `build` will inject."
    )]
    pub fn list_overrides(&self, Parameters(p): Parameters<ListOverridesParams>) -> CallToolResult {
        list_overrides_sync(&self.ctx.home, &p)
    }

    #[tool(
        description = "Delete one authored override (kind: skill | agent | doc). The next `build` \
                       reverts the affected skill/doc to its preset body or skeleton."
    )]
    pub async fn delete_override(
        &self,
        Parameters(p): Parameters<DeleteOverrideParams>,
    ) -> CallToolResult {
        let home = self.ctx.home.clone();
        let res = tokio::task::spawn_blocking(move || delete_override_blocking(&home, &p)).await;
        match res {
            Ok(r) => r,
            Err(join_err) => err_result(crate::domain::error::ByohError::Other(format!(
                "delete_override task join failed: {join_err}"
            ))),
        }
    }

    #[tool(
        description = "Enable a curated hook template (`registry/hooks/<id>.toml`) for a profile. \
                       Writes a POINTER (the hook id) to the profile overlay — never a command. \
                       The template supplies the declarative `spec:<id>` reference and seeds \
                       HOOK_REQUIRED_FIELDS so the static gate passes. Refuses any hook_id not \
                       in the curated set (no arbitrary commands). Hooks stay declarative and \
                       are NOT wired into the rendered plugin."
    )]
    pub async fn enable_hook(&self, Parameters(p): Parameters<EnableHookParams>) -> CallToolResult {
        let home = self.ctx.home.clone();
        let res = tokio::task::spawn_blocking(move || enable_hook_blocking(&home, &p)).await;
        match res {
            Ok(r) => r,
            Err(join_err) => err_result(crate::domain::error::ByohError::Other(format!(
                "enable_hook task join failed: {join_err}"
            ))),
        }
    }
}

#[tool_handler]
impl ServerHandler for ByohServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        // `#[tool_handler]` only auto-generates `get_info()` (with
        // `capabilities.tools` enabled) when the impl doesn't already define
        // one — defining our own for custom instructions means we must
        // declare `enable_tools()` ourselves, or clients see empty
        // capabilities and never call `tools/list`.
        rmcp::model::ServerInfo::new(
            rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_server_info(rmcp::model::Implementation::new(
            "byoh",
            env!("CARGO_PKG_VERSION"),
        ))
        .with_instructions(
            "BYOH: build a personalized AI agent harness. Drive the flow: \
                 profile_create → profile_scan → profile_interview → profile_confirm \
                 → build → install_plugin. build returns matched_skills (real preset \
                 bodies) vs skeleton_skills (genre-template placeholders); you decide \
                 whether to install now or iterate the profile first. The conversation \
                 IS the interview/wizard.",
        )
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
    if let Err(e) = require_confirmed(&profile) {
        return err_result(e);
    }
    let (bundle, _plan) = match crate::application::synthesize(&profile) {
        Ok(b) => b,
        Err(e) => return err_result(e),
    };
    // Apply LLM-authored overlays so rendered output carries authored content.
    let mut bundle = bundle;
    if let Err(e) =
        crate::application::overrides::apply_profile_overrides(home, &p.slug, &mut bundle)
    {
        return err_result(ByohError::Other(format!("override apply failed: {e}")));
    }
    match crate::application::render_target(&bundle, target, std::path::Path::new(&p.out), home) {
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
    if let Err(e) = require_confirmed(&profile) {
        return err_result(e);
    }
    let (bundle, _plan) = match crate::application::synthesize(&profile) {
        Ok(b) => b,
        Err(e) => return err_result(e),
    };
    // Apply LLM-authored overlays so installed output carries authored content.
    let mut bundle = bundle;
    if let Err(e) =
        crate::application::overrides::apply_profile_overrides(home, &p.slug, &mut bundle)
    {
        return err_result(ByohError::Other(format!("override apply failed: {e}")));
    }
    let scope = match crate::deploy::resolve_scope(p.scope.clone(), p.host) {
        Ok(s) => s,
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
        "scope": scope.as_str(),
    });
    match scope {
        crate::domain::scope::Scope::Publish => {
            if let Err(e) = crate::application::render_plugin::write_publish_extras(&path) {
                return err_result(e);
            }
            result["publish"] = json!({
                "license_added": true,
                "gitignore_added": true,
                "next_steps": format!(
                    "cd {} && git init && git add -A && git commit -m 'publish byoh-{}' && git remote add origin <url> && git push -u origin main",
                    path.display(), p.slug
                ),
            });
        }
        crate::domain::scope::Scope::DistOnly => {
            result["activated"] = json!(false);
        }
        crate::domain::scope::Scope::Global | crate::domain::scope::Scope::Local => {
            // Local: point Claude at the project-local .claude/ instead of HOME.
            let loc_act = if matches!(scope, crate::domain::scope::Scope::Local) {
                let local_claude = std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .join(".claude");
                loc.with_claude_config(local_claude)
            } else {
                loc.clone()
            };
            let commands = crate::adapters::StdCommand::new();
            let mut activations = Vec::new();
            for t in target.concrete() {
                // Local scope: codex/agy have no project-local mode — skip them.
                if matches!(scope, crate::domain::scope::Scope::Local)
                    && !matches!(t, crate::domain::render_target::Target::Claude)
                {
                    activations.push(json!({
                        "host": t.as_str(),
                        "status": "Skipped",
                        "reason": "this host's CLI has no project-local plugin scope (HOME only)",
                        "alternatives": [
                            "re-run install_plugin with scope=global",
                            format!("point the {} CLI at the dist tree directly: {}", t.as_str(), path.display()),
                        ],
                    }));
                    continue;
                }
                let entry =
                    match crate::deploy::activate_plugin(*t, &path, &p.slug, &loc_act, &commands) {
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
        }
    }
    ok_value(result)
}

/// Synchronous body of `profile_scan`. Owned inputs so it can move into a
/// `spawn_blocking` task; `home` is resolved by the caller on the runtime
/// thread (worker threads can't see the thread-local home override).
fn profile_scan_sync(home: &std::path::Path, slug: &str, paths: &[String]) -> CallToolResult {
    let mut profile = match crate::store::load_profile_in(home, slug) {
        Ok(p) => p,
        Err(e) => return err_result(e),
    };
    let (src, llm, iv) = orchestrator();
    let orch = ProfileOrchestrator::new(&src, &llm, &iv);
    let path_refs: Vec<&std::path::Path> = paths.iter().map(|s| s.as_str().as_ref()).collect();
    match orch.stage1_scan(&mut profile, &path_refs) {
        Ok(()) => match crate::store::write_profile_in(home, &profile) {
            Ok(()) => ok_value(compact_status(&profile)),
            Err(e) => err_result(e),
        },
        Err(e) => err_result(e),
    }
}

/// Synchronous body of `build`.
///
/// Synthesizes the bundle (compile + preset injection; `synthesize` re-runs the
/// static gate internally at synthesis.rs), classifies which skills got real
/// preset bodies vs. which are still genre-template skeletons, and optionally
/// runs the dry-run gate. The agent reads `matched_skills` / `skeleton_skills`
/// to decide whether to `install_plugin` now or iterate the profile first.
fn build_sync(home: &std::path::Path, p: &BuildParams) -> CallToolResult {
    let profile = match crate::store::load_profile_in(home, &p.slug) {
        Ok(p) => p,
        Err(e) => return err_result(e),
    };
    if let Err(e) = require_confirmed(&profile) {
        return err_result(e);
    }
    let (bundle, plan) = match crate::application::synthesize(&profile) {
        Ok(b) => b,
        Err(e) => return err_result(e),
    };
    // Apply LLM-authored overlays (defect-3 fix): authored skills/docs persist
    // across rebuilds and replace skeleton bodies here, before classification.
    let mut bundle = bundle;
    let override_report =
        match crate::application::overrides::apply_profile_overrides(home, &p.slug, &mut bundle) {
            Ok(r) => r,
            Err(e) => return err_result(ByohError::Other(format!("override apply failed: {e}"))),
        };
    // `synthesize` already ran static_gate; recompute the report for the JSON.
    let static_report = match static_gate(&bundle) {
        Ok(r) => r,
        Err(e) => return err_result(e),
    };

    // Skills the synthesis plan injected real preset bodies into.
    let matched_skills: Vec<&str> = plan
        .pipelines
        .iter()
        .flat_map(|pp| pp.steps.iter())
        .map(|s| s.skill_id.as_str())
        .collect();
    // Skills still carrying the genre-template placeholder body.
    let skeleton_skills: Vec<&str> = bundle
        .skills
        .iter()
        .filter(|s| is_skeleton_body(&s.body_markdown))
        .map(|s| s.id.as_str())
        .collect();

    let mut result = json!({
        "bundle": serde_json::to_value(&bundle).unwrap_or(Value::Null),
        "synthesis_plan": serde_json::to_value(&plan).unwrap_or(Value::Null),
        "static_gate_passed": static_report.passed(),
        "static_gate": static_gate_json(&static_report),
        "matched_skills": matched_skills,
        "authored_skills": override_report.authored_skills,
        "authored_docs": override_report.authored_docs,
        "enabled_hooks": override_report.enabled_hooks,
        "skeleton_skills": skeleton_skills,
        "override_collisions": override_report.collisions,
        "override_refused": override_report.refused,
    });

    if p.run_dry_run {
        // StdCommand shells out to dependency tools; missing tools are a graceful
        // fallback inside dry_run, surfaced in the report.
        let cmds = StdCommand::new();
        match dry_run(&bundle, &cmds) {
            Ok(dry_report) => {
                result["dry_run_passed"] = json!(dry_report.passed());
                result["dry_run"] = dry_run_json(&dry_report);
            }
            Err(e) => return err_result(e),
        }
    }
    ok_value(result)
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

/// Synchronous body of `author_skill`. Atomically writes the masked body to
/// `<home>/profiles/<slug>/overrides/skills/<id>.md` (temp + rename). Refuses
/// safety-gate ids.
fn author_skill_blocking(home: &std::path::Path, p: &AuthorSkillParams) -> CallToolResult {
    if !crate::application::overrides::is_overridable_skill(&p.skill_id) {
        return err_result(crate::domain::error::ByohError::Schema(format!(
            "skill '{}' is a safety gate and cannot be overridden",
            p.skill_id
        )));
    }
    let dir = match crate::application::overrides::overrides_dir(home, &p.slug) {
        Ok(d) => d.join("skills"),
        Err(e) => {
            return err_result(crate::domain::error::ByohError::Other(format!(
                "override dir: {e}"
            )));
        }
    };
    let masked = crate::security::mask(&p.body_markdown);
    let name = format!("{}.md", p.skill_id);
    match atomic_write(&dir, &name, &masked) {
        Ok(()) => ok_value(json!({
            "authored": "skill",
            "skill_id": p.skill_id,
            "slug": p.slug,
            "persisted": true,
        })),
        Err(e) => err_result(crate::domain::error::ByohError::Other(format!(
            "write override: {e}"
        ))),
    }
}

/// Synchronous body of `author_doc`. Atomically writes the masked doc body to
/// `<home>/profiles/<slug>/overrides/docs/<id>.<lang>.md`.
fn author_doc_blocking(home: &std::path::Path, p: &AuthorDocParams) -> CallToolResult {
    let doc_id = match p.doc_id.as_str() {
        "README" | "getting-started" | "AGENTS" => p.doc_id.as_str(),
        other => {
            return err_result(crate::domain::error::ByohError::Schema(format!(
                "unknown doc_id '{other}' (expected README | getting-started | AGENTS)"
            )));
        }
    };
    let dir = match crate::application::overrides::overrides_dir(home, &p.slug) {
        Ok(d) => d.join("docs"),
        Err(e) => {
            return err_result(crate::domain::error::ByohError::Other(format!(
                "override dir: {e}"
            )));
        }
    };
    let masked = crate::security::mask(&p.body_markdown);
    let name = format!("{doc_id}.{}.md", p.language);
    match atomic_write(&dir, &name, &masked) {
        Ok(()) => ok_value(json!({
            "authored": "doc",
            "doc_id": doc_id,
            "language": p.language,
            "slug": p.slug,
            "persisted": true,
        })),
        Err(e) => err_result(crate::domain::error::ByohError::Other(format!(
            "write override: {e}"
        ))),
    }
}

/// Synchronous body of `list_overrides`. Walks the override tree.
fn list_overrides_sync(home: &std::path::Path, p: &ListOverridesParams) -> CallToolResult {
    let root = match crate::application::overrides::overrides_dir(home, &p.slug) {
        Ok(d) => d,
        Err(e) => {
            return err_result(crate::domain::error::ByohError::Other(format!(
                "override dir: {e}"
            )));
        }
    };
    if !root.exists() {
        return ok_value(json!({ "skills": [], "agents": [], "docs": [], "hooks": [] }));
    }
    // Collect file stems regardless of extension (skills/agents/docs are .md,
    // hooks are .toml) so one helper covers every overlay kind.
    let collect = |sub: &str| -> Vec<String> {
        let d = root.join(sub);
        if !d.is_dir() {
            return vec![];
        }
        std::fs::read_dir(&d)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter_map(|e| e.path().file_stem()?.to_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    };
    ok_value(json!({
        "skills": collect("skills"),
        "agents": collect("agents"),
        "docs": collect("docs"),
        "hooks": collect("hooks"),
    }))
}

/// Synchronous body of `delete_override`. Removes one overlay file.
fn delete_override_blocking(home: &std::path::Path, p: &DeleteOverrideParams) -> CallToolResult {
    let (sub, ext) = match p.kind.as_str() {
        "skill" => ("skills", "md"),
        "agent" => ("agents", "md"),
        "doc" => ("docs", "md"),
        "hook" => ("hooks", "toml"),
        other => {
            return err_result(crate::domain::error::ByohError::Schema(format!(
                "unknown kind '{other}' (expected skill | agent | doc | hook)"
            )));
        }
    };
    let root = match crate::application::overrides::overrides_dir(home, &p.slug) {
        Ok(d) => d,
        Err(e) => {
            return err_result(crate::domain::error::ByohError::Other(format!(
                "override dir: {e}"
            )));
        }
    };
    let path = root.join(sub).join(format!("{}.{}", p.id, ext));
    if !path.exists() {
        return ok_value(
            json!({ "deleted": false, "reason": "not_found", "path": path.to_string_lossy() }),
        );
    }
    match std::fs::remove_file(&path) {
        Ok(()) => ok_value(json!({ "deleted": true, "kind": sub, "id": p.id })),
        Err(e) => err_result(crate::domain::error::ByohError::Other(format!(
            "remove override: {e}"
        ))),
    }
}

/// Synchronous body of `enable_hook`. Validates the hook_id against the curated
/// `registry/hooks/` set (refuses unknown ids), then writes a pointer TOML to
/// the profile overlay. The pointer carries only the id; the template supplies
/// the declarative command + required reads at apply time.
fn enable_hook_blocking(home: &std::path::Path, p: &EnableHookParams) -> CallToolResult {
    // Validate against the curated set BEFORE writing anything: refuse any id
    // not backed by a registry/hooks/<id>.toml template. No arbitrary command
    // ever enters the overlay from this tool.
    if let Err(reason) = crate::application::overrides::load_hook_template(&p.hook_id) {
        return err_result(crate::domain::error::ByohError::Schema(format!(
            "hook '{}' is not in the curated registry/hooks set: {reason}",
            p.hook_id
        )));
    }
    let dir = match crate::application::overrides::overrides_dir(home, &p.slug) {
        Ok(d) => d.join("hooks"),
        Err(e) => {
            return err_result(crate::domain::error::ByohError::Other(format!(
                "override dir: {e}"
            )));
        }
    };
    let body = format!("hook_id = \"{}\"\n", p.hook_id);
    let name = format!("{}.toml", p.hook_id);
    match atomic_write(&dir, &name, &body) {
        Ok(()) => ok_value(json!({
            "enabled": true,
            "hook_id": p.hook_id,
            "slug": p.slug,
            "note": "declarative spec pointer; the static gate enforces HOOK_REQUIRED_FIELDS at build time",
        })),
        Err(e) => err_result(crate::domain::error::ByohError::Other(format!(
            "write hook pointer: {e}"
        ))),
    }
}

/// Create `dir` then atomically write `name` via temp-file + rename, so
/// concurrent `author_skill` calls can't interleave a partial body.
fn atomic_write(dir: &std::path::Path, name: &str, content: &str) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let final_path = dir.join(name);
    let tmp = dir.join(format!(".{name}.tmp"));
    std::fs::write(&tmp, content).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &final_path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_info_declares_tools_capability() {
        // Regression: overriding `get_info()` for custom instructions makes
        // `#[tool_handler]` skip its own auto-generated version (which would
        // have enabled `capabilities.tools`), so without an explicit
        // `enable_tools()` call, clients see empty capabilities and never
        // call `tools/list` even though it works fine when called directly.
        let ctx = ByohContext {
            home: PathBuf::from("/tmp/byoh-test-home"),
            language: "en".to_string(),
        };
        let server = ByohServer::new(ctx);
        let info = server.get_info();
        assert!(
            info.capabilities.tools.is_some(),
            "get_info() must declare capabilities.tools so clients call tools/list"
        );
    }
}

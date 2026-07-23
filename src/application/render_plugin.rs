//! Target renderer — emit a self-contained plugin tree for a host tool.
//!
//! Takes a [`HarnessBundle`] (built by `compile_profile` / `synthesize`) and
//! writes a deployable plugin into `out`. The output dir is `git init`-ready:
//! a user can push it and anyone who clones gets a working plugin.
//!
//! The rendered plugin is deliberately **static** — skills, agents, manifests,
//! docs. Bundle-declared hooks and MCP tools are an internal spec and are NOT
//! wired into the plugin: they would require a `byoh` binary (plus this
//! machine's profile) on every consumer's machine, which turns into a dead MCP
//! server / failing hooks the moment the plugin leaves this repo. Static
//! content works in every host with zero runtime dependencies.
//!
//! Claude/Codex formats are grounded in real reference projects the author
//! has built (self-evolving harnesses, retrieval services, storage engines,
//! Obsidian tooling), not synthetic examples. The agy layout follows the
//! official Antigravity CLI plugin spec.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::Result;
use crate::domain::bundle::{AgentSpec, HarnessBundle, SkillSpec};
use crate::domain::render_target::Target;

/// Embedded Apache-2.0 text — what every BYOH-published plugin ships under.
/// `include_str!` bakes it into the binary at compile time, so a `cargo install
/// --git` binary carries its own LICENSE with no runtime file dependency.
const LICENSE_TEXT: &str = include_str!("../../LICENSE");

/// Minimal `.gitignore` for a published plugin tree.
const GITIGNORE_TEXT: &str = "\
# BYOH-generated plugin — build/runtime noise\n\
.byoh-manifest\n\
*.log\n\
.DS_Store\n\
";

/// Add the publish-only files (`LICENSE` + `.gitignore`) to an already-rendered
/// plugin tree. Called by `install`/`install_plugin` only when the scope is
/// `Publish`; the base render is unchanged for the other scopes.
pub fn write_publish_extras(out: &Path) -> Result<()> {
    crate::store::write_file(out, "LICENSE", LICENSE_TEXT)?;
    crate::store::write_file(out, ".gitignore", GITIGNORE_TEXT)?;
    Ok(())
}

/// Render `bundle` for `target` into `out`. For `Target::All`, writes a single
/// **polyglot** tree carrying all three hosts' manifests (`.claude-plugin/`,
/// `.codex-plugin/`, root agy `plugin.json`) plus shared `skills/`/`agents/`.
/// A single concrete target renders a host-only tree.
///
/// `home` resolves the profile's overlay directory: when the LLM has authored a
/// doc override (README / getting-started / AGENTS), the rendered skeleton doc
/// is replaced by the override body. Pass the resolved `byoh_home()`.
///
/// Refuses to write into an existing non-empty directory that is not
/// BYOH-owned (no `.byoh-manifest`) — an agent calling `render_plugin` with
/// `out: "."` must not silently clobber a real project's README/plugin.json.
pub fn render_target(
    bundle: &HarnessBundle,
    target: Target,
    out: &Path,
    home: &Path,
) -> Result<PathBuf> {
    guard_output_dir(out)?;
    if target == Target::All {
        render_polyglot(bundle, out)?;
        write_readme(bundle, out, Target::All, home)?;
        write_docs_guide(bundle, out, home)?;
    } else {
        render_one(bundle, target, out)?;
        write_readme(bundle, out, target, home)?;
    }
    // Mark the tree BYOH-owned so re-renders and installs recognize it.
    crate::deploy::install::write_owned_marker(out, &format!("byoh-{}", bundle.slug))?;
    Ok(out.to_path_buf())
}

/// Overwrite guard shared by all render paths.
fn guard_output_dir(out: &Path) -> Result<()> {
    let occupied = out.exists()
        && std::fs::read_dir(out)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false);
    if occupied && !crate::deploy::install::is_byoh_owned(out) {
        return Err(crate::domain::ByohError::ValidationGateFailed {
            gate: "render_output",
            reason: format!(
                "refusing to render into non-empty, non-BYOH directory {} — pass a fresh output dir",
                out.display()
            ),
        });
    }
    Ok(())
}

fn render_one(bundle: &HarnessBundle, target: Target, out: &Path) -> Result<()> {
    match target {
        Target::Claude => render_claude(bundle, out),
        Target::Codex => render_codex(bundle, out),
        Target::Agy => render_agy(bundle, out),
        Target::All => unreachable!("All is expanded by render_target"),
    }
}

// ─── polyglot (Target::All) ─────────────────────────────────────────────────
//
// One directory, all three hosts' manifests + de-duplicated shared skills/
// agents. The merge is conflict-free: the only overlapping root paths
// (skills/, agents/*.md, AGENTS.md) come from host-agnostic helpers, so they
// are written once. Each host reads its own manifest from this single tree;
// at install time agy/codex copy it into their own root and claude links it.

/// Render a single polyglot plugin tree into `out` (the Velith shape): root agy
/// `plugin.json` + `.claude-plugin/` + `.codex-plugin/` + shared `skills/` and
/// `agents/`, plus per-host hooks/MCP. See [`render_target`] for dispatch.
fn render_polyglot(bundle: &HarnessBundle, out: &Path) -> Result<()> {
    // agy marker (root, strict 3-key form).
    crate::store::write_file(out, "plugin.json", &pretty(&agy_manifest(bundle)))?;

    // Claude manifest + marketplace.json (so `claude plugin marketplace add
    // <this repo>` works the moment the tree is pushed to GitHub).
    crate::store::write_file(
        &out.join(".claude-plugin"),
        "plugin.json",
        &pretty(&claude_manifest(bundle)),
    )?;
    crate::store::write_file(
        &out.join(".claude-plugin"),
        "marketplace.json",
        &pretty(&claude_marketplace(bundle)),
    )?;

    // Codex manifest + TOML agents.
    crate::store::write_file(
        &out.join(".codex-plugin"),
        "plugin.json",
        &pretty(&codex_manifest(bundle)),
    )?;
    for agent in &bundle.agents {
        crate::store::write_file(
            &out.join(".codex-plugin").join("agents"),
            &format!("{}.toml", agent.id),
            &agent_toml(agent),
        )?;
    }

    // Shared skills + agents (written once; Claude/agy share the .md layout).
    write_skills(bundle, out)?;
    write_agents_md(bundle, out)?;

    // NOTE: bundle.hooks / bundle.mcp_tools are an internal spec and are NOT
    // rendered — see the module doc. A plugin that ships hook commands or an
    // MCP server pointing at a binary + profile the consumer doesn't have is
    // broken on arrival; the static tree works everywhere.

    // Codex .codex/ config + symlinks. Relative targets still resolve: the
    // polyglot root IS the codex root.
    crate::store::write_file(
        &out.join(".codex"),
        "config.toml",
        &codex_config_toml(bundle),
    )?;
    let codex = out.join(".codex");
    crate::store::create_symlink_or_copy(
        &PathBuf::from("../.codex-plugin/agents"),
        &codex.join("agents"),
    )?;
    crate::store::create_symlink_or_copy(&PathBuf::from("../skills"), &codex.join("skills"))?;

    // Shared root system prompt (once).
    crate::store::write_file(out, "AGENTS.md", &mask(&root_agents_md(bundle)))?;

    // Per-host discovery dirs (.claude/.gemini/.codex) linking shared skills/agents.
    write_host_symlinks(out)?;
    Ok(())
}

// ─── common builders ────────────────────────────────────────────────────────

/// Create the per-host discovery directories (`.claude/`, `.gemini/`) that link
/// the shared `skills/` and `agents/`. The single source of truth stays at the
/// root + `.{host}-plugin/`; hosts discover via their own dir. `.codex/`
/// already carries skills/agents links from [`render_polyglot`]. Uses
/// [`crate::store::create_symlink_or_copy`] (symlink on Unix, recursive copy
/// fallback elsewhere).
fn write_host_symlinks(out: &Path) -> Result<()> {
    for host in [".claude", ".gemini"] {
        let dir = out.join(host);
        crate::store::create_symlink_or_copy(&PathBuf::from("../skills"), &dir.join("skills"))?;
        crate::store::create_symlink_or_copy(&PathBuf::from("../agents"), &dir.join("agents"))?;
    }
    Ok(())
}

fn author_obj() -> Value {
    json!({ "name": "byoh", "url": "https://github.com/epicsagas/BuildYourOwnHarness" })
}

fn repo_url(slug: &str) -> String {
    format!("https://github.com/epicsagas/byoh-{slug}")
}

/// Plugin base metadata shared by both Claude and Codex manifests.
fn base_manifest(bundle: &HarnessBundle) -> Value {
    json!({
        "name": format!("byoh-{}", bundle.slug),
        "version": bundle.version.as_string(),
        "description": format!("BYOH-generated {} harness for '{}'.", bundle.genre.as_str(), bundle.slug),
        "author": author_obj(),
        "homepage": "https://github.com/epicsagas/BuildYourOwnHarness",
        "repository": repo_url(&bundle.slug),
        "license": "Apache-2.0",
        "keywords": ["byoh", "agent-harness", bundle.genre.as_str()],
    })
}

/// `.claude-plugin/plugin.json`: base manifest + skills + agents array (paths).
/// Adds `$schema` for editor/tooling parity with the reference plugins.
fn claude_manifest(bundle: &HarnessBundle) -> Value {
    let agent_paths: Vec<String> = bundle
        .agents
        .iter()
        .map(|a| format!("./agents/{}.md", a.id))
        .collect();
    let mut manifest = base_manifest(bundle);
    manifest["$schema"] = json!("https://json.schemastore.org/claude-code-plugin-manifest.json");
    manifest["skills"] = json!("./skills/");
    if !agent_paths.is_empty() {
        manifest["agents"] = json!(agent_paths);
    }
    manifest
}

/// `.claude-plugin/marketplace.json` — makes the rendered repo itself a
/// one-plugin marketplace. Without this file `claude plugin marketplace add
/// <repo>` rejects the repo outright, so "push it and others can install it"
/// would be false for the host we care most about.
fn claude_marketplace(bundle: &HarnessBundle) -> Value {
    let name = format!("byoh-{}", bundle.slug);
    json!({
        "$schema": "https://anthropic.com/claude-code/marketplace.schema.json",
        "name": name,
        "owner": author_obj(),
        "metadata": {
            "description": format!(
                "BYOH-generated {} harness for '{}'.",
                bundle.genre.as_str(),
                bundle.slug
            ),
        },
        "plugins": [{
            "name": name,
            "source": "./",
            "description": format!(
                "BYOH-generated {} harness for '{}'.",
                bundle.genre.as_str(),
                bundle.slug
            ),
        }],
    })
}

/// `.codex-plugin/plugin.json`: base manifest + skills + agents dir + interface.
fn codex_manifest(bundle: &HarnessBundle) -> Value {
    let mut manifest = base_manifest(bundle);
    manifest["skills"] = json!("./skills/");
    manifest["agents"] = json!("./.codex-plugin/agents/");
    manifest["interface"] = json!({
        "displayName": format!("BYOH {}", bundle.genre.as_str()),
        "shortDescription": manifest["description"].clone(),
        "developerName": "byoh",
        "category": "Development & Workflow",
        "capabilities": ["Read", "Write"],
    });
    manifest
}

/// Root agy `plugin.json` — STRICT (additionalProperties:false): only `$schema`,
/// `name`, `description`. agy rewrites this to a stub on install, so any extra
/// key is rejected.
fn agy_manifest(bundle: &HarnessBundle) -> Value {
    json!({
        "$schema": "https://antigravity.google/schemas/v1/plugin.json",
        "name": format!("byoh-{}", bundle.slug),
        "description": format!(
            "BYOH-generated {} harness for '{}'.",
            bundle.genre.as_str(),
            bundle.slug
        ),
    })
}

/// A SKILL.md body: YAML frontmatter (name + description) + the skill markdown.
fn skill_md(skill: &SkillSpec) -> String {
    format!(
        "---\nname: {id}\ndescription: {desc}\n---\n\n{body}\n",
        id = skill.id,
        desc = skill.description.replace('"', "\\\""),
        body = skill.body_markdown
    )
}

/// A Claude/agy agent .md: YAML frontmatter (name + description [+ tools]) + body.
fn agent_md(agent: &AgentSpec) -> String {
    let tools_line = match &agent.tools {
        Some(t) if !t.is_empty() => {
            let quoted: Vec<String> = t.iter().map(|x| format!("\"{x}\"")).collect();
            format!("\ntools: [{}]", quoted.join(", "))
        }
        _ => String::new(),
    };
    format!(
        "---\nname: {id}\ndescription: {desc}{tools}\n---\n\n{body}\n",
        id = agent.id,
        desc = agent.description.replace('"', "\\\""),
        tools = tools_line,
        body = agent.body_markdown
    )
}

/// A Codex agent .toml: exactly 3 keys. `tools` is dropped (verified — no Codex
/// agent TOML carries it). Body goes into a triple-quoted multi-line string.
fn agent_toml(agent: &AgentSpec) -> String {
    // Escape any `"""` inside the body (rare) to avoid breaking the TOML string.
    let body = agent.body_markdown.replace("\"\"\"", "\\\"\\\"\\\"");
    format!(
        "name = \"{id}\"\ndescription = \"{desc}\"\n\ndeveloper_instructions = \"\"\"\n{body}\n\"\"\"\n",
        id = agent.id,
        desc = agent.description.replace('\\', "\\\\").replace('"', "\\\""),
        body = body
    )
}

/// Secret masking over every profile-derived markdown artifact. Manifests are
/// built from fixed fields (sanitized slug + genre) and skip this; free-text
/// bodies (skills, agents, docs, AGENTS.md, README) go through `mask` so a
/// token pasted into a goal or vendored skill never lands on disk in a
/// published tree (R20).
fn mask(text: &str) -> String {
    crate::security::mask(text)
}

fn write_skills(bundle: &HarnessBundle, root: &Path) -> Result<()> {
    for skill in &bundle.skills {
        let dir = root.join("skills").join(&skill.id);
        crate::store::write_file(&dir, "SKILL.md", &mask(&skill_md(skill)))?;
    }
    Ok(())
}

/// Shared `agents/<id>.md` (Claude and agy share this format; Codex uses TOML).
fn write_agents_md(bundle: &HarnessBundle, out: &Path) -> Result<()> {
    for agent in &bundle.agents {
        crate::store::write_file(
            &out.join("agents"),
            &format!("{}.md", agent.id),
            &mask(&agent_md(agent)),
        )?;
    }
    Ok(())
}

// ─── Claude Code ────────────────────────────────────────────────────────────

fn render_claude(bundle: &HarnessBundle, out: &Path) -> Result<()> {
    // .claude-plugin/plugin.json — agents as array of file paths — plus the
    // marketplace.json that makes the pushed repo installable.
    crate::store::write_file(
        &out.join(".claude-plugin"),
        "plugin.json",
        &pretty(&claude_manifest(bundle)),
    )?;
    crate::store::write_file(
        &out.join(".claude-plugin"),
        "marketplace.json",
        &pretty(&claude_marketplace(bundle)),
    )?;

    // skills/<id>/SKILL.md + agents/<id>.md
    write_skills(bundle, out)?;
    write_agents_md(bundle, out)?;

    // AGENTS.md root system prompt. (No hooks/MCP — see module doc.)
    crate::store::write_file(out, "AGENTS.md", &mask(&root_agents_md(bundle)))?;
    Ok(())
}

// ─── Codex ──────────────────────────────────────────────────────────────────

fn render_codex(bundle: &HarnessBundle, out: &Path) -> Result<()> {
    // .codex-plugin/plugin.json — agents as directory path.
    crate::store::write_file(
        &out.join(".codex-plugin"),
        "plugin.json",
        &pretty(&codex_manifest(bundle)),
    )?;

    // .codex-plugin/agents/<id>.toml (3-key proprietary syntax).
    for agent in &bundle.agents {
        crate::store::write_file(
            &out.join(".codex-plugin").join("agents"),
            &format!("{}.toml", agent.id),
            &agent_toml(agent),
        )?;
    }

    // skills/ (shared with Claude layout at root).
    write_skills(bundle, out)?;

    // .codex/config.toml — instructions system prompt.
    crate::store::write_file(
        &out.join(".codex"),
        "config.toml",
        &codex_config_toml(bundle),
    )?;

    // .codex symlinks: agents → ../.codex-plugin/agents, skills → ../skills.
    let codex = out.join(".codex");
    crate::store::create_symlink_or_copy(
        &PathBuf::from("../.codex-plugin/agents"),
        &codex.join("agents"),
    )?;
    crate::store::create_symlink_or_copy(&PathBuf::from("../skills"), &codex.join("skills"))?;

    // AGENTS.md. (No hooks/MCP — see module doc.)
    crate::store::write_file(out, "AGENTS.md", &mask(&root_agents_md(bundle)))?;
    Ok(())
}

fn codex_config_toml(bundle: &HarnessBundle) -> String {
    format!(
        "# Codex config — generated by BYOH for '{slug}'.\n\
         # Codex loads this file plus the root AGENTS.md as system prompt.\n\n\
         instructions = \"\"\"\n\
         This is a BYOH-generated {genre} harness plugin. The root AGENTS.md is authoritative.\n\n\
         Drive the harness skills and agents defined under skills/ and .codex-plugin/agents/.\n\
         Conversation is the interface — do not shell out to a CLI when an agent/skill applies.\n\
         \"\"\"\n",
        slug = bundle.slug,
        genre = bundle.genre.as_str()
    )
}

// ─── agy (Antigravity) ──────────────────────────────────────────────────────
//
// Per the official Antigravity CLI plugin spec, a plugin directory contains:
//   plugin.json        (REQUIRED marker: only {name, description}+$schema —
//                        additionalProperties:false, so NO extra keys)
//   mcp_config.json     (optional MCP servers; remote uses `serverUrl`)
//   hooks.json          (optional pre/post tool hooks)
//   skills/             (optional <name>.md skills — frontmatter name+description)
//   agents/             (optional subagent templates)
//   rules/              (optional codebase rules)
// Installed under ~/.gemini/config/plugins/<plugin_name>/. We render
// that plugin directory into `out`.

fn render_agy(bundle: &HarnessBundle, out: &Path) -> Result<()> {
    // plugin.json — REQUIRED marker. Schema is strict (additionalProperties:false):
    // ONLY name + description (+ $schema). No version/author/etc.
    crate::store::write_file(out, "plugin.json", &pretty(&agy_manifest(bundle)))?;

    // skills/<id>/SKILL.md + agents/<id>.md subagent templates.
    write_skills(bundle, out)?;
    write_agents_md(bundle, out)?;

    // AGENTS.md root system prompt (still useful as a harness overview).
    // (No hooks/MCP — see module doc.)
    crate::store::write_file(out, "AGENTS.md", &mask(&root_agents_md(bundle)))?;
    Ok(())
}

// ─── shared docs ────────────────────────────────────────────────────────────

fn root_agents_md(bundle: &HarnessBundle) -> String {
    let skill_ids: Vec<&str> = bundle.skills.iter().map(|s| s.id.as_str()).collect();
    let agent_ids: Vec<&str> = bundle.agents.iter().map(|a| a.id.as_str()).collect();
    format!(
        "# BYOH Harness — {slug} ({genre})\n\n\
         Generated by [BYOH](https://github.com/epicsagas/BuildYourOwnHarness). \
         This plugin assembles a personalized agent harness from a user profile.\n\n\
         ## Skills\n{skills}\n\n## Agents\n{agents}\n\n\
         ## Safety\n\nThis harness enforces the 3 BYOH safety gates: {gates}.\n",
        slug = bundle.slug,
        genre = bundle.genre.as_str(),
        skills = if skill_ids.is_empty() {
            "_(none)_".into()
        } else {
            skill_ids
                .iter()
                .map(|s| format!("- `{s}`"))
                .collect::<Vec<_>>()
                .join("\n")
        },
        agents = if agent_ids.is_empty() {
            "_(none)_".into()
        } else {
            agent_ids
                .iter()
                .map(|a| format!("- `{a}`"))
                .collect::<Vec<_>>()
                .join("\n")
        },
        gates = bundle.safety_gates.join(", "),
    )
}

fn write_readme(bundle: &HarnessBundle, out: &Path, target: Target, home: &Path) -> Result<()> {
    // If the LLM authored a README override for this language, use it verbatim.
    // Otherwise emit the Rust skeleton (structural info is always correct here:
    // install commands, gate list, counts — only prose is a placeholder).
    let lang = bundle.language.as_str();
    if let Some(body) =
        crate::application::overrides::read_doc_override(home, &bundle.slug, "README", lang)
    {
        crate::store::write_file(out, "README.md", &body)?;
        return Ok(());
    }
    let body = match lang {
        "ko" => readme_ko(bundle, target),
        // English is the canonical fallback for every other / unknown language.
        _ => readme_en(bundle, target),
    };
    crate::store::write_file(out, "README.md", &mask(&body))?;
    Ok(())
}

/// Write a getting-started guide under `docs/`. User-facing (follows the README
/// language); the filename carries the lang code so multiple languages don't
/// collide. Only emitted for the polyglot (All) target — single-host renders
/// stay minimal. AI instructions (skills/agents/AGENTS.md) stay English. If the
/// LLM authored a `getting-started.<lang>` override, it replaces the skeleton.
fn write_docs_guide(bundle: &HarnessBundle, out: &Path, home: &Path) -> Result<()> {
    let lang_suffix = if bundle.language.as_str() == "ko" {
        "ko"
    } else {
        "en"
    };
    let body = if let Some(o) = crate::application::overrides::read_doc_override(
        home,
        &bundle.slug,
        "getting-started",
        lang_suffix,
    ) {
        o
    } else if lang_suffix == "ko" {
        docs_guide_ko(bundle)
    } else {
        docs_guide_en(bundle)
    };
    let name = format!("getting-started.{lang_suffix}.md");
    crate::store::write_file(&out.join("docs"), &name, &mask(&body))?;
    Ok(())
}

/// English getting-started guide. Covers harness-common structure (entry rule,
/// skill list from the bundle, output dirs, safety gates) — genre-neutral.
fn docs_guide_en(bundle: &HarnessBundle) -> String {
    let entry = bundle.entry_skill.as_deref().unwrap_or("spec");
    let skill_lines: Vec<String> = bundle
        .skills
        .iter()
        .map(|s| format!("- `{}` — {}", s.id, s.description.replace('\n', " ")))
        .collect();
    let agent_lines: Vec<String> = bundle
        .agents
        .iter()
        .map(|a| format!("- `{}` — {}", a.id, a.description.replace('\n', " ")))
        .collect();
    format!(
        "# Getting started — byoh-{slug}\n\n\
         This is a personalized BYOH harness. This guide covers the parts common to every harness.\n\n\
         ## Entry rule\n\n\
         Sessions begin with the `{entry}` skill (this harness's designated entry skill): it turns your \
         request into a structured spec and routes it to the right workflow. Skills persist their \
         output as files under the workspace — never keep results only in context.\n\n\
         ## Skills\n\n\
         {skills}\n\n\
         ## Agents\n\n\
         {agents}\n\n\
         ## Output locations\n\n\
         - Skills write notes, summaries, comparisons, and ideas to workspace files.\n\
         - Each skill documents its own output path in its SKILL.md.\n\n\
         ## Safety gates\n\n\
         Artifacts pass the BYOH gates before release: {gates}.\n",
        slug = bundle.slug,
        entry = entry,
        skills = if skill_lines.is_empty() {
            "_(none)_".into()
        } else {
            skill_lines.join("\n")
        },
        agents = if agent_lines.is_empty() {
            "_(none)_".into()
        } else {
            agent_lines.join("\n")
        },
        gates = bundle.safety_gates.join(", "),
    )
}

/// Korean getting-started guide (user-facing only; AI instructions stay English).
fn docs_guide_ko(bundle: &HarnessBundle) -> String {
    let entry = bundle.entry_skill.as_deref().unwrap_or("spec");
    let skill_lines: Vec<String> = bundle
        .skills
        .iter()
        .map(|s| format!("- `{}` — {}", s.id, s.description.replace('\n', " ")))
        .collect();
    let agent_lines: Vec<String> = bundle
        .agents
        .iter()
        .map(|a| format!("- `{}` — {}", a.id, a.description.replace('\n', " ")))
        .collect();
    format!(
        "# 시작하기 — byoh-{slug}\n\n\
         BYOH로 생성된 개인화 하네스입니다. 이 문서는 모든 하네스에 공통인 부분을 다룹니다.\n\n\
         ## 진입 규칙\n\n\
         세션은 `{entry}` 스킬(이 하네스의 지정 진입 스킬)에서 시작합니다. 사용자 요청을 \
         구조화된 명세로 바꾸고 알맞은 워크플로우로 분기합니다. 스킬은 산출물을 워크스페이스 파일로 \
         저장합니다 — 컨텍스트에만 두지 않습니다.\n\n\
         ## 스킬\n\n\
         {skills}\n\n\
         ## 에이전트\n\n\
         {agents}\n\n\
         ## 산출물 위치\n\n\
         - 스킬은 노트·요약·비교·아이디어를 워크스페이스 파일로 저장합니다.\n\
         - 각 스킬의 산출 경로는 해당 SKILL.md에 적혀 있습니다.\n\n\
         ## 안전 게이트\n\n\
         산출물은 출시 전 BYOH 게이트를 거칩니다: {gates}.\n",
        slug = bundle.slug,
        entry = entry,
        skills = if skill_lines.is_empty() {
            "_(없음)_".into()
        } else {
            skill_lines.join("\n")
        },
        agents = if agent_lines.is_empty() {
            "_(없음)_".into()
        } else {
            agent_lines.join("\n")
        },
        gates = bundle.safety_gates.join(", "),
    )
}

/// English README (canonical). User-facing doc; AI instructions stay English
/// regardless of this language.
fn readme_en(bundle: &HarnessBundle, target: Target) -> String {
    let targets_line = if target == Target::All {
        "polyglot (Claude Code + Codex + Antigravity)".to_string()
    } else {
        format!("target `{}`", target.as_str())
    };
    format!(
        "# byoh-{slug}\n\n\
         A personalized AI agent harness generated by [BYOH](https://github.com/epicsagas/BuildYourOwnHarness).\n\n\
         This directory contains the {targets} plugin. It is fully static — skills, \
         agents, manifests — so it works on any machine with no extra binaries.\n\n\
         ## Install\n\n\
         ### Claude Code\n\n\
         From a pushed GitHub repo (this tree ships its own `.claude-plugin/marketplace.json`):\n\n\
         ```bash\n\
         claude plugin marketplace add <github-owner>/<repo>\n\
         claude plugin install byoh-{slug}@byoh-{slug}\n\
         ```\n\n\
         From a local checkout: `claude plugin marketplace add /path/to/this-dir`, then the \
         same install command.\n\n\
         ### agy (Antigravity)\n\n\
         ```bash\n\
         agy plugin install <this-dir>\n\
         agy plugin enable byoh-{slug}\n\
         ```\n\n\
         ### Codex\n\n\
         ```bash\n\
         codex plugin marketplace add /path/to/this-dir\n\
         codex plugin add byoh-{slug}\n\
         ```\n\n\
         ## Structure\n\n\
         The single source of truth is the root `skills/` + `agents/`. Each host \
         reads it via its own manifest (`.claude-plugin/`, `.codex-plugin/`, root \
         `plugin.json` for agy); `.claude/`, `.gemini/`, `.codex/` carry symlinks \
         to `skills` and `agents` for project-local use. Note: symlinks require a \
         symlink-capable checkout (on Windows, enable `core.symlinks`).\n\n\
         ## Contents\n\n\
         - Genre: `{genre}`\n\
         - Skills: {n_skills} · Agents: {n_agents}\n\
         - Safety gates (BYOH compile/evolve-time): {gates}\n",
        slug = bundle.slug,
        targets = targets_line,
        genre = bundle.genre.as_str(),
        n_skills = bundle.skills.len(),
        n_agents = bundle.agents.len(),
        gates = bundle.safety_gates.join(", "),
    )
}

/// Korean README. User-facing only; AI instructions (skills/agents/AGENTS.md)
/// remain English by design.
fn readme_ko(bundle: &HarnessBundle, target: Target) -> String {
    let targets_line = if target == Target::All {
        "다호스트 폴리글롯(Claude Code + Codex + Antigravity)".to_string()
    } else {
        format!("`{}` 타겟", target.as_str())
    };
    format!(
        "# byoh-{slug}\n\n\
         [BYOH](https://github.com/epicsagas/BuildYourOwnHarness)로 생성된 AI 에이전트 하네스입니다.\n\n\
         이 디렉토리는 {targets} 플러그인입니다. 스킬·에이전트·매니페스트만으로 구성된 \
         정적 플러그인이라 추가 바이너리 없이 어느 머신에서든 동작합니다.\n\n\
         ## 설치\n\n\
         ### Claude Code\n\n\
         깃헙에 push한 리포에서 (이 트리는 자체 `.claude-plugin/marketplace.json`을 포함합니다):\n\n\
         ```bash\n\
         claude plugin marketplace add <github-owner>/<repo>\n\
         claude plugin install byoh-{slug}@byoh-{slug}\n\
         ```\n\n\
         로컬 체크아웃에서는 `claude plugin marketplace add /path/to/this-dir` 후 같은 install \
         명령을 실행하세요.\n\n\
         ### agy (Antigravity)\n\n\
         ```bash\n\
         agy plugin install <이-디렉토리>\n\
         agy plugin enable byoh-{slug}\n\
         ```\n\n\
         ### Codex\n\n\
         ```bash\n\
         codex plugin marketplace add /path/to/this-dir\n\
         codex plugin add byoh-{slug}\n\
         ```\n\n\
         ## 구조\n\n\
         단일 진실원천은 루트의 `skills/` + `agents/` 입니다. 각 호스트는 자기 매니페스트\
         (`.claude-plugin/`, `.codex-plugin/`, agy용 루트 `plugin.json`)로 이를 읽고, \
         `.claude/`, `.gemini/`, `.codex/`는 프로젝트-로컬 사용을 위한 `skills`/`agents` \
         심볼릭링크를 담습니다. 참고: 심볼릭링크는 지원되는 체크아웃이 필요합니다 \
         (Windows에서는 `core.symlinks` 활성화).\n\n\
         ## 내용\n\n\
         - 장르(genre): `{genre}`\n\
         - 스킬: {n_skills}개 · 에이전트: {n_agents}개\n\
         - 안전 게이트(BYOH 컴파일/진화 시점): {gates}\n",
        slug = bundle.slug,
        targets = targets_line,
        genre = bundle.genre.as_str(),
        n_skills = bundle.skills.len(),
        n_agents = bundle.agents.len(),
        gates = bundle.safety_gates.join(", "),
    )
}

fn pretty(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_else(|_| "{}".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::compile_profile;
    use crate::domain::bundle::Ring;
    use crate::domain::genre::Genre;
    use crate::domain::profile::{GenreConfidence, ProfileStatus, UserProfile};

    fn confirmed_profile() -> UserProfile {
        let mut p = UserProfile::new_draft("dev", "en");
        p.candidates.identity.genre = Some(GenreConfidence {
            value: Genre::Developer,
            confidence: 1.0,
            provenance: vec![],
        });
        p.status = ProfileStatus::Confirmed;
        p
    }

    #[test]
    fn render_claude_emits_manifest_and_agents_md() {
        let bundle = compile_profile(&confirmed_profile()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        render_claude(&bundle, dir.path()).unwrap();

        let manifest: Value = serde_json::from_str(
            &std::fs::read_to_string(dir.path().join(".claude-plugin/plugin.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(manifest["name"], "byoh-dev");
        assert_eq!(manifest["skills"], "./skills/");
        assert!(
            manifest["agents"].is_array(),
            "Claude agents must be an array"
        );
        assert!(
            manifest["agents"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "./agents/code-reviewer.md")
        );

        // agents/<id>.md frontmatter present.
        let md = std::fs::read_to_string(dir.path().join("agents/code-reviewer.md")).unwrap();
        assert!(md.starts_with("---\nname: code-reviewer"));
        assert!(md.contains("tools:"));

        // AGENTS.md root.
        assert!(dir.path().join("AGENTS.md").exists());
    }

    #[test]
    fn render_codex_emits_toml_agents_with_three_keys() {
        let bundle = compile_profile(&confirmed_profile()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        render_codex(&bundle, dir.path()).unwrap();

        let toml_path = dir.path().join(".codex-plugin/agents/code-reviewer.toml");
        assert!(toml_path.exists(), "Codex agent .toml must exist");
        let body = std::fs::read_to_string(&toml_path).unwrap();
        // exactly the 3 keys
        assert!(body.starts_with("name = \"code-reviewer\""));
        assert!(body.contains("\ndescription = \""));
        assert!(body.contains("developer_instructions = \"\"\""));
        // no tools/model keys (verified Codex schema)
        assert!(!body.contains("\ntools"));
        assert!(!body.contains("\nmodel"));

        // symlinks exist (unix) — resolve without error.
        let agents_link = dir.path().join(".codex/agents");
        assert!(agents_link.exists(), ".codex/agents must resolve");

        // .codex/config.toml
        assert!(dir.path().join(".codex/config.toml").exists());
    }

    #[test]
    fn render_agy_emits_plugin_json_and_dirs() {
        let bundle = compile_profile(&confirmed_profile()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        render_agy(&bundle, dir.path()).unwrap();

        // plugin.json is REQUIRED and strict: only $schema + name + description.
        let manifest: Value =
            serde_json::from_str(&std::fs::read_to_string(dir.path().join("plugin.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["name"], "byoh-dev");
        assert!(manifest["description"].is_string());
        // additionalProperties:false — must NOT carry version/author/etc.
        let obj = manifest.as_object().unwrap();
        let mut keys: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
        keys.sort();
        assert_eq!(
            keys,
            vec!["$schema", "description", "name"],
            "agy plugin.json must be exactly 3 keys"
        );

        // agents/ + skills/ dirs.
        assert!(dir.path().join("agents/code-reviewer.md").exists());
        assert!(
            std::fs::read_dir(dir.path().join("skills"))
                .unwrap()
                .count()
                > 0
        );
        // NOT a Claude/Codex plugin dir.
        assert!(!dir.path().join(".claude-plugin").exists());
        assert!(!dir.path().join(".codex-plugin").exists());
        assert!(dir.path().join("AGENTS.md").exists());
    }

    #[test]
    fn rendered_plugin_is_static_no_mcp_no_hooks() {
        // Regression: the rendered plugin must NOT wire bundle mcp_tools/hooks
        // into host config. An mcp_config.json pointing at `byoh` + a local
        // profile is a dead server on every other machine, and hook commands
        // have no executable — both were confirmed DOA in review.
        let mut bundle = compile_profile(&confirmed_profile()).unwrap();
        bundle.mcp_tools = vec![crate::domain::bundle::McpTool {
            name: "byoh-search".into(),
            description: "search".into(),
            input_schema: serde_json::json!({"type": "object"}),
        }];
        let dir = tempfile::tempdir().unwrap();
        render_target(&bundle, Target::All, dir.path(), &crate::store::byoh_home()).unwrap();

        for f in ["mcp_config.json", ".mcp.json", "hooks.json"] {
            assert!(!dir.path().join(f).exists(), "{f} must not be rendered");
        }
        assert!(!dir.path().join(".claude-plugin/hooks.json").exists());
        assert!(!dir.path().join(".codex-plugin/hooks.json").exists());
        for host_manifest in [".claude-plugin/plugin.json", ".codex-plugin/plugin.json"] {
            let m: Value = serde_json::from_str(
                &std::fs::read_to_string(dir.path().join(host_manifest)).unwrap(),
            )
            .unwrap();
            assert!(
                m.get("mcpServers").is_none(),
                "{host_manifest} must not reference an MCP server"
            );
            assert!(m.get("hooks").is_none());
        }
    }

    #[test]
    fn rendered_plugin_ships_claude_marketplace() {
        // `claude plugin marketplace add <repo>` requires marketplace.json —
        // without it, "push and install" is false for Claude Code.
        let bundle = compile_profile(&confirmed_profile()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        render_target(&bundle, Target::All, dir.path(), &crate::store::byoh_home()).unwrap();
        let m: Value = serde_json::from_str(
            &std::fs::read_to_string(dir.path().join(".claude-plugin/marketplace.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(m["name"], "byoh-dev");
        let plugins = m["plugins"].as_array().unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0]["name"], "byoh-dev");
        assert_eq!(plugins[0]["source"], "./");
    }

    #[test]
    fn render_refuses_non_byoh_output_dir_and_allows_rerender() {
        let bundle = compile_profile(&confirmed_profile()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        // Simulate a real project dir: has a README the render would clobber.
        std::fs::write(dir.path().join("README.md"), "my real project").unwrap();
        let err = render_target(&bundle, Target::All, dir.path(), &crate::store::byoh_home());
        assert!(err.is_err(), "must refuse a non-empty non-BYOH dir");
        assert_eq!(
            std::fs::read_to_string(dir.path().join("README.md")).unwrap(),
            "my real project",
            "existing files must be untouched"
        );

        // Fresh dir renders fine, and re-rendering over a BYOH-owned tree works.
        let fresh = tempfile::tempdir().unwrap();
        render_target(
            &bundle,
            Target::All,
            fresh.path(),
            &crate::store::byoh_home(),
        )
        .unwrap();
        assert!(fresh.path().join(".byoh-manifest").exists());
        render_target(
            &bundle,
            Target::All,
            fresh.path(),
            &crate::store::byoh_home(),
        )
        .unwrap();
    }

    #[test]
    fn rendered_markdown_is_secret_masked() {
        let mut bundle = compile_profile(&confirmed_profile()).unwrap();
        // A leaked token pasted into a skill body must not survive rendering.
        bundle.skills[0].body_markdown = "use TOKEN=supersecretvalue1234 to auth".into();
        let dir = tempfile::tempdir().unwrap();
        render_target(&bundle, Target::All, dir.path(), &crate::store::byoh_home()).unwrap();
        let skill_dir = dir.path().join("skills").join(&bundle.skills[0].id);
        let body = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
        assert!(
            !body.contains("supersecretvalue1234"),
            "secret must be masked in rendered SKILL.md"
        );
    }

    #[test]
    fn render_all_creates_polyglot_tree() {
        let bundle = compile_profile(&confirmed_profile()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        render_target(&bundle, Target::All, dir.path(), &crate::store::byoh_home()).unwrap();

        // One tree carrying all three hosts' manifests.
        assert!(dir.path().join(".claude-plugin/plugin.json").exists());
        assert!(dir.path().join(".codex-plugin/plugin.json").exists());
        assert!(
            dir.path().join("plugin.json").exists(),
            "agy root plugin.json"
        );
        assert!(dir.path().join("README.md").exists());

        // No per-host subdir split.
        for t in ["claude", "codex", "agy"] {
            assert!(
                !dir.path().join(t).exists(),
                "{t}/ must not exist in a polyglot tree"
            );
        }
    }

    #[test]
    fn render_polyglot_writes_shared_paths_once() {
        let bundle = compile_profile(&confirmed_profile()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        render_target(&bundle, Target::All, dir.path(), &crate::store::byoh_home()).unwrap();

        // Shared skills/agents/AGENTS.md exist exactly once (not triplicated).
        assert!(dir.path().join("AGENTS.md").exists());
        assert!(dir.path().join("agents/code-reviewer.md").exists());
        assert!(
            std::fs::read_dir(dir.path().join("skills"))
                .unwrap()
                .count()
                > 0,
            "shared skills/ must be populated"
        );
    }

    #[test]
    fn render_polyglot_codex_toml_coexists_with_agents_md() {
        let bundle = compile_profile(&confirmed_profile()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        render_target(&bundle, Target::All, dir.path(), &crate::store::byoh_home()).unwrap();

        // Codex TOML agents + shared .md agents coexist (different dirs).
        assert!(
            dir.path()
                .join(".codex-plugin/agents/code-reviewer.toml")
                .exists()
        );
        assert!(dir.path().join("agents/code-reviewer.md").exists());
    }

    #[test]
    fn render_polyglot_creates_host_symlinks() {
        let bundle = compile_profile(&confirmed_profile()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        render_target(&bundle, Target::All, dir.path(), &crate::store::byoh_home()).unwrap();

        // Each host dir links the shared skills + agents.
        for host in [".claude", ".gemini", ".codex"] {
            let h = dir.path().join(host);
            assert!(h.join("skills").exists(), "{host}/skills must resolve");
            assert!(h.join("agents").exists(), "{host}/agents must resolve");
        }
    }

    #[test]
    fn render_polyglot_readme_follows_language() {
        // Korean profile → Korean README (user doc); AI instructions stay English.
        let mut p = confirmed_profile();
        p.language = "ko".into();
        let bundle = compile_profile(&p).unwrap();
        let dir = tempfile::tempdir().unwrap();
        render_target(&bundle, Target::All, dir.path(), &crate::store::byoh_home()).unwrap();
        let readme = std::fs::read_to_string(dir.path().join("README.md")).unwrap();
        assert!(readme.contains("설치"), "ko profile → Korean README");
        // AGENTS.md is AI-facing → always English regardless of language.
        let agents = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
        assert!(
            agents.contains("BYOH Harness"),
            "AGENTS.md stays English (AI instruction)"
        );
    }

    #[test]
    fn agent_toml_drops_tools_and_escapes_quotes() {
        let a = AgentSpec {
            id: "x".into(),
            name: "X".into(),
            description: "has \"quotes\"".into(),
            body_markdown: "body".into(),
            tools: Some(vec!["Read".into()]),
        };
        let t = agent_toml(&a);
        assert!(!t.contains("tools"));
        assert!(t.contains("developer_instructions = \"\"\""));
    }

    #[test]
    fn skill_md_has_frontmatter() {
        let s = SkillSpec {
            id: "tdd".into(),
            ring: Ring::Ring2,
            name: "TDD".into(),
            description: "test first".into(),
            body_markdown: "# TDD".into(),
            pipeline: None,
            order: None,
        };
        let md = skill_md(&s);
        assert!(md.starts_with("---\nname: tdd\ndescription: test first\n---"));
    }

    #[test]
    fn write_publish_extras_adds_license_and_gitignore() {
        let dir = tempfile::tempdir().unwrap();
        write_publish_extras(dir.path()).unwrap();
        let license = std::fs::read_to_string(dir.path().join("LICENSE")).unwrap();
        assert!(
            license.contains("Apache License") && license.contains("Version 2.0"),
            "LICENSE must be the full Apache-2.0 text"
        );
        let gi = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(gi.contains(".byoh-manifest") && gi.contains("*.log"));
    }
}

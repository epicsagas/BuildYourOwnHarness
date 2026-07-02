//! Target renderer — emit a self-contained plugin tree for a host tool.
//!
//! Takes a [`HarnessBundle`] (built by `compile_profile` / `synthesize`) and
//! writes a deployable plugin into `out`. The output dir is `git init`-ready:
//! a user can push it and anyone who clones gets a working plugin.
//!
//! Claude/Codex formats are grounded in real epiccounty reference projects
//! (korean-law-rag, epic-harness, Velith, obsidian-forge). The agy layout
//! follows the official Antigravity CLI plugin spec (plugin.json marker +
//! skills/agents/hooks.json/mcp_config.json/rules). See the plan doc.

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
pub fn render_target(bundle: &HarnessBundle, target: Target, out: &Path) -> Result<PathBuf> {
    if target == Target::All {
        render_polyglot(bundle, out)?;
        write_readme(bundle, out, Target::All)?;
        write_docs_guide(bundle, out)?;
        return Ok(out.to_path_buf());
    }
    render_one(bundle, target, out)?;
    write_readme(bundle, out, target)?;
    Ok(out.to_path_buf())
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

    // Claude manifest.
    crate::store::write_file(
        &out.join(".claude-plugin"),
        "plugin.json",
        &pretty(&claude_manifest(bundle)),
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

    // Per-host hooks (distinct paths + bodies).
    if !bundle.hooks.is_empty() {
        crate::store::write_file(
            &out.join(".claude-plugin"),
            "hooks.json",
            &pretty(&hooks_json(bundle, "${CLAUDE_PLUGIN_ROOT}", false)),
        )?;
        crate::store::write_file(
            &out.join(".codex-plugin"),
            "hooks.json",
            &pretty(&codex_hooks(bundle)),
        )?;
        // Root hooks.json is the agy (Antigravity) schema — a distinct shape
        // from Claude's, with SessionStart/End remapped to PreInvocation/Stop.
        crate::store::write_file(out, "hooks.json", &pretty(&agy_hooks(bundle)))?;
    }

    // Shared MCP: one `mcp_config.json` at the root. Claude and Codex reference
    // it via their manifest `mcpServers` field; agy reads it from the root.
    if !bundle.mcp_tools.is_empty() {
        crate::store::write_file(out, "mcp_config.json", &pretty(&mcp_servers(bundle)))?;
    }

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
    crate::store::write_file(out, "AGENTS.md", &root_agents_md(bundle))?;

    // Per-host discovery dirs (.claude/.gemini/.codex) linking shared skills/
    // agents and the host-specific hooks.json.
    write_host_symlinks(out, !bundle.hooks.is_empty())?;
    Ok(())
}

// ─── common builders ────────────────────────────────────────────────────────

/// Create the per-host discovery directories (`.claude/`, `.gemini/`) that link
/// the shared `skills/`, `agents/`, and host-specific `hooks.json`. The single
/// source of truth stays at the root + `.{host}-plugin/`; hosts discover via
/// their own dir. `.codex/` already carries skills/agents links from
/// [`render_polyglot`]; only its `hooks.json` link is added here.
///
/// `has_hooks` gates the `hooks.json` links so they never dangle when a bundle
/// has no hooks. Uses [`create_symlink_or_copy`] (symlink on Unix, recursive
/// copy fallback elsewhere) — the same mechanism as the existing `.codex` links.
fn write_host_symlinks(out: &Path, has_hooks: bool) -> Result<()> {
    // `.claude/` → Claude-plugin skills/agents/hooks.
    let claude = out.join(".claude");
    crate::store::create_symlink_or_copy(&PathBuf::from("../skills"), &claude.join("skills"))?;
    crate::store::create_symlink_or_copy(&PathBuf::from("../agents"), &claude.join("agents"))?;
    if has_hooks {
        crate::store::create_symlink_or_copy(
            &PathBuf::from("../.claude-plugin/hooks.json"),
            &claude.join("hooks.json"),
        )?;
    }

    // `.gemini/` → agy (root) skills/agents/hooks. agy's hooks.json is the root
    // file (distinct agy schema), NOT .claude-plugin's.
    let gemini = out.join(".gemini");
    crate::store::create_symlink_or_copy(&PathBuf::from("../skills"), &gemini.join("skills"))?;
    crate::store::create_symlink_or_copy(&PathBuf::from("../agents"), &gemini.join("agents"))?;
    if has_hooks {
        crate::store::create_symlink_or_copy(
            &PathBuf::from("../hooks.json"),
            &gemini.join("hooks.json"),
        )?;
    }

    // `.codex/` already has skills/agents links; add the hooks.json link only.
    if has_hooks {
        crate::store::create_symlink_or_copy(
            &PathBuf::from("../.codex-plugin/hooks.json"),
            &out.join(".codex").join("hooks.json"),
        )?;
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
    if !bundle.mcp_tools.is_empty() {
        manifest["mcpServers"] = json!("./mcp_config.json");
    }
    manifest
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
    if !bundle.hooks.is_empty() {
        manifest["hooks"] = json!("./.codex-plugin/hooks.json");
    }
    if !bundle.mcp_tools.is_empty() {
        manifest["mcpServers"] = json!("./mcp_config.json");
    }
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

/// Claude/hooks.json shape. Codex uses a variant (see codex_hooks).
fn hooks_json(bundle: &HarnessBundle, root_var: &str, versioned: bool) -> Value {
    // Group hooks by event. Each event → list of {matcher, hooks:[{type,command}], description}.
    let mut by_event: std::collections::BTreeMap<String, Vec<Value>> =
        std::collections::BTreeMap::new();
    for h in &bundle.hooks {
        by_event.entry(h.event.clone()).or_default().push(json!({
            "matcher": "*",
            "hooks": [{ "type": "command", "command": h.command.replace("${PLUGIN_ROOT}", root_var) }],
            "description": format!("{} hook (reads: {})", h.event, h.reads.join(", ")),
        }));
    }
    let hooks = json!(by_event);
    if versioned {
        json!({ "version": 1, "hooks": hooks })
    } else {
        json!({ "hooks": hooks })
    }
}

/// `mcp_config.json` (shared MCP config): { mcpServers: { <slug>: {command, args} } }.
///
/// A single server entry, keyed by the harness slug, launches `byoh
/// harness-serve <slug>` — the real `byoh` binary, not a fabricated
/// `byoh-<tool>` binary that never existed on disk. That subcommand loads
/// this bundle's `mcp_tools` and serves them over stdio MCP (see
/// `src/mcp/harness_server.rs`), so every tool the harness declares is
/// actually reachable instead of failing at process spawn.
fn mcp_servers(bundle: &HarnessBundle) -> Value {
    json!({
        "mcpServers": {
            bundle.slug.clone(): {
                "command": "byoh",
                "args": ["harness-serve", bundle.slug.clone()],
            }
        }
    })
}

fn write_skills(bundle: &HarnessBundle, root: &Path) -> Result<()> {
    for skill in &bundle.skills {
        let dir = root.join("skills").join(&skill.id);
        crate::store::write_file(&dir, "SKILL.md", &skill_md(skill))?;
    }
    Ok(())
}

/// Shared `agents/<id>.md` (Claude and agy share this format; Codex uses TOML).
fn write_agents_md(bundle: &HarnessBundle, out: &Path) -> Result<()> {
    for agent in &bundle.agents {
        crate::store::write_file(
            &out.join("agents"),
            &format!("{}.md", agent.id),
            &agent_md(agent),
        )?;
    }
    Ok(())
}

// ─── Claude Code ────────────────────────────────────────────────────────────

fn render_claude(bundle: &HarnessBundle, out: &Path) -> Result<()> {
    // .claude-plugin/plugin.json — agents as array of file paths.
    crate::store::write_file(
        &out.join(".claude-plugin"),
        "plugin.json",
        &pretty(&claude_manifest(bundle)),
    )?;

    // skills/<id>/SKILL.md + agents/<id>.md
    write_skills(bundle, out)?;
    write_agents_md(bundle, out)?;

    // hooks.json (Claude: ${CLAUDE_PLUGIN_ROOT}).
    if !bundle.hooks.is_empty() {
        crate::store::write_file(
            &out.join(".claude-plugin"),
            "hooks.json",
            &pretty(&hooks_json(bundle, "${CLAUDE_PLUGIN_ROOT}", false)),
        )?;
    }

    // mcp_config.json at the root (referenced by the manifest's `mcpServers` field).
    if !bundle.mcp_tools.is_empty() {
        crate::store::write_file(out, "mcp_config.json", &pretty(&mcp_servers(bundle)))?;
    }

    // AGENTS.md root system prompt.
    crate::store::write_file(out, "AGENTS.md", &root_agents_md(bundle))?;
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

    // hooks.json (Codex: version:1, Stop event, ${PLUGIN_ROOT}).
    if !bundle.hooks.is_empty() {
        crate::store::write_file(
            &out.join(".codex-plugin"),
            "hooks.json",
            &pretty(&codex_hooks(bundle)),
        )?;
    }

    // mcp_config.json at the root (referenced by the codex manifest's `mcpServers` field).
    if !bundle.mcp_tools.is_empty() {
        crate::store::write_file(out, "mcp_config.json", &pretty(&mcp_servers(bundle)))?;
    }

    // AGENTS.md.
    crate::store::write_file(out, "AGENTS.md", &root_agents_md(bundle))?;
    Ok(())
}

/// Codex hooks: remap SessionEnd → Stop (Codex event name), version:1, ${PLUGIN_ROOT}.
fn codex_hooks(bundle: &HarnessBundle) -> Value {
    let mut remapped = bundle.hooks.clone();
    for h in &mut remapped {
        if h.event == "SessionEnd" {
            h.event = "Stop".into();
        }
    }
    let mut tmp_bundle = bundle.clone();
    tmp_bundle.hooks = remapped;
    hooks_json(&tmp_bundle, "${PLUGIN_ROOT}", true)
}

/// Antigravity (agy) hooks schema. agy uses a DIFFERENT shape than Claude:
///   { "<hook-name>": { "<EventName>": [ { matcher?, hooks: [...] } ] } }
/// (top-level is a map keyed by hook name, not `{ hooks: { Event: [...] } }`).
/// agy does NOT support SessionStart/SessionEnd — remap them:
///   SessionStart → PreInvocation, SessionEnd → Stop.
/// PreToolUse/PostToolUse pass through. `matcher` is included for tool events
/// and omitted for PreInvocation/Stop (which ignore matchers). No
/// `description` field (not in the agy schema). Each command carries a 30s
/// timeout. `${PLUGIN_ROOT}` resolves at agy load time.
///
/// Verified shape: `/Users/hackme/Downloads/paper-whisperer/dist/.../hooks.json`.
fn agy_hooks(bundle: &HarnessBundle) -> Value {
    // hook-name key → { agy_event → matcher? }. Insertion order is preserved by
    // serde_json::Map (preserve_order feature), so output is deterministic.
    let mut map = serde_json::Map::new();
    for h in &bundle.hooks {
        let Some((hook_key, agy_event)) = agy_hook_key(&h.event) else {
            continue; // unsupported event for agy — drop it.
        };
        let entry = {
            let hooks_arr = json!([{
                "type": "command",
                "command": h.command,
                "timeout": 30,
            }]);
            // Tool events carry a matcher; PreInvocation/Stop omit it (ignored).
            if agy_event == "PreToolUse" || agy_event == "PostToolUse" {
                json!([{
                    "matcher": "*",
                    "hooks": hooks_arr,
                }])
            } else {
                json!([{ "hooks": hooks_arr }])
            }
        };
        // Each hook-name key maps to a single event bucket.
        let bucket = map.entry(hook_key.to_string()).or_insert_with(|| json!({}));
        bucket[agy_event] = entry;
    }
    Value::Object(map)
}

/// Map a neutral hook event (from `bundle/hooks/hooks.json`) to its agy
/// equivalent: `(hook-name key, agy event name)`. Returns `None` for events agy
/// cannot express (dropped rather than emitted broken).
fn agy_hook_key(event: &str) -> Option<(&'static str, &'static str)> {
    match event {
        "SessionStart" => Some(("byoh-session-resume", "PreInvocation")),
        "SessionEnd" => Some(("byoh-session-end-observe", "Stop")),
        "PreToolUse" => Some(("byoh-pre-tool-guard", "PreToolUse")),
        "PostToolUse" => Some(("byoh-post-tool-compress", "PostToolUse")),
        _ => None,
    }
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

    // hooks.json at the plugin root (NOT a hooks/ subdir). agy schema: a
    // hook-name-keyed map with PreInvocation/Stop (not SessionStart/End).
    if !bundle.hooks.is_empty() {
        crate::store::write_file(out, "hooks.json", &pretty(&agy_hooks(bundle)))?;
    }

    // mcp_config.json at the plugin root — agy reads MCP from here (NOT .mcp.json).
    if !bundle.mcp_tools.is_empty() {
        crate::store::write_file(out, "mcp_config.json", &pretty(&mcp_servers(bundle)))?;
    }

    // AGENTS.md root system prompt (still useful as a harness overview).
    crate::store::write_file(out, "AGENTS.md", &root_agents_md(bundle))?;
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

fn write_readme(bundle: &HarnessBundle, out: &Path, target: Target) -> Result<()> {
    let body = match bundle.language.as_str() {
        "ko" => readme_ko(bundle, target),
        // English is the canonical fallback for every other / unknown language.
        _ => readme_en(bundle, target),
    };
    crate::store::write_file(out, "README.md", &body)?;
    Ok(())
}

/// Write a getting-started guide under `docs/`. User-facing (follows the README
/// language); the filename carries the lang code so multiple languages don't
/// collide. Only emitted for the polyglot (All) target — single-host renders
/// stay minimal. AI instructions (skills/agents/AGENTS.md) stay English.
fn write_docs_guide(bundle: &HarnessBundle, out: &Path) -> Result<()> {
    let (body, lang_suffix) = match bundle.language.as_str() {
        "ko" => (docs_guide_ko(bundle), "ko"),
        _ => (docs_guide_en(bundle), "en"),
    };
    let name = format!("getting-started.{lang_suffix}.md");
    crate::store::write_file(&out.join("docs"), &name, &body)?;
    Ok(())
}

/// English getting-started guide. Covers harness-common structure (entry rule,
/// skill list from the bundle, output dirs, safety gates) — genre-neutral.
fn docs_guide_en(bundle: &HarnessBundle) -> String {
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
         Sessions begin with the `spec` skill (or this harness's designated entry skill): it turns your \
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
         세션은 `spec` 스킬(또는 이 하네스의 지정 진입 스킬)에서 시작합니다. 사용자 요청을 \
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
         This directory contains the {targets} plugin and is ready to `git init && git push`.\n\n\
         ## Install\n\n\
         Clone this repo into your host tool's plugin location, or follow the host's \
         plugin-install flow:\n\n\
         - **Claude Code**: the `.claude-plugin/` manifest is auto-discovered.\n\
         - **Codex**: the `.codex-plugin/` manifest + `.codex/` config are auto-discovered.\n\
         - **agy (Antigravity)**: `agy plugin install <this-dir>` (the `plugin.json` \
         marker + skills/agents/hooks.json/mcp_config.json are staged under \
         `~/.gemini/config/plugins/`).\n\n\
         ## Structure\n\n\
         The single source of truth is the root `skills/` + `agents/`. Each host \
         discovers it via its own directory (symlinks): `.claude/`, `.gemini/`, \
         `.codex/` each link `skills`, `agents`, and the host-specific `hooks.json`.\n\n\
         ## Contents\n\n\
         - Genre: `{genre}`\n\
         - Skills: {n_skills} · Agents: {n_agents}\n\
         - Safety gates: {gates}\n",
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
         이 디렉토리는 {targets} 플러그인이며, `git init && git push` 하면 바로 사용할 수 있습니다.\n\n\
         ## 설치\n\n\
         각 호스트의 플러그인 설치 흐름을 따르거나, 플러그인 위치에 클론하세요.\n\n\
         - **Claude Code**: `.claude-plugin/` 매니페스트가 자동 인식됩니다.\n\
         - **Codex**: `.codex-plugin/` 매니페스트 + `.codex/` 설정이 자동 인식됩니다.\n\
         - **agy (Antigravity)**: `agy plugin install <이-디렉토리>` (`plugin.json` 마커 + \
         skills/agents/hooks.json/mcp_config.json 이 `~/.gemini/config/plugins/` 아래에 스테이징됩니다).\n\n\
         ## 구조\n\n\
         단일 진실원천은 루트의 `skills/` + `agents/` 입니다. 각 호스트는 자기 디렉토리\
         (`.claude/`, `.gemini/`, `.codex/`)에서 심볼릭링크로 `skills`, `agents`, 호스트별 \
         `hooks.json`을 가리켜 발견합니다.\n\n\
         ## 내용\n\n\
         - 장르(genre): `{genre}`\n\
         - 스킬: {n_skills}개 · 에이전트: {n_agents}개\n\
         - 안전 게이트: {gates}\n",
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

    fn bundle_with_mcp() -> HarnessBundle {
        let mut bundle = compile_profile(&confirmed_profile()).unwrap();
        bundle.mcp_tools = vec![crate::domain::bundle::McpTool {
            name: "byoh-search".into(),
            description: "search".into(),
            input_schema: serde_json::json!({"type": "object"}),
        }];
        bundle
    }

    #[test]
    fn render_agy_uses_mcp_config_json() {
        let bundle = bundle_with_mcp();
        let dir = tempfile::tempdir().unwrap();
        render_agy(&bundle, dir.path()).unwrap();
        // agy reads MCP from mcp_config.json (verified live — it does NOT read .mcp.json).
        assert!(dir.path().join("mcp_config.json").exists());
        assert!(
            !dir.path().join(".mcp.json").exists(),
            "agy must not emit .mcp.json (it reads mcp_config.json)"
        );
    }

    #[test]
    fn render_codex_emits_mcp_config_and_manifest_ref() {
        let bundle = bundle_with_mcp();
        let dir = tempfile::tempdir().unwrap();
        render_codex(&bundle, dir.path()).unwrap();
        assert!(
            dir.path().join("mcp_config.json").exists(),
            "codex must emit shared mcp_config.json"
        );
        let m: Value = serde_json::from_str(
            &std::fs::read_to_string(dir.path().join(".codex-plugin/plugin.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            m["mcpServers"], "./mcp_config.json",
            "codex manifest must reference mcp_config.json via mcpServers"
        );
    }

    #[test]
    fn render_claude_manifest_references_mcp() {
        let bundle = bundle_with_mcp();
        let dir = tempfile::tempdir().unwrap();
        render_claude(&bundle, dir.path()).unwrap();
        assert!(dir.path().join("mcp_config.json").exists());
        let m: Value = serde_json::from_str(
            &std::fs::read_to_string(dir.path().join(".claude-plugin/plugin.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            m["mcpServers"], "./mcp_config.json",
            "claude manifest must reference mcp_config.json via mcpServers"
        );
    }

    #[test]
    fn render_polyglot_emits_single_mcp_config() {
        let bundle = bundle_with_mcp();
        let dir = tempfile::tempdir().unwrap();
        render_target(&bundle, Target::All, dir.path()).unwrap();
        // One shared mcp_config.json — claude/codex via mcpServers field, agy from root.
        assert!(
            dir.path().join("mcp_config.json").exists(),
            "polyglot tree has shared mcp_config.json"
        );
        assert!(
            !dir.path().join(".mcp.json").exists(),
            "no legacy .mcp.json (consolidated to mcp_config.json)"
        );
    }

    #[test]
    fn mcp_config_command_is_real_byoh_binary() {
        // Regression: the server entry must launch the actual `byoh` binary via
        // its `harness-serve` subcommand, not a fabricated `byoh-<tool>` binary
        // that was never installed anywhere.
        let bundle = bundle_with_mcp();
        let dir = tempfile::tempdir().unwrap();
        render_target(&bundle, Target::All, dir.path()).unwrap();
        let cfg: Value = serde_json::from_str(
            &std::fs::read_to_string(dir.path().join("mcp_config.json")).unwrap(),
        )
        .unwrap();
        let servers = cfg["mcpServers"].as_object().unwrap();
        assert_eq!(servers.len(), 1, "one server entry keyed by harness slug");
        let entry = servers
            .get(&bundle.slug)
            .expect("server keyed by bundle.slug");
        assert_eq!(entry["command"], "byoh");
        assert_eq!(entry["args"], json!(["harness-serve", bundle.slug]));
    }

    #[test]
    fn render_all_creates_polyglot_tree() {
        let bundle = compile_profile(&confirmed_profile()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        render_target(&bundle, Target::All, dir.path()).unwrap();

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
        render_target(&bundle, Target::All, dir.path()).unwrap();

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
        render_target(&bundle, Target::All, dir.path()).unwrap();

        // Codex TOML agents + shared .md agents coexist (different dirs).
        assert!(
            dir.path()
                .join(".codex-plugin/agents/code-reviewer.toml")
                .exists()
        );
        assert!(dir.path().join("agents/code-reviewer.md").exists());
    }

    #[test]
    fn codex_hooks_remap_sessionend_to_stop() {
        let mut bundle = compile_profile(&confirmed_profile()).unwrap();
        bundle.hooks = vec![crate::domain::bundle::HookSpec {
            event: "SessionEnd".into(),
            command: "epic reflect".into(),
            reads: vec![],
        }];
        let v = codex_hooks(&bundle);
        assert_eq!(v["version"], 1);
        assert!(v["hooks"]["Stop"].is_array(), "SessionEnd must map to Stop");
    }

    #[test]
    fn agy_hooks_remap_session_events() {
        // Full neutral set from bundle/hooks/hooks.json.
        let mut bundle = compile_profile(&confirmed_profile()).unwrap();
        bundle.hooks = vec![
            crate::domain::bundle::HookSpec {
                event: "SessionStart".into(),
                command: "byoh hook resume".into(),
                reads: vec![],
            },
            crate::domain::bundle::HookSpec {
                event: "SessionEnd".into(),
                command: "byoh hook observe".into(),
                reads: vec![],
            },
            crate::domain::bundle::HookSpec {
                event: "PreToolUse".into(),
                command: "byoh hook guard".into(),
                reads: vec![],
            },
        ];
        let v = agy_hooks(&bundle);
        let obj = v.as_object().expect("agy hooks is a hook-name-keyed map");
        // Top-level keys are hook names, NOT Claude's {hooks: {...}}.
        assert!(
            !obj.contains_key("hooks"),
            "agy hooks.json must NOT use the Claude {{hooks:{{}}}} shape"
        );
        // SessionStart → PreInvocation under its hook-name key.
        assert!(
            obj["byoh-session-resume"]["PreInvocation"].is_array(),
            "SessionStart must map to PreInvocation"
        );
        // SessionEnd → Stop.
        assert!(
            obj["byoh-session-end-observe"]["Stop"].is_array(),
            "SessionEnd must map to Stop"
        );
        // PreToolUse keeps a matcher.
        let pre = &obj["byoh-pre-tool-guard"]["PreToolUse"][0];
        assert_eq!(pre["matcher"], "*", "PreToolUse keeps matcher");
        assert_eq!(pre["hooks"][0]["timeout"], 30);
        // PreInvocation has NO matcher (agy ignores it).
        let invoc = &obj["byoh-session-resume"]["PreInvocation"][0];
        assert!(
            invoc.get("matcher").is_none(),
            "PreInvocation must omit matcher"
        );
        // No description field (not in the agy schema).
        assert!(
            !v.to_string().contains("description"),
            "agy hooks must not carry a description field"
        );
    }

    #[test]
    fn render_polyglot_creates_host_symlinks() {
        let bundle = compile_profile(&confirmed_profile()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        render_target(&bundle, Target::All, dir.path()).unwrap();

        // Each host dir links skills + agents (always) + hooks.json (bundle has hooks).
        for host in [".claude", ".gemini", ".codex"] {
            let h = dir.path().join(host);
            assert!(h.join("skills").exists(), "{host}/skills must resolve");
            assert!(h.join("agents").exists(), "{host}/agents must resolve");
            assert!(
                h.join("hooks.json").exists(),
                "{host}/hooks.json must resolve (bundle has hooks)"
            );
        }
        // The links point to the right host hooks (distinct schemas).
        let claude_hooks = std::fs::read_to_string(dir.path().join(".claude/hooks.json")).unwrap();
        assert!(
            claude_hooks.contains("\"SessionStart\""),
            ".claude hooks must be the Claude schema"
        );
        let gemini_hooks = std::fs::read_to_string(dir.path().join(".gemini/hooks.json")).unwrap();
        assert!(
            gemini_hooks.contains("byoh-session-resume"),
            ".gemini hooks must be the agy schema"
        );
    }

    #[test]
    fn render_polyglot_readme_follows_language() {
        // Korean profile → Korean README (user doc); AI instructions stay English.
        let mut p = confirmed_profile();
        p.language = "ko".into();
        let bundle = compile_profile(&p).unwrap();
        let dir = tempfile::tempdir().unwrap();
        render_target(&bundle, Target::All, dir.path()).unwrap();
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

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
        crate::store::write_file(
            out,
            "hooks.json",
            &pretty(&hooks_json(bundle, "${PLUGIN_ROOT}", false)),
        )?;
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
    Ok(())
}

// ─── common builders ────────────────────────────────────────────────────────

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

/// `mcp_config.json` (shared MCP config): { mcpServers: { name: {command, args} } }.
fn mcp_servers(bundle: &HarnessBundle) -> Value {
    let servers: Vec<(String, Value)> = bundle
        .mcp_tools
        .iter()
        .map(|t| {
            (
                t.name.clone(),
                json!({ "command": format!("byoh-{}", t.name), "args": [] }),
            )
        })
        .collect();
    let map = serde_json::Map::from_iter(servers);
    json!({ "mcpServers": Value::Object(map) })
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

    // hooks.json at the plugin root (NOT a hooks/ subdir).
    if !bundle.hooks.is_empty() {
        crate::store::write_file(
            out,
            "hooks.json",
            &pretty(&hooks_json(bundle, "${PLUGIN_ROOT}", false)),
        )?;
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
    let targets_line = if target == Target::All {
        "polyglot (Claude Code + Codex + Antigravity)".to_string()
    } else {
        format!("target `{}`", target.as_str())
    };
    let body = format!(
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
    );
    crate::store::write_file(out, "README.md", &body)?;
    Ok(())
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

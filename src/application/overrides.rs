//! Profile-scoped overrides — the LLM-authored content layer.
//!
//! When `build` classifies a skill as `skeleton` (no preset matched), the
//! `byoh-guide` agent authors real `Process`/`Anti-Rationalization`/`Evidence`/
//! `Red Flags` content via the `author_skill` MCP tool, which writes it to the
//! profile's overlay directory. `apply_profile_overrides` reads those overlays
//! and replaces the skeleton body in the bundle, so authored content persists
//! across rebuilds (defect-3 fix) instead of being silently overwritten.
//!
//! Resolution order: **preset → overlay → skeleton**.
//! - Preset injection runs first (`synthesize` step 4); overlay runs after
//!   (this module, called right after `synthesize`).
//! - An overlay for a preset-matched skill wins (explicit user intent); the
//!   collision is recorded in [`OverrideReport::collisions`] so the agent can
//!   tell the user their override replaced a vetted preset body.
//! - Safety-gate skills (`critic`/`seesaw`/`stagnation`) are REFUSED as
//!   overlays — gate integrity is a Rust invariant, not LLM-editable.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::compiler::is_skeleton_body;
use crate::deploy::presets::parse_frontmatter;
use crate::domain::bundle::{AgentSpec, HarnessBundle, HookSpec, SkillSpec};
use crate::domain::genre::SafetyGate;
use crate::security::mask;
use crate::store::{profiles_root_in, sanitize_slug};

/// Prefix every skeleton DOC body starts with. The 5 doc emitters
/// (`root_agents_md`, `docs_guide_en`/`ko`, `readme_en`/`ko`) produce skeleton
/// docs beginning with this; `is_skeleton_doc` detects them. Kept distinct from
/// the skill skeleton prefix (`compiler::render::SKELETON_BODY_PREFIX`) so the
/// two classifiers don't cross-fire.
pub const SKELETON_DOC_PREFIX: &str = "<!-- byoh-skeleton-doc -->\n";

/// True when a doc body is the placeholder skeleton emitted by a Rust doc
/// generator. Used by `render_target` to decide whether to prefer an overlay.
pub fn is_skeleton_doc(body: &str) -> bool {
    body.starts_with(SKELETON_DOC_PREFIX)
}

/// The set of skill ids that are mandatory safety gates and therefore NOT
/// overridable — overlay authoring for these is refused by `author_skill` and
/// ignored here as a defense-in-depth belt to the suspenders.
fn is_safety_gate(id: &str) -> bool {
    SafetyGate::ALL.iter().any(|g| g.as_str() == id)
}

/// A skill id is overridable iff it is NOT a safety-gate skill.
pub fn is_overridable_skill(id: &str) -> bool {
    !is_safety_gate(id)
}

/// Result of applying profile overrides — stashed in `bundle.config.extra` so
/// `build_sync` can surface `authored_skills`/`authored_docs` without re-walking
/// the overlay directory.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OverrideReport {
    /// Skill ids whose skeleton body was replaced by an overlay.
    pub authored_skills: Vec<String>,
    /// Agent ids whose body was replaced by an overlay.
    pub authored_agents: Vec<String>,
    /// Doc ids (e.g. `README.en`, `getting-started.ko`) authored as overlays.
    pub authored_docs: Vec<String>,
    /// Hook ids enabled from the curated `registry/hooks/` templates. Each is a
    /// declarative `spec:<id>` reference seeded with `HOOK_REQUIRED_FIELDS` so it
    /// passes the static gate; never an executable command.
    pub enabled_hooks: Vec<String>,
    /// `(skill_id, reason)` pairs where an overlay was refused (e.g. a safety
    /// gate) — surfaced so the agent can tell the user the override was ignored.
    pub refused: Vec<(String, String)>,
    /// Skill ids where an overlay replaced a PRESET-matched body (not just a
    /// skeleton). Explicit user intent, but worth flagging.
    pub collisions: Vec<String>,
}

/// The override root for a profile: `<home>/profiles/<slug>/overrides`.
/// Slug-sanitized so a hostile slug can't escape the profiles root.
pub fn overrides_dir(home: &Path, slug: &str) -> Result<PathBuf, String> {
    let clean = sanitize_slug(slug).map_err(|e| e.to_string())?;
    Ok(profiles_root_in(home).join(clean).join("overrides"))
}

/// Apply profile overrides to an already-synthesized bundle in place.
///
/// Reads `overrides/{skills,agents,docs}/*.md` and replaces skeleton bodies
/// (or, for collisions, preset bodies) with the authored overlay content.
/// Safety-gate skills are never overridden. Overlay content is `mask()`-ed on
/// read as defense in depth (the `author_skill` writer also masks on write).
pub fn apply_profile_overrides(
    home: &Path,
    slug: &str,
    bundle: &mut HarnessBundle,
) -> Result<OverrideReport, String> {
    let mut report = OverrideReport::default();
    let root = overrides_dir(home, slug)?;
    if !root.exists() {
        return Ok(report);
    }

    // Skills — replace skeleton (or colliding preset) bodies.
    let skills_dir = root.join("skills");
    if skills_dir.is_dir() {
        for entry in std::fs::read_dir(&skills_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let id = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            if is_safety_gate(&id) {
                report
                    .refused
                    .push((id, "safety-gate skills are not overridable".into()));
                continue;
            }
            let raw = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let masked = mask(&raw);
            let (name, description, body) = parse_frontmatter(&masked, &id);
            if let Some(skill) = bundle.skills.iter_mut().find(|s| s.id == id) {
                let was_skeleton = is_skeleton_body(&skill.body_markdown);
                // Overlay wins regardless; record whether it replaced a real
                // preset body (collision) or just a skeleton.
                skill.body_markdown = body;
                if !name.is_empty() {
                    skill.name = name;
                }
                if !description.is_empty() {
                    skill.description = description;
                }
                report.authored_skills.push(id.clone());
                if !was_skeleton {
                    report.collisions.push(id);
                }
            } else {
                // Overlay for a skill not in the bundle — clone it in (Ring 2).
                bundle.skills.push(SkillSpec {
                    id: id.clone(),
                    ring: crate::domain::bundle::Ring::Ring2,
                    name,
                    description,
                    body_markdown: body,
                    pipeline: None,
                    order: None,
                });
                report.authored_skills.push(id);
            }
        }
    }

    // Agents — replace bodies.
    let agents_dir = root.join("agents");
    if agents_dir.is_dir() {
        for entry in std::fs::read_dir(&agents_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let id = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            let raw = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let masked = mask(&raw);
            let (name, description, body) = parse_frontmatter(&masked, &id);
            if let Some(agent) = bundle.agents.iter_mut().find(|a| a.id == id) {
                agent.body_markdown = body;
                if !name.is_empty() {
                    agent.name = name;
                }
                if !description.is_empty() {
                    agent.description = description;
                }
            } else {
                bundle.agents.push(AgentSpec {
                    id: id.clone(),
                    name,
                    description,
                    body_markdown: body,
                    tools: None,
                });
            }
            report.authored_agents.push(id);
        }
    }

    // Docs — read every override; the renderer decides per-doc whether to use
    // it (only when the generated doc is still a skeleton). Record them so
    // `build_sync` can report which docs are authored.
    let docs_dir = root.join("docs");
    if docs_dir.is_dir() {
        for entry in std::fs::read_dir(&docs_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            report.authored_docs.push(stem);
        }
    }

    // Hooks — each enabled pointer (a `<id>.toml` in overrides/hooks/) names a
    // curated template from `registry/hooks/`. We load the template (never trust
    // a command from the overlay itself) and append a declarative `HookSpec`.
    // The static gate then enforces HOOK_REQUIRED_FIELDS on the result.
    let hooks_dir = root.join("hooks");
    if hooks_dir.is_dir() {
        for entry in std::fs::read_dir(&hooks_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }
            let hook_id = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            match load_hook_template(&hook_id) {
                Ok(spec) => {
                    // De-duplicate by event+command so re-enabling doesn't pile up.
                    let already = bundle.hooks.iter().any(|h| h.command == spec.command);
                    if !already {
                        bundle.hooks.push(spec);
                    }
                    report.enabled_hooks.push(hook_id);
                }
                Err(reason) => {
                    report.refused.push((hook_id, reason));
                }
            }
        }
    }

    Ok(report)
}

/// Parsed `registry/hooks/<id>.toml` template.
#[derive(Debug, Deserialize)]
struct HookTemplate {
    event: String,
    command: String,
    reads: Vec<String>,
}

/// The curated hook templates, embedded into the binary at compile time so a
/// `cargo install`-ed binary resolves them regardless of cwd (no runtime disk
/// read of `registry/hooks/`). Add a template file under `registry/hooks/` and
/// a matching `include_str!` row here; that is the ONLY way a new hook id
/// becomes enable-able — `enable_hook` refuses anything not in this map.
const HOOK_TEMPLATES: &[(&str, &str)] = &[
    (
        "pre-commit-lint",
        include_str!("../../registry/hooks/pre-commit-lint.toml"),
    ),
    (
        "session-start-resume",
        include_str!("../../registry/hooks/session-start-resume.toml"),
    ),
];

/// Load a curated hook template by id. Returns an error string (not a ByohError)
/// so `apply_profile_overrides` can record it in `refused` without aborting the
/// whole override pass. The template's `reads` is unioned with
/// `HOOK_REQUIRED_FIELDS` so the static gate always passes.
pub fn load_hook_template(hook_id: &str) -> Result<HookSpec, String> {
    // The hook id must be a simple identifier (defense in depth; the curated
    // map lookup would reject traversal anyway).
    if !hook_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!("hook id '{hook_id}' is not a simple identifier"));
    }
    let raw = HOOK_TEMPLATES
        .iter()
        .find(|(id, _)| *id == hook_id)
        .map(|(_, body)| *body)
        .ok_or_else(|| format!("hook '{hook_id}' is not in the curated registry/hooks set"))?;
    let tpl: HookTemplate =
        toml::from_str(raw).map_err(|e| format!("curated hook '{hook_id}' malformed: {e}"))?;
    // Seed the required fields (union, preserving template extras). The static
    // gate enforces HOOK_REQUIRED_FIELDS on every hook — seeding them here means
    // a freshly-enabled hook passes without the agent knowing the field names.
    let mut reads = tpl.reads;
    for field in crate::domain::bundle::HOOK_REQUIRED_FIELDS {
        if !reads.iter().any(|r| r == field) {
            reads.push(field.to_string());
        }
    }
    Ok(HookSpec {
        event: tpl.event,
        command: tpl.command,
        reads,
    })
}

/// Read an overlay doc body by `<id>.<lang>.md` (e.g. `README.en.md`).
/// Returns the masked content if present, else `None`.
pub fn read_doc_override(home: &Path, slug: &str, id: &str, lang: &str) -> Option<String> {
    let root = overrides_dir(home, slug).ok()?;
    let name = format!("{id}.{lang}.md");
    let path = root.join("docs").join(name);
    let raw = std::fs::read_to_string(&path).ok()?;
    Some(mask(&raw))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::set_home_override;

    fn confirmed_dev_profile(slug: &str) -> crate::domain::profile::UserProfile {
        use crate::domain::genre::Genre;
        use crate::domain::profile::{GenreConfidence, ProfileStatus, UserProfile};
        let mut p = UserProfile::new_draft(slug, "en");
        p.candidates.identity.genre = Some(GenreConfidence {
            value: Genre::Developer,
            confidence: 1.0,
            provenance: vec![],
        });
        p.status = ProfileStatus::Confirmed;
        p
    }

    #[test]
    fn overlay_fills_a_skeleton_skill() {
        let dir = tempfile::tempdir().unwrap();
        set_home_override(Some(dir.path().to_path_buf()));
        let profile = confirmed_dev_profile("dev1");
        crate::store::write_profile_in(dir.path(), &profile).unwrap();
        let (mut bundle, _plan) = crate::application::synthesize(&profile).unwrap();
        // The `spec` skill starts as a skeleton in the dev template.
        let spec = bundle
            .skills
            .iter()
            .find(|s| s.id == "spec")
            .expect("dev template has a spec skill");
        assert!(is_skeleton_body(&spec.body_markdown));

        // Author an overlay for it.
        let skills = overrides_dir(dir.path(), "dev1").unwrap().join("skills");
        std::fs::create_dir_all(&skills).unwrap();
        std::fs::write(
            skills.join("spec.md"),
            "---\nname: spec\ndescription: Real router.\n---\n\n## Process\nReal content.\n",
        )
        .unwrap();

        let report = apply_profile_overrides(dir.path(), "dev1", &mut bundle).unwrap();
        assert!(report.authored_skills.contains(&"spec".to_string()));
        let spec = bundle.skills.iter().find(|s| s.id == "spec").unwrap();
        assert!(!is_skeleton_body(&spec.body_markdown));
        assert!(spec.body_markdown.contains("Real content"));
        set_home_override(None);
    }

    #[test]
    fn overlay_refuses_safety_gate_skills() {
        let dir = tempfile::tempdir().unwrap();
        set_home_override(Some(dir.path().to_path_buf()));
        let profile = confirmed_dev_profile("dev2");
        crate::store::write_profile_in(dir.path(), &profile).unwrap();
        let (mut bundle, _plan) = crate::application::synthesize(&profile).unwrap();

        let skills = overrides_dir(dir.path(), "dev2").unwrap().join("skills");
        std::fs::create_dir_all(&skills).unwrap();
        std::fs::write(
            skills.join("critic.md"),
            "---\nname: critic\ndescription: Tampered.\n---\n\nHacked gate.\n",
        )
        .unwrap();

        let report = apply_profile_overrides(dir.path(), "dev2", &mut bundle).unwrap();
        assert!(!report.authored_skills.contains(&"critic".to_string()));
        assert!(report.refused.iter().any(|(id, _)| id == "critic"));
        // Gate body untouched.
        let critic = bundle
            .skills
            .iter()
            .find(|s| s.id == "critic")
            .expect("critic gate present");
        assert!(!critic.body_markdown.contains("Hacked gate"));
        set_home_override(None);
    }

    #[test]
    fn overlay_persists_across_rebuilds() {
        // Defect-3 regression: an authored skill must survive a second build.
        let dir = tempfile::tempdir().unwrap();
        set_home_override(Some(dir.path().to_path_buf()));
        let profile = confirmed_dev_profile("dev3");
        crate::store::write_profile_in(dir.path(), &profile).unwrap();

        let skills = overrides_dir(dir.path(), "dev3").unwrap().join("skills");
        std::fs::create_dir_all(&skills).unwrap();
        std::fs::write(
            skills.join("spec.md"),
            "---\nname: spec\ndescription: Persistent.\n---\n\n## Process\nLives across rebuilds.\n",
        )
        .unwrap();

        // First build + override.
        let (mut bundle1, _) = crate::application::synthesize(&profile).unwrap();
        apply_profile_overrides(dir.path(), "dev3", &mut bundle1).unwrap();
        // Second build — synthesize resets the bundle, but overlay re-applies.
        let (mut bundle2, _) = crate::application::synthesize(&profile).unwrap();
        apply_profile_overrides(dir.path(), "dev3", &mut bundle2).unwrap();

        let spec2 = bundle2.skills.iter().find(|s| s.id == "spec").unwrap();
        assert!(spec2.body_markdown.contains("Lives across rebuilds"));
        set_home_override(None);
    }
}

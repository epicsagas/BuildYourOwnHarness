//! Render a ConfirmedProfile + genre template into a HarnessBundle.

use sha2::{Digest, Sha256};

use crate::domain::bundle::{
    BundleConfig, BundleVersion, DependencyPin, HarnessBundle, HookSpec, McpTool, Ring, SkillSpec,
};
use crate::domain::genre::Genre;
use crate::domain::profile::UserProfile;
use crate::templates::TemplateLibrary;

/// Compile a confirmed profile into a bundle. Does NOT run the validation or
/// dry-run gates — callers invoke [`static_gate`] / [`dry_run`] explicitly.
pub fn compile_profile(profile: &UserProfile) -> crate::domain::Result<HarnessBundle> {
    let genre = profile
        .candidates
        .identity
        .genre
        .as_ref()
        .map(|g| g.value)
        .ok_or(crate::domain::ByohError::MissingTruth {
            field: "candidates.identity.genre",
        })?;

    let lib = TemplateLibrary::new();
    let template = lib.get(genre);
    let merged = crate::templates::inherit::merge_child_into_base(
        &crate::templates::base::base_template(),
        &template,
    );

    // Render skills per ring.
    let mut skills: Vec<SkillSpec> = Vec::new();
    for id in &merged.rings.ring1_pipeline {
        skills.push(render_skill(id, Ring::Ring1, genre));
    }
    for id in &merged.rings.ring2_quality {
        skills.push(render_skill(id, Ring::Ring2, genre));
    }
    for id in &merged.rings.ring3_evolution {
        skills.push(render_skill(id, Ring::Ring3, genre));
    }

    // Render hooks (Ring 0).
    let hooks: Vec<HookSpec> = merged
        .rings
        .ring0_hooks
        .iter()
        .map(|id| render_hook(id))
        .collect();

    // Render MCP tools (B4 self-describing).
    let mcp_tools: Vec<McpTool> = merged
        .tool_blueprints
        .iter()
        .map(|bp| render_mcp_tool(bp, genre))
        .collect();

    let config = BundleConfig {
        slug: profile.slug.clone(),
        genre,
        profile_version: profile.profile_version.clone(),
        depends_on: default_dependencies(),
        extra: std::collections::BTreeMap::new(),
    };

    let source_profile_hash = hash_profile(profile);

    Ok(HarnessBundle {
        config,
        version: BundleVersion::new(1, 0, 0),
        genre,
        slug: profile.slug.clone(),
        skills,
        hooks,
        mcp_tools,
        agents: crate::templates::agents::genre_agents(genre),
        safety_gates: merged.rings.ring3_evolution.clone(),
        stagnation_limit: merged.evolution.stagnation_limit,
        improvement_threshold: merged.evolution.improvement_threshold,
        source_profile_hash,
        language: profile.language.clone(),
    })
}

fn default_dependencies() -> Vec<DependencyPin> {
    vec![
        DependencyPin {
            id: "obsidian-forge".into(),
            min_version: "0.1.0".into(),
        },
        DependencyPin {
            id: "alcove".into(),
            min_version: "0.1.0".into(),
        },
        DependencyPin {
            id: "epic-harness".into(),
            min_version: "0.1.0".into(),
        },
    ]
}

fn render_skill(id: &str, ring: Ring, genre: Genre) -> SkillSpec {
    let (name, description, body) = skill_doc(id, ring, genre);
    SkillSpec {
        id: id.to_string(),
        ring,
        name,
        description,
        body_markdown: body,
        pipeline: None,
        order: None,
    }
}

/// SKILL.md-style doc: frontmatter (name, description) + 4-section body
/// (Process / Anti-Rationalization / Evidence / Red Flags) per epic-harness.
fn skill_doc(id: &str, _ring: Ring, genre: Genre) -> (String, String, String) {
    let name = id.to_string();
    let description = format!("{id} skill for the {genre} genre harness.");
    let body = format!(
        "---\nname: {id}\ndescription: {description}\n---\n\n\
         ## Process\nThe {id} skill executes its {genre} pipeline step.\n\n\
         ## Anti-Rationalization\nDo not skip steps; evidence required.\n\n\
         ## Evidence\nOutputs are persisted as files for inspection.\n\n\
         ## Red Flags\n- Empty output\n- Skipped validation\n"
    );
    (name, description, body)
}

fn render_hook(id: &str) -> HookSpec {
    let event = match id {
        "session_start_resume" => "SessionStart",
        "pre_tool_use_guard" => "PreToolUse",
        "post_tool_use_read_compress" => "PostToolUse",
        "post_tool_use_tone_spellcheck" => "PostToolUse",
        "session_end_observe" => "SessionEnd",
        other => other,
    };
    HookSpec {
        event: event.to_string(),
        command: format!("byoh hook {id}"),
        reads: crate::domain::bundle::HOOK_REQUIRED_FIELDS
            .iter()
            .map(|s| s.to_string())
            .collect(),
    }
}

fn render_mcp_tool(bp: &str, genre: Genre) -> McpTool {
    // B4: self-describing description that also tells the agent WHEN to call the
    // tool. BYOH ships no embedded search backend — these genre search tools are
    // wired to the user's own knowledge base (e.g. alcove, a local doc server, or
    // a project index the host already runs).
    let description = match (bp, genre) {
        ("search_draft_continuity", Genre::Creator) => {
            "Search the novel draft for character/setting/plot continuity. Call this when the \
             user asks about foreshadowing or a character's arc. Backed by the user's knowledge \
             base (e.g. alcove)."
                .to_string()
        }
        ("search_code", Genre::Developer) => {
            "Search the codebase for symbols/definitions. Call this when the user asks 'where is \
             X defined'. Backed by the user's knowledge base (e.g. alcove or a project index)."
                .to_string()
        }
        ("search_citations", Genre::Researcher) => {
            "Search indexed citations for a claim. Call this when verifying a source. Backed by \
             the user's knowledge base (e.g. alcove)."
                .to_string()
        }
        ("search_decisions", Genre::Business) => {
            "Search the decision log. Call this when reviewing past ROI decisions. Backed by the \
             user's knowledge base (e.g. alcove)."
                .to_string()
        }
        _ => format!("Tool blueprint {bp} for {genre}. Backed by the user's knowledge base."),
    };
    McpTool {
        name: bp.to_string(),
        description,
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" }
            },
            "required": ["query"]
        }),
    }
}

fn hash_profile(p: &UserProfile) -> String {
    let yaml = serde_yaml::to_string(p).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(yaml.as_bytes());
    let digest = hasher.finalize();
    format!("sha256:{}", hex_encode(&digest))
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::{GenreConfidence, ProfileStatus};

    fn confirmed() -> UserProfile {
        let mut p = UserProfile::new_draft("dev1", "en");
        p.candidates.identity.genre = Some(GenreConfidence {
            value: Genre::Developer,
            confidence: 1.0,
            provenance: vec!["wizard".into()],
        });
        p.status = ProfileStatus::Confirmed;
        p
    }

    #[test]
    fn compile_produces_all_bundle_components() {
        // AC6: config, skills, mcp tools, hooks, evolution_policy all present.
        let p = confirmed();
        let b = compile_profile(&p).unwrap();
        assert_eq!(b.genre, Genre::Developer);
        assert!(!b.skills.is_empty());
        assert!(!b.hooks.is_empty());
        assert!(b.skills.iter().any(|s| s.ring == Ring::Ring1));
        assert!(b.skills.iter().any(|s| s.ring == Ring::Ring2));
        assert!(b.skills.iter().any(|s| s.ring == Ring::Ring3));
        assert!(b.source_profile_hash.starts_with("sha256:"));
        assert!(b.config.depends_on.len() >= 3);
    }

    #[test]
    fn compiled_skills_have_skillmd_format() {
        let p = confirmed();
        let b = compile_profile(&p).unwrap();
        let spec = b.skills.iter().find(|s| s.id == "spec").unwrap();
        assert!(spec.body_markdown.contains("name: spec"));
        assert!(spec.body_markdown.contains("## Process"));
        assert!(spec.body_markdown.contains("## Red Flags"));
    }

    #[test]
    fn hooks_declare_required_hookinput_fields() {
        let p = confirmed();
        let b = compile_profile(&p).unwrap();
        for h in &b.hooks {
            for f in crate::domain::bundle::HOOK_REQUIRED_FIELDS {
                assert!(
                    h.reads.contains(&f.to_string()),
                    "hook {} missing {}",
                    h.event,
                    f
                );
            }
        }
    }

    #[test]
    fn mcp_tools_are_self_describing_and_well_formed() {
        let p = confirmed();
        let b = compile_profile(&p).unwrap();
        for t in &b.mcp_tools {
            assert!(t.is_well_formed(), "tool {} not well-formed", t.name);
            assert!(t.description.len() > 20);
        }
    }
}

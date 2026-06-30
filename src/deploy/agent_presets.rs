//! Local vetted-agent presets — mirrors the skill-preset pattern
//! ([`crate::deploy::presets`]) for agents (Issue #6).
//!
//! Where skill presets enrich a bundle's *skills*, agent presets enrich a
//! bundle's *agents*. A verified agent body can be *cloned* into a bundle, or
//! an existing genre-default agent (see [`crate::templates::agents`]) can be
//! *augmented* with the richer preset body. Presets live under
//! `registry/agents/<genre>/<agent_id>.md` and are embedded at compile time via
//! `include_str!` — zero runtime file/network dependency (spec §Out).
//!
//! [`inject_agent`] dedupes by agent `id`: an existing agent is *augmented*
//! (body + description + tools replaced); a missing one is *cloned* into the
//! bundle's agent list. Generate (genre defaults) and clone coexist — they
//! never duplicate.

use crate::Result;
use crate::domain::bundle::{AgentSpec, HarnessBundle};
use crate::domain::error::ByohError;
use crate::domain::genre::Genre;

/// Raw agent-preset bodies, keyed by `(genre, agent_id)`. Embedded at compile time.
fn raw_agent_preset(genre: Genre, agent_id: &str) -> Result<&'static str> {
    use Genre::*;
    Ok(match (genre, agent_id) {
        (Developer, "code-reviewer") => {
            include_str!("../../registry/agents/developer/code-reviewer.md")
        }
        (Developer, "debugger") => include_str!("../../registry/agents/developer/debugger.md"),
        (Developer, "tech-debt-auditor") => {
            include_str!("../../registry/agents/developer/tech-debt-auditor.md")
        }
        (Creator, "draft-writer") => {
            include_str!("../../registry/agents/creator/draft-writer.md")
        }
        (Creator, "consistency-editor") => {
            include_str!("../../registry/agents/creator/consistency-editor.md")
        }
        (Researcher, "research-analyst") => {
            include_str!("../../registry/agents/researcher/research-analyst.md")
        }
        (Business, "decision-analyst") => {
            include_str!("../../registry/agents/business/decision-analyst.md")
        }
        _ => {
            return Err(ByohError::Schema(format!(
                "no agent preset for genre '{}' agent '{}'",
                genre.as_str(),
                agent_id
            )));
        }
    })
}

/// Resolve an agent preset's full markdown body (frontmatter + body).
pub fn agent_body(genre: Genre, agent_id: &str) -> Result<String> {
    Ok(raw_agent_preset(genre, agent_id)?.to_string())
}

/// Searchable metadata for an agent preset — keywords the synthesis engine
/// matches against profile-derived tags, plus the owning genre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentPresetMeta {
    pub genre: Genre,
    pub agent_id: &'static str,
    pub keywords: &'static [&'static str],
}

/// The full local agent-preset catalog. Mirrors [`preset_catalog`] for skills.
/// This is the synthesis engine's agent "registry": every embedded agent preset
/// with the keyword tags it should match on. Community agents are OUT of scope
/// (offline-vetted local only).
pub fn agent_catalog() -> &'static [AgentPresetMeta] {
    use Genre::*;
    &[
        AgentPresetMeta {
            genre: Developer,
            agent_id: "code-reviewer",
            keywords: &[
                "code",
                "review",
                "merge",
                "diff",
                "security",
                "developer",
                "quality",
                "ship",
            ],
        },
        AgentPresetMeta {
            genre: Developer,
            agent_id: "debugger",
            keywords: &[
                "debug",
                "bug",
                "root-cause",
                "error",
                "stack",
                "developer",
                "code",
                "reproduce",
            ],
        },
        AgentPresetMeta {
            genre: Developer,
            agent_id: "tech-debt-auditor",
            keywords: &[
                "debt",
                "refactor",
                "todo",
                "fixme",
                "tech",
                "legacy",
                "developer",
                "code",
            ],
        },
        AgentPresetMeta {
            genre: Creator,
            agent_id: "draft-writer",
            keywords: &[
                "draft", "write", "writing", "chapter", "scene", "creator", "story", "content",
            ],
        },
        AgentPresetMeta {
            genre: Creator,
            agent_id: "consistency-editor",
            keywords: &[
                "edit",
                "consistency",
                "continuity",
                "canon",
                "creator",
                "proofread",
                "writing",
            ],
        },
        AgentPresetMeta {
            genre: Researcher,
            agent_id: "research-analyst",
            keywords: &[
                "research",
                "evidence",
                "citation",
                "synthesis",
                "source",
                "analysis",
                "literature",
            ],
        },
        AgentPresetMeta {
            genre: Business,
            agent_id: "decision-analyst",
            keywords: &[
                "decision",
                "roi",
                "opportunity",
                "business",
                "strategy",
                "tradeoff",
                "prioritize",
            ],
        },
    ]
}

/// Does an agent preset match any of the given tags (case-insensitive substring)?
/// Identical predicate to [`preset_matches`] for skills.
pub fn agent_matches(meta: &AgentPresetMeta, tags: &[String]) -> bool {
    tags.iter().any(|t| {
        let lower = t.to_lowercase();
        meta.keywords
            .iter()
            .any(|k| lower.contains(k) || k.contains(&lower))
    })
}

// NOTE: parse_frontmatter + agent_matches mirror deploy/presets.rs. Extract a
// shared module when a THIRD preset-style type is added (e.g. hooks/pipeline).
/// Parse minimal YAML frontmatter (`name:` / `description:`) + markdown body.
/// Returns `(name, description, body_markdown)`. Falls back to the agent_id and
/// an empty description if frontmatter is absent. Shared shape with the skill
/// preset parser.
fn parse_frontmatter(raw: &str, fallback_id: &str) -> (String, String, String) {
    let mut name = fallback_id.to_string();
    let mut description = String::new();
    let mut body = raw.to_string();

    let trimmed = raw.trim_start();
    if let Some(rest) = trimmed.strip_prefix("---") {
        if let Some(end) = rest.find("\n---") {
            let fm = &rest[..end];
            for line in fm.lines() {
                let line = line.trim();
                if let Some(v) = line.strip_prefix("name:") {
                    name = v.trim().trim_matches('"').to_string();
                } else if let Some(v) = line.strip_prefix("description:") {
                    description = v.trim().trim_matches('"').to_string();
                }
            }
            if let Some(after) = rest[end..].strip_prefix("\n---") {
                body = after.trim_start().to_string();
            }
        }
    }
    (name, description, body)
}

/// Inject a vetted agent preset into a compiled bundle's agent list.
///
/// - **Augment**: if an agent with `agent_id` already exists (e.g. a
///   genre-default `debugger`), replace its `body_markdown` / `name` /
///   `description` with the richer preset. Agent count is unchanged.
/// - **Clone**: otherwise append a new `AgentSpec`. Cloned agents inherit the
///   preset's name/description/body; `tools` is left `None` (preset bodies are
///   target-agnostic — the renderer omits the `tools:` frontmatter line, so the
///   host applies its default tool set, broader than the curated list an
///   augmented genre-default agent keeps).
///
/// Either way the result is deduplicated by `id` — generate + clone coexist.
pub fn inject_agent(bundle: &mut HarnessBundle, genre: Genre, agent_id: &str) -> Result<()> {
    let raw = raw_agent_preset(genre, agent_id)?;
    let (name, description, body) = parse_frontmatter(raw, agent_id);

    if let Some(existing) = bundle.agents.iter_mut().find(|a| a.id == agent_id) {
        existing.name = name;
        existing.description = description;
        existing.body_markdown = body;
    } else {
        bundle.agents.push(AgentSpec {
            id: agent_id.to_string(),
            name,
            description,
            body_markdown: body,
            tools: None,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::compile_profile;
    use crate::domain::profile::{ProfileStatus, UserProfile};

    fn confirmed_developer_profile() -> UserProfile {
        let mut p = UserProfile::new_draft("dev", "en");
        p.candidates.identity.genre = Some(crate::domain::profile::GenreConfidence {
            value: Genre::Developer,
            confidence: 1.0,
            provenance: vec![],
        });
        p.status = ProfileStatus::Confirmed;
        p
    }

    #[test]
    fn inject_augments_existing_agent_body() {
        let p = confirmed_developer_profile();
        let mut bundle = compile_profile(&p).unwrap();
        // genre_defaults give a developer `debugger` → augment.
        assert!(
            bundle.agents.iter().any(|a| a.id == "debugger"),
            "developer bundle should have a default debugger to augment"
        );
        let before = bundle.agents.len();
        inject_agent(&mut bundle, Genre::Developer, "debugger").unwrap();
        assert_eq!(bundle.agents.len(), before, "augment must not add an agent");
        let dbg = bundle.agents.iter().find(|a| a.id == "debugger").unwrap();
        assert!(
            dbg.body_markdown.contains("Reproduce"),
            "preset body should replace the generated stub"
        );
        assert_eq!(dbg.name, "Debugger", "name comes from preset frontmatter");
    }

    #[test]
    fn inject_clones_new_agent() {
        let p = confirmed_developer_profile();
        let mut bundle = compile_profile(&p).unwrap();
        // `tech-debt-auditor` is not a genre default → clone path.
        assert!(
            !bundle.agents.iter().any(|a| a.id == "tech-debt-auditor"),
            "tech-debt-auditor should not be a genre default"
        );
        let before = bundle.agents.len();
        inject_agent(&mut bundle, Genre::Developer, "tech-debt-auditor").unwrap();
        assert_eq!(bundle.agents.len(), before + 1, "clone must add one agent");
        let tda = bundle
            .agents
            .iter()
            .find(|a| a.id == "tech-debt-auditor")
            .unwrap();
        assert_eq!(tda.name, "Tech Debt Auditor");
        assert!(tda.body_markdown.to_lowercase().contains("severity"));
        assert!(tda.tools.is_none(), "cloned agent has no tools by default");
    }

    #[test]
    fn inject_unknown_agent_errors() {
        let p = confirmed_developer_profile();
        let mut bundle = compile_profile(&p).unwrap();
        let err = inject_agent(&mut bundle, Genre::Developer, "nonexistent").unwrap_err();
        assert!(matches!(err, ByohError::Schema(_)), "got {err:?}");
    }

    #[test]
    fn inject_is_idempotent() {
        // Injecting twice must not duplicate (second is an augment of the first).
        let p = confirmed_developer_profile();
        let mut bundle = compile_profile(&p).unwrap();
        inject_agent(&mut bundle, Genre::Developer, "tech-debt-auditor").unwrap();
        let after_one = bundle.agents.len();
        inject_agent(&mut bundle, Genre::Developer, "tech-debt-auditor").unwrap();
        assert_eq!(bundle.agents.len(), after_one, "second inject dedupes");
    }

    #[test]
    fn agent_body_round_trips() {
        let body = agent_body(Genre::Business, "decision-analyst").unwrap();
        assert!(body.contains("opportunity"));
    }

    #[test]
    fn catalog_lists_all_embedded_agents() {
        // Every catalog entry must resolve a body (catch a stale catalog vs files).
        for meta in agent_catalog() {
            assert!(raw_agent_preset(meta.genre, meta.agent_id).is_ok());
        }
        assert!(agent_catalog().len() >= 7, "7 embedded agent presets");
    }

    #[test]
    fn agent_matches_by_keyword() {
        let meta = agent_catalog()
            .iter()
            .find(|m| m.agent_id == "research-analyst")
            .unwrap();
        assert!(agent_matches(meta, &["evidence".into(), "fishing".into()]));
        assert!(!agent_matches(meta, &["fishing".into()]));
    }

    #[test]
    fn inject_researcher_and_business_agents() {
        use crate::domain::profile::GenreConfidence;
        // Researcher bundle gets research-analyst (already default → augment).
        let mut rp = UserProfile::new_draft("res", "en");
        rp.candidates.identity.genre = Some(GenreConfidence {
            value: Genre::Researcher,
            confidence: 1.0,
            provenance: vec![],
        });
        rp.status = ProfileStatus::Confirmed;
        let mut rbundle = compile_profile(&rp).unwrap();
        let rbefore = rbundle.agents.len();
        inject_agent(&mut rbundle, Genre::Researcher, "research-analyst").unwrap();
        assert_eq!(rbundle.agents.len(), rbefore, "augment, not clone");

        // Business bundle gets decision-analyst (already default → augment).
        let mut bp = UserProfile::new_draft("biz", "en");
        bp.candidates.identity.genre = Some(GenreConfidence {
            value: Genre::Business,
            confidence: 1.0,
            provenance: vec![],
        });
        bp.status = ProfileStatus::Confirmed;
        let mut bbundle = compile_profile(&bp).unwrap();
        let bbefore = bbundle.agents.len();
        inject_agent(&mut bbundle, Genre::Business, "decision-analyst").unwrap();
        assert_eq!(bbundle.agents.len(), bbefore, "augment, not clone");
        let da = bbundle
            .agents
            .iter()
            .find(|a| a.id == "decision-analyst")
            .unwrap();
        assert!(da.body_markdown.to_lowercase().contains("opportunity cost"));
    }
}

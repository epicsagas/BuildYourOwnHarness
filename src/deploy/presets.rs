//! Local vetted-skill presets — the "clone" path of BYOH's generate-or-clone model.
//!
//! Instead of (or in addition to) letting the compiler *generate* a skill from a
//! genre template, a verified skill body can be *cloned* into a bundle. Presets
//! live under `registry/presets/<genre>/<skill_id>.md` and are embedded at
//! compile time via `include_str!` — zero runtime file/network dependency
//! (spec §Out: no remote registry). Network git-clone is out of scope (PR #3).
//!
//! [`inject_preset`] dedupes by skill `id`: an existing skill (e.g. the base
//! template's `tdd`) is *augmented* (body replaced); a missing one is *cloned*
//! into Ring 2. Generate and clone coexist — they never duplicate.

use crate::domain::bundle::{HarnessBundle, Ring, SkillSpec};
use crate::domain::error::ByohError;
use crate::domain::genre::Genre;
use crate::Result;

/// Raw preset bodies, keyed by `(genre, skill_id)`. Embedded at compile time.
fn raw_preset(genre: Genre, skill_id: &str) -> Result<&'static str> {
    use Genre::*;
    Ok(match (genre, skill_id) {
        (Developer, "tdd") => include_str!("../../registry/presets/developer/tdd.md"),
        (Developer, "debug") => include_str!("../../registry/presets/developer/debug.md"),
        (Creator, "continuity") => {
            include_str!("../../registry/presets/creator/continuity.md")
        }
        (Researcher, "evidence") => {
            include_str!("../../registry/presets/researcher/evidence.md")
        }
        (Researcher, "reproducibility") => {
            include_str!("../../registry/presets/researcher/reproducibility.md")
        }
        (Business, "decision") => {
            include_str!("../../registry/presets/business/decision.md")
        }
        (Business, "plainlanguage") => {
            include_str!("../../registry/presets/business/plainlanguage.md")
        }
        _ => {
            return Err(ByohError::Schema(format!(
                "no preset for genre '{}' skill '{}'",
                genre.as_str(),
                skill_id
            )))
        }
    })
}

/// Resolve a preset's full markdown body (frontmatter + body).
pub fn preset_body(genre: Genre, skill_id: &str) -> Result<String> {
    Ok(raw_preset(genre, skill_id)?.to_string())
}

/// Searchable metadata for a preset — keywords the synthesis engine matches
/// against profile-derived tags, plus the owning genre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetMeta {
    pub genre: Genre,
    pub skill_id: &'static str,
    pub keywords: &'static [&'static str],
}

/// The full local preset catalog. This is the synthesis engine's "registry":
/// every embedded preset with the keyword tags it should match on. Community
/// presets are OUT of scope this orbit (offline-vetted local only).
pub fn preset_catalog() -> &'static [PresetMeta] {
    use Genre::*;
    &[
        PresetMeta {
            genre: Developer,
            skill_id: "tdd",
            keywords: &["test", "tdd", "quality", "red-green", "developer", "code"],
        },
        PresetMeta {
            genre: Developer,
            skill_id: "debug",
            keywords: &["debug", "bug", "root-cause", "error", "developer", "code"],
        },
        PresetMeta {
            genre: Creator,
            skill_id: "continuity",
            keywords: &[
                "continuity",
                "consistency",
                "writing",
                "story",
                "creator",
                "edit",
            ],
        },
        PresetMeta {
            genre: Researcher,
            skill_id: "evidence",
            keywords: &[
                "evidence", "research", "citation", "claim", "source", "analysis",
            ],
        },
        PresetMeta {
            genre: Researcher,
            skill_id: "reproducibility",
            keywords: &[
                "reproducibility",
                "reproducible",
                "seed",
                "pin",
                "research",
                "data",
            ],
        },
        PresetMeta {
            genre: Business,
            skill_id: "decision",
            keywords: &[
                "decision",
                "roi",
                "opportunity",
                "business",
                "strategy",
                "tradeoff",
            ],
        },
        PresetMeta {
            genre: Business,
            skill_id: "plainlanguage",
            keywords: &[
                "writing",
                "communication",
                "plain",
                "exec",
                "business",
                "audience",
            ],
        },
    ]
}

/// Does a preset match any of the given tags (case-insensitive substring)?
pub fn preset_matches(meta: &PresetMeta, tags: &[String]) -> bool {
    tags.iter().any(|t| {
        let lower = t.to_lowercase();
        meta.keywords
            .iter()
            .any(|k| lower.contains(k) || k.contains(&lower))
    })
}

/// Parse minimal YAML frontmatter (`name:` / `description:`) + markdown body.
/// Returns `(name, description, body_markdown)`. Falls back to the skill_id and
/// an empty description if frontmatter is absent.
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
            // Body = everything after the closing `---` fence.
            if let Some(after) = rest[end..].strip_prefix("\n---") {
                body = after.trim_start().to_string();
            }
        }
    }
    (name, description, body)
}

/// Inject a vetted preset skill into a compiled bundle's skill list.
///
/// - **Augment**: if a skill with `skill_id` already exists, replace its
///   `body_markdown` / `name` / `description` with the richer preset. Skill
///   count is unchanged.
/// - **Clone**: otherwise append a new `SkillSpec` in Ring 2 (quality).
///
/// Either way the result is deduplicated by `id` — generate + clone coexist.
pub fn inject_preset(bundle: &mut HarnessBundle, genre: Genre, skill_id: &str) -> Result<()> {
    let raw = raw_preset(genre, skill_id)?;
    let (name, description, body) = parse_frontmatter(raw, skill_id);

    if let Some(existing) = bundle.skills.iter_mut().find(|s| s.id == skill_id) {
        existing.name = name;
        existing.description = description;
        existing.body_markdown = body;
    } else {
        bundle.skills.push(SkillSpec {
            id: skill_id.to_string(),
            ring: Ring::Ring2,
            name,
            description,
            body_markdown: body,
            pipeline: None,
            order: None,
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
    fn inject_augments_existing_skill_body() {
        let p = confirmed_developer_profile();
        let mut bundle = compile_profile(&p).unwrap();
        let before = bundle.skills.len();
        // `tdd` already exists in the developer base template → augment.
        inject_preset(&mut bundle, Genre::Developer, "tdd").unwrap();
        assert_eq!(bundle.skills.len(), before, "augment must not add a skill");
        let tdd = bundle.skills.iter().find(|s| s.id == "tdd").unwrap();
        assert!(
            tdd.body_markdown.contains("Red→Green→Refactor"),
            "preset body should replace the generated stub: {}",
            &tdd.body_markdown[..tdd.body_markdown.len().min(80)]
        );
    }

    #[test]
    fn inject_clones_new_skill_into_ring2() {
        let p = confirmed_developer_profile();
        let mut bundle = compile_profile(&p).unwrap();
        // Remove `debug` to force the clone path.
        bundle.skills.retain(|s| s.id != "debug");
        let before = bundle.skills.len();

        inject_preset(&mut bundle, Genre::Developer, "debug").unwrap();
        assert_eq!(bundle.skills.len(), before + 1, "clone must add one skill");
        let debug = bundle.skills.iter().find(|s| s.id == "debug").unwrap();
        assert_eq!(debug.ring, Ring::Ring2);
        assert!(
            debug.body_markdown.to_lowercase().contains("root cause"),
            "debug preset body should describe root-cause isolation"
        );
    }

    #[test]
    fn inject_unknown_skill_errors() {
        let p = confirmed_developer_profile();
        let mut bundle = compile_profile(&p).unwrap();
        let err = inject_preset(&mut bundle, Genre::Developer, "nonexistent").unwrap_err();
        assert!(matches!(err, ByohError::Schema(_)), "got {err:?}");
    }

    #[test]
    fn preset_body_round_trips() {
        let body = preset_body(Genre::Creator, "continuity").unwrap();
        assert!(body.contains("continuity"));
    }

    #[test]
    fn inject_researcher_and_business_presets_clone() {
        // These presets don't exist in the base templates → clone path (append).
        use crate::domain::profile::GenreConfidence;

        // Researcher bundle.
        let mut rp = UserProfile::new_draft("res", "en");
        rp.candidates.identity.genre = Some(GenreConfidence {
            value: Genre::Researcher,
            confidence: 1.0,
            provenance: vec![],
        });
        rp.status = ProfileStatus::Confirmed;
        let mut rbundle = compile_profile(&rp).unwrap();
        let rbefore = rbundle.skills.len();
        inject_preset(&mut rbundle, Genre::Researcher, "evidence").unwrap();
        assert_eq!(rbundle.skills.len(), rbefore + 1);
        inject_preset(&mut rbundle, Genre::Researcher, "reproducibility").unwrap();
        assert_eq!(rbundle.skills.len(), rbefore + 2);

        // Business bundle.
        let mut bp = UserProfile::new_draft("biz", "en");
        bp.candidates.identity.genre = Some(GenreConfidence {
            value: Genre::Business,
            confidence: 1.0,
            provenance: vec![],
        });
        bp.status = ProfileStatus::Confirmed;
        let mut bbundle = compile_profile(&bp).unwrap();
        let bbefore = bbundle.skills.len();
        inject_preset(&mut bbundle, Genre::Business, "decision").unwrap();
        assert_eq!(bbundle.skills.len(), bbefore + 1);
        inject_preset(&mut bbundle, Genre::Business, "plainlanguage").unwrap();
        assert_eq!(bbundle.skills.len(), bbefore + 2);

        // Bodies carry their distinctive content.
        assert!(rbundle
            .skills
            .iter()
            .find(|s| s.id == "evidence")
            .unwrap()
            .body_markdown
            .to_lowercase()
            .contains("tier"));
        assert!(bbundle
            .skills
            .iter()
            .find(|s| s.id == "decision")
            .unwrap()
            .body_markdown
            .to_lowercase()
            .contains("opportunity cost"));
    }
}

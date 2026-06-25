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
}

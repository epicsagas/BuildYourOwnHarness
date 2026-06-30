//! Inheritance / override merge — a child template's rings are the union of
//! base + child, with child entries appended after base (override = presence).

use crate::domain::genre::{GenreTemplate, TemplateRings};

/// Merge a child template into its base. Ring skeletons from base are always
/// retained; child adds/overrides skill ids (ARCH §6.1). Safety gates are
/// never removed (validated separately at compile/evolve time).
pub fn merge_child_into_base(base: &GenreTemplate, child: &GenreTemplate) -> GenreTemplate {
    let mut merged = base.clone();
    merged.name = child.name.clone();
    merged.genre = child.genre;
    merged.mvp = child.mvp;
    merged.extends = Some(base.name.clone());
    merged.rings = TemplateRings {
        ring0_hooks: union(&base.rings.ring0_hooks, &child.rings.ring0_hooks),
        ring1_pipeline: child.rings.ring1_pipeline.clone(), // child fully overrides pipeline
        ring2_quality: child.rings.ring2_quality.clone(),   // child fully overrides quality
        ring3_evolution: union(&base.rings.ring3_evolution, &child.rings.ring3_evolution),
    };
    merged.tool_blueprints = union(&base.tool_blueprints, &child.tool_blueprints);
    merged.evolution = child.evolution.clone();
    merged.description_en = child.description_en.clone();
    merged.description_ko = child.description_ko.clone();
    merged
}

fn union(base: &[String], child: &[String]) -> Vec<String> {
    let mut out: Vec<String> = base.to_vec();
    for item in child {
        if !out.contains(item) {
            out.push(item.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::base::base_template;
    use crate::templates::library::creator_template;

    #[test]
    fn merge_keeps_base_hooks_and_adds_child_hook() {
        let base = base_template();
        let child = creator_template();
        let merged = merge_child_into_base(&base, &child);
        assert!(
            merged
                .rings
                .ring0_hooks
                .contains(&"session_start_resume".to_string())
        );
        assert!(
            merged
                .rings
                .ring0_hooks
                .contains(&"post_tool_use_tone_spellcheck".to_string())
        );
        // safety gates retained
        assert!(merged.rings.ring3_evolution.contains(&"critic".to_string()));
    }
}

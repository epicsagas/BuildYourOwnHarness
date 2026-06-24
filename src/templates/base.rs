//! The base template — immutable Ring 0-3 skeleton + 3 safety gates.

use crate::domain::genre::{Genre, GenreEvolutionParams, GenreTemplate, SafetyGate, TemplateRings};

/// The shared base. Every child inherits Ring 0-3 + all three safety gates
/// (ARCH §6.1: override is allowed on skill bodies / tools / domain entities,
/// NOT on the Ring skeleton or safety gates).
pub fn base_template() -> GenreTemplate {
    GenreTemplate {
        name: "base".into(),
        extends: None,
        genre: Genre::Developer, // placeholder; children set their own
        mvp: false,
        rings: TemplateRings {
            ring0_hooks: vec![
                "session_start_resume".into(),
                "pre_tool_use_guard".into(),
                "post_tool_use_read_compress".into(), // B13
                "session_end_observe".into(),
            ],
            ring1_pipeline: vec!["spec".into(), "go".into(), "check".into(), "ship".into()],
            ring2_quality: vec!["tdd".into(), "debug".into(), "secure".into(), "perf".into()],
            ring3_evolution: vec!["critic".into(), "seesaw".into(), "stagnation".into()],
        },
        tool_blueprints: Vec::new(),
        evolution: GenreEvolutionParams {
            genre: Genre::Developer,
            improvement_threshold: 0.02,
            stagnation_limit: 3,
            critic_weight: 1.0,
        },
        description_en: "Base harness skeleton — 4-Ring + 3 safety gates.".into(),
        description_ko: "기본 하네스 골격 — 4-Ring + 3중 안전장치.".into(),
    }
}

/// The mandatory safety gates names, in canonical order.
pub fn mandatory_safety_gates() -> Vec<String> {
    SafetyGate::ALL
        .iter()
        .map(|g| g.as_str().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_has_all_three_safety_gates() {
        let b = base_template();
        let gates = &b.rings.ring3_evolution;
        for g in SafetyGate::ALL {
            assert!(gates.contains(&g.as_str().to_string()), "missing {g}");
        }
    }

    #[test]
    fn base_has_four_rings() {
        let b = base_template();
        assert!(!b.rings.ring0_hooks.is_empty());
        assert!(!b.rings.ring1_pipeline.is_empty());
        assert!(!b.rings.ring2_quality.is_empty());
        assert!(!b.rings.ring3_evolution.is_empty());
    }
}

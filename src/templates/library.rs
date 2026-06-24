//! The four child templates + library lookup.

use crate::domain::genre::{Genre, GenreEvolutionParams, GenreTemplate};
use crate::templates::base::base_template;

/// `developer` — MVP. Reuses epic-harness-like coding skills (lowest risk).
pub fn developer_template() -> GenreTemplate {
    let mut t = base_template();
    t.name = "developer".into();
    t.genre = Genre::Developer;
    t.mvp = true;
    t.extends = Some("base".into());
    t.rings.ring2_quality = vec!["tdd".into(), "debug".into(), "secure".into(), "perf".into()];
    t.tool_blueprints = vec![
        "search_code".into(), // B5 code search
    ];
    t.evolution = GenreEvolutionParams::for_genre(Genre::Developer);
    t.description_en = "Developer harness — spec→go→check→ship with code-quality skills.".into();
    t.description_ko = "개발자 하네스 — spec→go→check→ship + 코드 품질 스킬.".into();
    t
}

/// `creator` — MVP. Long-form drafting pipeline + tone/continuity.
pub fn creator_template() -> GenreTemplate {
    let mut t = base_template();
    t.name = "creator".into();
    t.genre = Genre::Creator;
    t.mvp = true;
    t.extends = Some("base".into());
    t.rings.ring1_pipeline = vec![
        "draft".into(),
        "edit".into(),
        "proofread".into(),
        "publish".into(),
    ];
    t.rings
        .ring0_hooks
        .push("post_tool_use_tone_spellcheck".into());
    t.rings.ring2_quality = vec!["continuity".into(), "character_consistency".into()];
    t.tool_blueprints = vec!["search_draft_continuity".into()];
    t.evolution = GenreEvolutionParams::for_genre(Genre::Creator);
    t.description_en =
        "Creator harness — draft→edit→proofread→publish with continuity skills.".into();
    t.description_ko = "크리에이터 하네스 — draft→edit→proofread→publish + 연속성 스킬.".into();
    t
}

/// `researcher` — extension. Adds literature-review step + citation PARA.
pub fn researcher_template() -> GenreTemplate {
    let mut t = base_template();
    t.name = "researcher".into();
    t.genre = Genre::Researcher;
    t.mvp = false;
    t.extends = Some("base".into());
    t.rings.ring1_pipeline = vec![
        "spec".into(),
        "literature_review".into(),
        "go".into(),
        "check".into(),
        "ship".into(),
    ];
    t.rings.ring2_quality = vec!["citation_accuracy".into(), "source_verification".into()];
    t.tool_blueprints = vec!["search_citations".into()];
    t.evolution = GenreEvolutionParams::for_genre(Genre::Researcher);
    t.description_en = "Researcher harness — adds literature review + citation management.".into();
    t.description_ko = "연구자 하네스 — 문헌 리뷰 + 인용 관리 추가.".into();
    t
}

/// `business` — extension. Decision memory + ROI skill + Council gate.
pub fn business_template() -> GenreTemplate {
    let mut t = base_template();
    t.name = "business".into();
    t.genre = Genre::Business;
    t.mvp = false;
    t.extends = Some("base".into());
    t.rings.ring1_pipeline = vec![
        "goal".into(),
        "analyze".into(),
        "decide".into(),
        "execute".into(),
    ];
    t.rings.ring2_quality = vec!["roi_evaluation".into(), "risk_assessment".into()];
    t.tool_blueprints = vec!["search_decisions".into()];
    t.evolution = GenreEvolutionParams::for_genre(Genre::Business);
    t.description_en =
        "Business harness — decision pipeline with ROI/risk + Council decision gate.".into();
    t.description_ko =
        "비즈니스 하네스 — 의사결정 파이프라인 + ROI/리스크 + Council 게이트.".into();
    t
}

/// The template library. Lookup by genre; children are built by inheriting base.
#[derive(Debug, Default, Clone)]
pub struct TemplateLibrary;

impl TemplateLibrary {
    pub fn new() -> Self {
        Self
    }

    pub fn get(&self, genre: Genre) -> GenreTemplate {
        match genre {
            Genre::Developer => developer_template(),
            Genre::Creator => creator_template(),
            Genre::Researcher => researcher_template(),
            Genre::Business => business_template(),
        }
    }

    pub fn all(&self) -> Vec<GenreTemplate> {
        Genre::all().iter().copied().map(|g| self.get(g)).collect()
    }

    pub fn mvp_genres(&self) -> Vec<GenreTemplate> {
        self.all().into_iter().filter(|t| t.mvp).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::genre::SafetyGate;

    #[test]
    fn every_child_inherits_three_safety_gates() {
        let lib = TemplateLibrary::new();
        for g in Genre::all() {
            let t = lib.get(*g);
            for gate in SafetyGate::ALL {
                assert!(
                    t.rings.ring3_evolution.contains(&gate.as_str().to_string()),
                    "genre {:?} missing gate {}",
                    g,
                    gate
                );
            }
        }
    }

    #[test]
    fn mvp_set_is_developer_creator() {
        let lib = TemplateLibrary::new();
        let mvps: Vec<Genre> = lib.mvp_genres().into_iter().map(|t| t.genre).collect();
        assert_eq!(mvps.len(), 2);
        assert!(mvps.contains(&Genre::Developer));
        assert!(mvps.contains(&Genre::Creator));
    }

    #[test]
    fn creator_overrides_pipeline() {
        let lib = TemplateLibrary::new();
        let c = lib.get(Genre::Creator);
        assert_eq!(c.rings.ring1_pipeline[0], "draft");
        assert!(c
            .rings
            .ring0_hooks
            .contains(&"post_tool_use_tone_spellcheck".to_string()));
    }

    #[test]
    fn researcher_adds_literature_review() {
        let lib = TemplateLibrary::new();
        let r = lib.get(Genre::Researcher);
        assert!(r
            .rings
            .ring1_pipeline
            .contains(&"literature_review".to_string()));
    }
}

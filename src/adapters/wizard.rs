//! Static wizard adapter — S3 decisive self-describing options.

use crate::domain::genre::Genre;
use crate::ports::wizard::{WizardOption, WizardPort};

/// A built-in, static set of self-describing options (B17 ko/en).
#[derive(Debug, Default, Clone)]
pub struct StaticWizard;

impl StaticWizard {
    pub fn new() -> Self {
        Self
    }
}

impl WizardPort for StaticWizard {
    fn genre_options(&self, _language: &str) -> Vec<WizardOption> {
        vec![
            opt(
                "developer",
                "Developer",
                "개발자",
                "You write code and want spec→build→check pipelines plus code-quality skills.",
                "코드를 작성하며 spec→build→check 파이프라인과 코드 품질 스킬이 필요합니다.",
            ),
            opt(
                "creator",
                "Creator",
                "크리에이터",
                "You draft long-form content and need continuity, tone consistency, and drafting pipelines.",
                "장문 콘텐츠를 쓰며 연속성·톤 일관성·초안 파이프라인이 필요합니다.",
            ),
            opt(
                "researcher",
                "Researcher",
                "연구자",
                "You work with citations and literature-review steps (extension genre).",
                "인용과 문헌 리뷰 단계를 다룹니다 (확장 장르).",
            ),
            opt(
                "business",
                "Business",
                "비즈니스",
                "You make decisions and want ROI/risk skills and decision memory (extension genre).",
                "의사결정을 하며 ROI/리스크 스킬과 결정 메모리가 필요합니다 (확장 장르).",
            ),
        ]
    }

    fn goal_options(&self, genre: Genre, _language: &str) -> Vec<WizardOption> {
        match genre {
            Genre::Developer => vec![
                opt(
                    "ship",
                    "Ship features faster",
                    "기능을 더 빠르게 출시",
                    "Pipeline accelerates spec→ship.",
                    "파이프라인이 spec→ship을 가속합니다.",
                ),
                opt(
                    "quality",
                    "Raise code quality",
                    "코드 품질 향상",
                    "Ring 2 adds tdd/debug/secure.",
                    "Ring 2에 tdd/debug/secure가 추가됩니다.",
                ),
            ],
            Genre::Creator => vec![
                opt(
                    "draft",
                    "Draft more",
                    "더 많이 초안 작성",
                    "Drafting pipeline + tone hooks.",
                    "초안 파이프라인 + 톤 훅.",
                ),
                opt(
                    "edit",
                    "Edit efficiently",
                    "효율적으로 편집",
                    "Continuity + character consistency.",
                    "연속성 + 캐릭터 일관성.",
                ),
            ],
            Genre::Researcher => vec![opt(
                "cite",
                "Cite accurately",
                "정확하게 인용",
                "Citation PARA + literature step.",
                "인용 PARA + 문헌 단계.",
            )],
            Genre::Business => vec![opt(
                "decide",
                "Decide with ROI",
                "ROI로 의사결정",
                "ROI skill + decision memory.",
                "ROI 스킬 + 결정 메모리.",
            )],
        }
    }
}

fn opt(id: &str, label_en: &str, label_ko: &str, why_en: &str, why_ko: &str) -> WizardOption {
    WizardOption {
        id: id.into(),
        label_en: label_en.into(),
        label_ko: label_ko.into(),
        why_en: why_en.into(),
        why_ko: why_ko.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genre_options_self_describing() {
        let w = StaticWizard::new();
        let opts = w.genre_options("ko");
        assert_eq!(opts.len(), 4);
        // AC5: each option carries a "why".
        assert!(!opts[0].why_ko.is_empty());
    }

    #[test]
    fn render_option_bilingual() {
        let w = StaticWizard::new();
        let opts = w.genre_options("en");
        let en = w.render_option(&opts[0], "en");
        let ko = w.render_option(&opts[0], "ko");
        assert!(en.contains("Developer"));
        assert!(ko.contains("개발자"));
    }
}

//! Rule-based LLM adapter — deterministic, no network. Used for tests and the
//! offline/M0 default. A real provider adapter can implement `LlmPort` later.

use crate::domain::profile::DerivedFact;
use crate::ports::llm::{CouncilVoice, LlmPort, Suggestion};

/// A deterministic, offline LLM. Suggestions are templated from the candidate
/// values; council questions are voice-specific heuristics.
#[derive(Debug, Default, Clone)]
pub struct RuleLlm;

impl RuleLlm {
    pub fn new() -> Self {
        Self
    }
}

impl LlmPort for RuleLlm {
    fn suggest_for(&self, weak: &[&DerivedFact], language: &str) -> Vec<Suggestion> {
        weak.iter()
            .enumerate()
            .map(|(i, fact)| {
                let (rationale, text) = if language == "ko" {
                    (
                        format!(
                            "'{}' 후보의 신뢰도가 {:.2}로 낮아 재확인이 필요합니다.",
                            fact.value, fact.confidence
                        ),
                        fact.value.clone(),
                    )
                } else {
                    (
                        format!(
                            "Candidate '{}' has low confidence {:.2}; please confirm.",
                            fact.value, fact.confidence
                        ),
                        fact.value.clone(),
                    )
                };
                Suggestion {
                    question_id: format!("Q{}", i + 1),
                    suggested_answer: text,
                    rationale,
                    confidence: fact.confidence,
                }
            })
            .collect()
    }

    fn council_questions(&self, context: &str, language: &str) -> Vec<(CouncilVoice, String)> {
        CouncilVoice::ALL
            .iter()
            .map(|&voice| {
                let q = match voice {
                    CouncilVoice::Architect => {
                        if language == "ko" {
                            format!("문맥({context})에서 가장 유지보수하기 쉬운 장르는 무엇입니까?")
                        } else {
                            format!(
                                "Given the context ({context}), which genre is most maintainable?"
                            )
                        }
                    }
                    CouncilVoice::Skeptic => {
                        if language == "ko" {
                            "가장 단순한 대안은 무엇이며, 복잡한 장르가 정말 필요합니까?"
                                .to_string()
                        } else {
                            "What is the simplest alternative, and is a complex genre truly needed?"
                                .to_string()
                        }
                    }
                    CouncilVoice::Pragmatist => {
                        if language == "ko" {
                            "지금 당장 가장 빠르게 가치를 낼 수 있는 장르는 무엇입니까?".to_string()
                        } else {
                            "Which genre delivers value fastest right now?".to_string()
                        }
                    }
                    CouncilVoice::Critic => {
                        if language == "ko" {
                            "잘못된 장르 선택의 가장 큰 실패 모드는 무엇입니까?".to_string()
                        } else {
                            "What is the biggest failure mode of picking the wrong genre?"
                                .to_string()
                        }
                    }
                };
                (voice, q)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn council_has_four_voices() {
        let llm = RuleLlm::new();
        let qs = llm.council_questions("novel writing", "en");
        assert_eq!(qs.len(), 4);
        let voices: Vec<_> = qs.iter().map(|(v, _)| *v).collect();
        assert!(voices.contains(&CouncilVoice::Critic));
    }

    #[test]
    fn suggest_for_weak_candidates() {
        let llm = RuleLlm::new();
        let f = DerivedFact {
            value: "k8s".into(),
            confidence: 0.4,
            provenance: vec![],
        };
        let s = llm.suggest_for(&[&f], "en");
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].suggested_answer, "k8s");
    }
}

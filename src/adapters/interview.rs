//! Rule-based interview adapter — S2 Suggest-Confirm loop.

use std::collections::HashMap;

use crate::domain::profile::{Axis, UserProfile};
use crate::ports::interview::{InterviewPort, Question};
use crate::ports::llm::{LlmPort, Suggestion};

/// Drives S2 using an LLM port for suggestions. Axis coverage is tracked in
/// `interview_meta.axis_completion`.
pub struct RuleInterview<L: LlmPort> {
    llm: L,
}

impl<L: LlmPort> RuleInterview<L> {
    pub fn new(llm: L) -> Self {
        Self { llm }
    }
}

impl<L: LlmPort> InterviewPort for RuleInterview<L> {
    fn next_questions(&self, profile: &UserProfile) -> Vec<Question> {
        let lang = &profile.language;
        let weak = profile.weak_candidates();
        let suggestions = self.llm.suggest_for(&weak, lang);

        let mut by_id: HashMap<String, Suggestion> = HashMap::new();
        for s in suggestions {
            by_id.insert(s.question_id.clone(), s);
        }

        let mut qs = Vec::new();

        // Identity/domain questions target the Tacit axis.
        if profile.truth.identity.domain.is_none() {
            let id = "Q_domain".to_string();
            qs.push(Question {
                id: id.clone(),
                text: if lang == "ko" {
                    "당신의 주 작업 영역은 무엇입니까?".into()
                } else {
                    "What is your primary working domain?".into()
                },
                axis: Axis::Tacit,
                suggestion: by_id.get("Q1").cloned(),
            });
        }

        // Goal questions → Goals axis.
        if profile.truth.goals.goal_30d.is_none() {
            qs.push(Question {
                id: "Q_goal".into(),
                text: if lang == "ko" {
                    "다음 30일간 가장 중요한 목표 한 가지는?".into()
                } else {
                    "What is your single most important goal for the next 30 days?".into()
                },
                axis: Axis::Goals,
                suggestion: by_id.get("Q2").cloned(),
            });
        }

        // Genre questions → Genre axis.
        if profile.candidates.identity.genre.is_none()
            || profile
                .candidates
                .identity
                .genre
                .as_ref()
                .map(|g| g.confidence < 0.7)
                .unwrap_or(true)
        {
            qs.push(Question {
                id: "Q_genre".into(),
                text: if lang == "ko" {
                    "당신의 작업은 어떤 장르에 가장 가깝습니까? (developer/creator/researcher/business)".into()
                } else {
                    "Which genre best fits your work? (developer/creator/researcher/business)".into()
                },
                axis: Axis::Genre,
                suggestion: by_id.get("Q3").cloned(),
            });
        }

        // Note: there is no Data-axis question. Data context is derived from the
        // S1 autoscan (`data_sources`), not a user question — asking "do you have
        // existing resources?" was a KB-ingestion prompt with no consumer now that
        // BYOH ships no embedded knowledge base. One fewer interview round-trip.
        qs
    }

    fn apply_answer(
        &self,
        profile: &mut UserProfile,
        question: &Question,
        accepted_answer: &str,
        confidence: f64,
    ) {
        match question.id.as_str() {
            "Q_domain" => {
                profile.truth.identity.domain = Some(accepted_answer.to_string());
                profile.set_axis(Axis::Tacit, 0.85_f64.max(confidence));
            }
            "Q_goal" => {
                profile.truth.goals.goal_30d = Some(accepted_answer.to_string());
                profile.set_axis(Axis::Goals, 0.8_f64.max(confidence));
            }
            "Q_genre" => {
                if let Ok(g) = accepted_answer.parse::<crate::domain::genre::Genre>() {
                    profile.candidates.identity.genre =
                        Some(crate::domain::profile::GenreConfidence {
                            value: g,
                            confidence,
                            provenance: vec!["interview".into()],
                        });
                }
                profile.set_axis(Axis::Genre, 0.9_f64.max(confidence));
            }
            _ => {}
        }
        profile.interview_meta.questions_asked += 1;
        profile.updated_at = Some(chrono::Utc::now());
    }

    fn is_complete(&self, profile: &UserProfile) -> bool {
        profile.interview_meta.axis_completion.all_above_threshold()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::RuleLlm;

    #[test]
    fn interview_fills_truth_and_advances_axes() {
        let llm = RuleLlm::new();
        let iv = RuleInterview::new(llm);
        let mut p = UserProfile::new_draft("dev1", "en");
        // seed a weak candidate so the LLM offers suggestions
        p.candidates
            .identity
            .primary_expertise
            .push(crate::domain::profile::DerivedFact {
                value: "k8s".into(),
                confidence: 0.4,
                provenance: vec![],
            });

        let qs = iv.next_questions(&p);
        assert!(!qs.is_empty());

        for q in &qs {
            iv.apply_answer(&mut p, q, "backend", 0.9);
        }
        assert!(iv.is_complete(&p));
    }
}

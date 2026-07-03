//! Rule-based interview adapter — S2 Suggest-Confirm loop.

use crate::domain::profile::{Axis, UserProfile};
use crate::ports::interview::{InterviewPort, Question};

/// Rule-based S2 interview: three axis questions (domain / goal / genre).
/// Axis coverage is tracked in `interview_meta.axis_completion`.
#[derive(Debug, Default, Clone)]
pub struct RuleInterview;

impl RuleInterview {
    pub fn new() -> Self {
        Self
    }
}

impl InterviewPort for RuleInterview {
    fn next_questions(&self, profile: &UserProfile) -> Vec<Question> {
        let lang = &profile.language;
        // No auto-suggestions are attached: scan-derived candidates are keyed by
        // weak-candidate index, not by question, so mapping them onto questions
        // would write machine noise into the truth block (Suggest-don't-move).
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
                suggestion: None,
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
                suggestion: None,
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
                suggestion: None,
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
                // Only an answer that parses to a real genre completes the axis;
                // an unparseable answer leaves the question open instead of
                // "completing" the interview with no genre.
                if let Ok(g) = accepted_answer
                    .trim()
                    .to_lowercase()
                    .parse::<crate::domain::genre::Genre>()
                {
                    profile.candidates.identity.genre =
                        Some(crate::domain::profile::GenreConfidence {
                            value: g,
                            confidence,
                            provenance: vec!["interview".into()],
                        });
                    profile.set_axis(Axis::Genre, 0.9_f64.max(confidence));
                }
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

    #[test]
    fn interview_fills_truth_and_advances_axes() {
        let iv = RuleInterview::new();
        let mut p = UserProfile::new_draft("dev1", "en");
        // seed a weak candidate (scan-derived; must NOT leak into questions)
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
            let answer = if q.id == "Q_genre" {
                "Developer"
            } else {
                "backend"
            };
            iv.apply_answer(&mut p, q, answer, 0.9);
        }
        assert!(iv.is_complete(&p));
    }

    #[test]
    fn unparseable_genre_leaves_axis_incomplete() {
        let iv = RuleInterview::new();
        let mut p = UserProfile::new_draft("dev1", "en");
        let qs = iv.next_questions(&p);
        for q in &qs {
            iv.apply_answer(&mut p, q, "not-a-genre", 0.9);
        }
        assert!(p.candidates.identity.genre.is_none());
        assert!(!iv.is_complete(&p));
    }
}

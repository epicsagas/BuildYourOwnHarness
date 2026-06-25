//! Interview port — S2 Suggest-Confirm loop.

use crate::domain::profile::UserProfile;
use crate::ports::llm::Suggestion;

/// A question the interview poses (with a suggested answer attached).
#[derive(Debug, Clone, PartialEq)]
pub struct Question {
    pub id: String,
    pub text: String,
    pub axis: crate::domain::profile::Axis,
    pub suggestion: Option<Suggestion>,
}

/// Drives S2. Produces questions, ingests answers into `truth`.
pub trait InterviewPort {
    /// Produce the next batch of questions given the current profile.
    fn next_questions(&self, profile: &UserProfile) -> Vec<Question>;

    /// Apply a user's answer (accepting or editing a suggestion) into `truth`.
    fn apply_answer(
        &self,
        profile: &mut UserProfile,
        question: &Question,
        accepted_answer: &str,
        confidence: f64,
    );

    /// Whether all axes are above the completion threshold (Interview §6).
    fn is_complete(&self, profile: &UserProfile) -> bool {
        profile.interview_meta.axis_completion.all_above_threshold()
    }
}

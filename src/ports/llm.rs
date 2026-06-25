//! LLM port — used by interview (S2), council, and evolution. Abstracts the
//! external model so the crate is testable with a rule-based adapter.

use crate::domain::profile::DerivedFact;

/// A suggested answer for an open question (B1 Suggest-don't-move).
#[derive(Debug, Clone, PartialEq)]
pub struct Suggestion {
    pub question_id: String,
    pub suggested_answer: String,
    /// Why this answer is proposed (transparency for the user).
    pub rationale: String,
    pub confidence: f64,
}

/// Council voice identities (ARCH §7.4, B12 anti-anchoring).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CouncilVoice {
    Architect,
    Skeptic,
    Pragmatist,
    Critic,
}

impl CouncilVoice {
    pub const ALL: [CouncilVoice; 4] = [
        CouncilVoice::Architect,
        CouncilVoice::Skeptic,
        CouncilVoice::Pragmatist,
        CouncilVoice::Critic,
    ];
    pub fn as_str(self) -> &'static str {
        use CouncilVoice::*;
        match self {
            Architect => "architect",
            Skeptic => "skeptic",
            Pragmatist => "pragmatist",
            Critic => "critic",
        }
    }
}

/// The port. Implementations: [`crate::adapters::llm::RuleLlm`] (tests/offline),
/// and a real-provider adapter behind a feature in the future.
pub trait LlmPort {
    /// Generate suggestions for weak candidates (S2).
    fn suggest_for(&self, weak: &[&DerivedFact], language: &str) -> Vec<Suggestion>;

    /// Generate a clarifying question from each council voice for an ambiguous
    /// genre (B12). Returns one question per voice — independent context.
    fn council_questions(&self, context: &str, language: &str) -> Vec<(CouncilVoice, String)>;

    /// Verdict voice for the compile gate (ARCH §5.4): does this bundle threaten
    /// the user's goal? Returns `true` if it looks safe per this voice.
    fn council_verdict(&self, voice: CouncilVoice, goal: &str, bundle_summary: &str) -> bool;
}

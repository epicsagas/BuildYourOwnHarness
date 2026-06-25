//! Evolution engine — Ring 3 (ARCH §7). Observe → Analyze → Evolve → Gate,
//! with the 3 mandatory safety gates (Critic / Seesaw / Stagnation).

pub mod compress;
pub mod gates;
pub mod lifecycle;
pub mod recall;
pub mod skills;

pub use compress::{compress, CompressionTier, ImportanceWeights, Token, TokenKind};
pub use gates::{
    critic_review, CriticVerdict, EditType, SafetyGateSet, SeesawState, StagnationAction,
    StagnationState,
};
pub use lifecycle::{run_cycle, EvolutionCycle, EvolutionDecision};
pub use recall::{recall_score, recency_value, RecallWeights};
pub use skills::{mine_patterns, SkillSeed};

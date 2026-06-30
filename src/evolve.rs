//! Evolution engine — Ring 3 (ARCH §7). Observe → Analyze → Evolve → Gate,
//! with the 3 mandatory safety gates (Critic / Seesaw / Stagnation).

pub mod compress;
pub mod gates;
pub mod lifecycle;
pub mod recall;
pub mod skills;
pub mod state;

pub use compress::{CompressionTier, ImportanceWeights, Token, TokenKind, compress};
pub use gates::{
    CriticVerdict, EditType, SafetyGateSet, SeesawState, StagnationAction, StagnationState,
    critic_review,
};
pub use lifecycle::{EvolutionCycle, EvolutionDecision, run_cycle};
pub use recall::{RecallWeights, recall_score, recency_value};
pub use skills::{SkillSeed, mine_patterns};
pub use state::{EvolveState, EvolveStore};

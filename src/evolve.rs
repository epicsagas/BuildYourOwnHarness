//! Evolution engine — Ring 3 (ARCH §7). Analyze → Evolve → Gate,
//! with the 3 mandatory safety gates (Critic / Seesaw / Stagnation).

pub mod gates;
pub mod lifecycle;
pub mod state;

pub use gates::{
    CriticVerdict, EditType, SafetyGateSet, SeesawState, StagnationAction, StagnationState,
    critic_review,
};
pub use lifecycle::{EvolutionCycle, EvolutionDecision, run_cycle};
pub use state::{EvolveState, EvolveStore};

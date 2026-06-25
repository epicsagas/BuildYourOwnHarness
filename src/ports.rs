//! Port traits (hexagonal boundaries). Adapters implement these.
//!
//! All ports are deliberately external-LLM-free at the type level: the
//! `LlmPort` returns structured suggestions, and tests use a rule-based
//! adapter so the crate builds and tests with no network.

pub mod command;
pub mod embedder;
pub mod interview;
pub mod llm;
pub mod source;
pub mod wizard;

pub use command::CommandPort;
pub use embedder::{EmbedderProvider, Embedding};
pub use interview::{InterviewPort, Question};
pub use llm::{CouncilVoice, LlmPort, Suggestion};
pub use source::{ProfileSource, ScanHit};
pub use wizard::{WizardOption, WizardPort};

//! Pure domain types — no I/O, no trait objects that touch the outside world.
//!
//! Mirrors the schemas in `docs/02_ARCHITECTURE.md` §3-6 and
//! `docs/03_INTERVIEW_DESIGN.md` §6.

pub mod bundle;
pub mod error;
pub mod evidence;
pub mod genre;
pub mod profile;
pub mod render_target;
pub mod scope;
pub mod synthesis;

pub use bundle::{
    AgentSpec, BundleConfig, BundleVersion, HarnessBundle, HookSpec, McpTool, Ring, SkillSpec,
};
pub use error::{ByohError, Result};
pub use evidence::{ObservationRecord, ObservedOutcome};
pub use genre::{Genre, GenreTemplate, SafetyGates};
pub use profile::{
    Candidates, ConfidenceItem, ConfirmedProfile, DataSources, DerivedBlock, DerivedFact,
    DraftProfile, GenreConfidence, InterviewMeta, ProfileStatus, ProviderPreference, TruthBlock,
    UserProfile,
};

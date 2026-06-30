//! UserProfile schema — the heart of the aggregation pipeline.
//!
//! Implements the nested-block structure from `docs/03_INTERVIEW_DESIGN.md` §6:
//!   - `truth:`     — user-confirmed single source of truth (B6 `UserTruth`)
//!   - `candidates:` — auto-scan / interview inferences, all `derived` (B6 `DerivedFact`)
//!   - `derived:`    — inverse-derived from truth, fallback only
//!
//! State machine (B1 extension, naming aligned with ARCH §3.2 / Interview §6):
//!   `draft → interviewed → confirmed → evolving`
//! Cross-document naming note: ARCH §3.2 also uses `scan/suggested/confirmed/processed`.
//! We canonicalize on the Interview-§6 four-state set (`draft/interviewed/confirmed/evolving`)
//! because ARCH §3.2 line 661 declares it authoritative for the ER data model.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::genre::Genre;

// ──────────────────────────────────────────────────────────────────────────
// Status state machine
// ──────────────────────────────────────────────────────────────────────────

/// Profile lifecycle state. See module docs for the legal transition graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProfileStatus {
    /// Autoscan candidates injected (M0: empty candidates) — awaiting interview.
    Draft,
    /// Interview in progress / complete; `truth` filling progressively.
    Interviewed,
    /// User confirmed via wizard; truth block frozen.
    Confirmed,
    /// Harness installed; evolution engine (B10) accumulating observations.
    Evolving,
}

impl ProfileStatus {
    /// Legal successors from this state.
    pub fn allowed_next(self) -> &'static [ProfileStatus] {
        use ProfileStatus::*;
        match self {
            Draft => &[Interviewed],
            Interviewed => &[Confirmed],
            Confirmed => &[Evolving],
            Evolving => &[],
        }
    }

    /// Validate a proposed transition. Returns the canonical error otherwise.
    pub fn transition(self, to: ProfileStatus) -> super::Result<ProfileStatus> {
        if self.allowed_next().contains(&to) {
            Ok(to)
        } else {
            Err(super::ByohError::InvalidTransition {
                from: self,
                to,
                allowed: "see ProfileStatus::allowed_next",
            })
        }
    }

    pub fn as_str(self) -> &'static str {
        use ProfileStatus::*;
        match self {
            Draft => "draft",
            Interviewed => "interviewed",
            Confirmed => "confirmed",
            Evolving => "evolving",
        }
    }
}

impl std::fmt::Display for ProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ProfileStatus {
    type Err = super::ByohError;
    fn from_str(s: &str) -> super::Result<Self> {
        use ProfileStatus::*;
        Ok(match s {
            "draft" => Draft,
            "interviewed" => Interviewed,
            "confirmed" => Confirmed,
            "evolving" => Evolving,
            other => {
                return Err(super::ByohError::Schema(format!(
                    "unknown profile_status '{other}'"
                )));
            }
        })
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Confidence-tagged values (B1 candidate fields + provenance)
// ──────────────────────────────────────────────────────────────────────────

/// A user-weighted value (truth block): confidence is self-reported.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceItem {
    pub value: String,
    /// User-reported confidence 0.0..=1.0.
    #[serde(default)]
    pub confidence_user: Option<f64>,
}

/// An inferred value (candidates/derived block): confidence is machine-derived.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DerivedFact {
    pub value: String,
    /// Machine confidence 0.0..=1.0. Below 0.6 → re-question.
    #[serde(default)]
    pub confidence: f64,
    /// B6 provenance: where the inference came from.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provenance: Vec<String>,
}

impl DerivedFact {
    /// Values below the re-question threshold (Interview §6).
    pub const REQUESTION_THRESHOLD: f64 = 0.6;

    pub fn needs_reevaluation(&self) -> bool {
        self.confidence < Self::REQUESTION_THRESHOLD
    }
}

/// `truth.identity` block.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TruthIdentity {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub primary_expertise: Vec<ConfidenceItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routines: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub automation_targets: Vec<String>,
}

/// `truth.goals` block.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TruthGoals {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_30d: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success_90d: Option<String>,
}

/// `truth.resources` block.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TruthResources {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<Budget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_budget: Option<TimeBudget>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Budget {
    #[serde(default)]
    pub monthly_usd: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TimeBudget {
    #[serde(default)]
    pub daily_minutes: Option<u32>,
}

/// `truth.context` block.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TruthContext {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<String>,
}

/// `truth.data` block — privacy tier classification (B6).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TruthData {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    #[serde(default)]
    pub privacy_tier: PrivacyTier,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrivacyTier {
    Internal,
    #[default]
    Confidential,
    Restricted,
}

/// The immutable single-source-of-truth block. (ARCH §4.2 `UserTruth`.)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TruthBlock {
    #[serde(default)]
    pub identity: TruthIdentity,
    #[serde(default)]
    pub context: TruthContext,
    #[serde(default)]
    pub goals: TruthGoals,
    #[serde(default)]
    pub resources: TruthResources,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    #[serde(default)]
    pub data: TruthData,
}

// ──────────────────────────────────────────────────────────────────────────
// Candidates block (machine-derived, all `derived:true`)
// ──────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CandidatesIdentity {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub primary_expertise: Vec<DerivedFact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub genre: Option<GenreConfidence>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CandidatesGoals {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<DerivedFact>,
}

/// The `candidates:` block (ARCH §4.2 `DerivedFact` collection).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Candidates {
    #[serde(default)]
    pub identity: CandidatesIdentity,
    #[serde(default)]
    pub goals: CandidatesGoals,
}

/// A genre guess with confidence + provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenreConfidence {
    pub value: Genre,
    pub confidence: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provenance: Vec<String>,
}

// ──────────────────────────────────────────────────────────────────────────
// Derived (inverse) block
// ──────────────────────────────────────────────────────────────────────────

/// Inverse-derived values (B6 `derive_inverse_relations`). Free-form map keyed
/// by semantic role, e.g. `review_checklist`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DerivedBlock {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub entries: BTreeMap<String, DerivedFact>,
}

// ──────────────────────────────────────────────────────────────────────────
// Provider preference (B14)
// ──────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProviderPreference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_family: Option<String>,
    #[serde(default)]
    pub capability_constraints: CapabilityConstraints,
    /// truth | derived
    #[serde(default)]
    pub source: ProvenanceSource,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CapabilityConstraints {
    #[serde(default)]
    pub tool_use: ToolUseRequirement,
    #[serde(default)]
    pub context_window_min: Option<u32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolUseRequirement {
    #[default]
    Unspecified,
    Required,
    NotRequired,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProvenanceSource {
    Truth,
    #[default]
    Derived,
}

// ──────────────────────────────────────────────────────────────────────────
// Data sources (S1 autoscan) + interview meta
// ──────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DataSources {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<DataSource>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataSource {
    pub path: String,
    /// obsidian | git_repo | markdown_dir | text_dir
    pub kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidate_tags: Vec<String>,
    /// truth | derived
    #[serde(default)]
    pub tags_source: ProvenanceSource,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct InterviewMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub questions_asked: u32,
    #[serde(default)]
    pub questions_remaining: u32,
    /// 0.0..=1.0; interview halts above 0.7.
    #[serde(default)]
    pub fatigue_score: f64,
    #[serde(default)]
    pub contradictions_detected: u32,
    /// Per-axis completion; threshold 0.7.
    #[serde(default)]
    pub axis_completion: AxisCompletion,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AxisCompletion {
    #[serde(default)]
    pub tacit: f64,
    #[serde(default)]
    pub data: f64,
    #[serde(default)]
    pub genre: f64,
    #[serde(default)]
    pub goals: f64,
}

impl AxisCompletion {
    pub const THRESHOLD: f64 = 0.7;
    pub fn all_above_threshold(&self) -> bool {
        [self.tacit, self.data, self.genre, self.goals]
            .iter()
            .all(|v| *v >= Self::THRESHOLD)
    }
}

// ──────────────────────────────────────────────────────────────────────────
// The profile itself
// ──────────────────────────────────────────────────────────────────────────

/// Full user profile (YAML-serialized to `~/.byoh/profiles/<slug>.yaml`).
///
/// Aliases align with both ARCH §4.2 (`profile_status`) and Interview §6 (`status`).
/// `status` is the serialized canonical field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
    pub profile_version: String,

    #[serde(default = "default_status")]
    pub status: ProfileStatus,

    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,

    /// ko | en (B17)
    #[serde(default = "default_lang")]
    pub language: String,

    pub slug: String,

    #[serde(default)]
    pub truth: TruthBlock,
    #[serde(default)]
    pub candidates: Candidates,
    #[serde(default)]
    pub derived: DerivedBlock,
    #[serde(default)]
    pub data_sources: DataSources,

    #[serde(default)]
    pub provider_preference: ProviderPreference,

    #[serde(default)]
    pub interview_meta: InterviewMeta,

    /// Evolution policy (B10) — only meaningful once `status = evolving`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evolution_policy: Option<EvolutionPolicyConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvolutionPolicyConfig {
    pub enabled: bool,
    /// MUST contain all three for the evolution gate (R11).
    pub safety_gates: Vec<String>,
    #[serde(default = "default_stagnation_limit")]
    pub stagnation_limit: u32,
    #[serde(default = "default_improvement_threshold")]
    pub improvement_threshold: f64,
}

fn default_status() -> ProfileStatus {
    ProfileStatus::Draft
}
fn default_lang() -> String {
    "ko".to_string()
}
fn default_stagnation_limit() -> u32 {
    3
}
fn default_improvement_threshold() -> f64 {
    0.02
}

impl UserProfile {
    /// Create a fresh draft profile (M0: empty candidates).
    pub fn new_draft(slug: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            profile_version: "1.0".to_string(),
            status: ProfileStatus::Draft,
            updated_at: Some(Utc::now()),
            language: language.into(),
            slug: slug.into(),
            truth: TruthBlock::default(),
            candidates: Candidates::default(),
            derived: DerivedBlock::default(),
            data_sources: DataSources::default(),
            provider_preference: ProviderPreference::default(),
            interview_meta: InterviewMeta::default(),
            evolution_policy: None,
        }
    }

    /// Transition the state machine in place, stamping `updated_at`.
    pub fn advance(&mut self, to: ProfileStatus) -> super::Result<()> {
        self.status = self.status.transition(to)?;
        self.updated_at = Some(Utc::now());
        Ok(())
    }

    /// All candidate facts below the re-question threshold — drive the interview.
    pub fn weak_candidates(&self) -> Vec<&DerivedFact> {
        let mut out = Vec::new();
        out.extend(
            self.candidates
                .identity
                .primary_expertise
                .iter()
                .filter(|f| f.needs_reevaluation()),
        );
        out.extend(
            self.candidates
                .goals
                .items
                .iter()
                .filter(|f| f.needs_reevaluation()),
        );
        out
    }

    /// Bump an axis completion value, clamped to [0,1].
    pub fn set_axis(&mut self, axis: Axis, value: f64) {
        let v = value.clamp(0.0, 1.0);
        match axis {
            Axis::Tacit => self.interview_meta.axis_completion.tacit = v,
            Axis::Data => self.interview_meta.axis_completion.data = v,
            Axis::Genre => self.interview_meta.axis_completion.genre = v,
            Axis::Goals => self.interview_meta.axis_completion.goals = v,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Tacit,
    Data,
    Genre,
    Goals,
}

// ──────────────────────────────────────────────────────────────────────────
// Pipeline-stage type aliases (ARCH §3.1 S1/S2/S3 outputs)
// ──────────────────────────────────────────────────────────────────────────

/// S1 output. (DraftProfile == UserProfile at status=draft after autoscan.)
pub type DraftProfile = UserProfile;
/// S2 output (status=interviewed, truth filling).
pub type InterviewedProfile = UserProfile;
/// S3 output (status=confirmed, truth frozen).
pub type ConfirmedProfile = UserProfile;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ByohError;

    #[test]
    fn status_transitions_legal() {
        use ProfileStatus::*;
        assert!(Draft.transition(Interviewed).is_ok());
        assert!(Interviewed.transition(Confirmed).is_ok());
        assert!(Confirmed.transition(Evolving).is_ok());
        assert!(Evolving.allowed_next().is_empty());
    }

    #[test]
    fn status_transitions_illegal_rejected() {
        use ProfileStatus::*;
        // R2/AC2: draft → evolving must be rejected.
        let err = Draft.transition(Evolving).unwrap_err();
        assert!(matches!(err, ByohError::InvalidTransition { .. }));
        assert!(Confirmed.transition(Draft).is_err());
    }

    #[test]
    fn profile_roundtrip_yaml() {
        // AC2: round-trip preserves identity.
        let mut p = UserProfile::new_draft("creator-jane", "ko");
        p.truth.identity.domain = Some("Fiction".into());
        p.candidates.identity.genre = Some(GenreConfidence {
            value: Genre::Creator,
            confidence: 0.82,
            provenance: vec!["vault scan".into()],
        });
        p.advance(ProfileStatus::Interviewed).unwrap();

        let yaml = serde_yaml::to_string(&p).unwrap();
        let back: UserProfile = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(p, back);
        assert_eq!(back.status, ProfileStatus::Interviewed);
        assert_eq!(back.slug, "creator-jane");
    }

    #[test]
    fn weak_candidates_below_threshold() {
        let mut p = UserProfile::new_draft("d", "ko");
        p.candidates.identity.primary_expertise.push(DerivedFact {
            value: "strong".into(),
            confidence: 0.9,
            provenance: vec![],
        });
        p.candidates.identity.primary_expertise.push(DerivedFact {
            value: "weak".into(),
            confidence: 0.4,
            provenance: vec![],
        });
        let weak = p.weak_candidates();
        assert_eq!(weak.len(), 1);
        assert_eq!(weak[0].value, "weak");
    }
}

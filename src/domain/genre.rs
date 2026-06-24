//! Genre taxonomy + template skeleton.

use serde::{Deserialize, Serialize};

/// The four genres. MVP ships `Developer` + `Creator`; `Researcher` + `Business`
/// are extensions (M5) using the identical inheritance mechanism (ARCH §6.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Genre {
    Developer,
    Creator,
    Researcher,
    Business,
}

impl Genre {
    pub fn as_str(self) -> &'static str {
        use Genre::*;
        match self {
            Developer => "developer",
            Creator => "creator",
            Researcher => "researcher",
            Business => "business",
        }
    }

    pub fn all() -> &'static [Genre] {
        use Genre::*;
        &[Developer, Creator, Researcher, Business]
    }

    pub fn is_mvp(self) -> bool {
        matches!(self, Genre::Developer | Genre::Creator)
    }
}

impl std::fmt::Display for Genre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Genre {
    type Err = super::ByohError;
    fn from_str(s: &str) -> super::Result<Self> {
        use Genre::*;
        Ok(match s {
            "developer" => Developer,
            "creator" => Creator,
            "researcher" => Researcher,
            "business" => Business,
            other => return Err(super::ByohError::Schema(format!("unknown genre '{other}'"))),
        })
    }
}

/// B10 — the three mandatory safety gates. ALL must be present for evolution
/// (ARCH §1.4 invariant, R11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SafetyGate {
    Critic,
    Seesaw,
    Stagnation,
}

impl SafetyGate {
    pub const ALL: [SafetyGate; 3] = [
        SafetyGate::Critic,
        SafetyGate::Seesaw,
        SafetyGate::Stagnation,
    ];

    pub fn as_str(self) -> &'static str {
        use SafetyGate::*;
        match self {
            Critic => "critic",
            Seesaw => "seesaw",
            Stagnation => "stagnation",
        }
    }

    /// Validate that a list of gate names contains all three (R11/AC10).
    pub fn validate_all_present(names: &[String]) -> super::Result<()> {
        for gate in Self::ALL {
            if !names.iter().any(|n| n == gate.as_str()) {
                return Err(super::ByohError::SafetyGateMissing {
                    gate: gate.as_str(),
                });
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for SafetyGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Serialized form of safety gates in evolution_policy.toml.
pub type SafetyGates = Vec<String>;

/// Genre-specific evolution parameters (ARCH §7.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenreEvolutionParams {
    pub genre: Genre,
    pub improvement_threshold: f64,
    pub stagnation_limit: u32,
    /// Critic weight multiplier (business is higher).
    pub critic_weight: f64,
}

impl GenreEvolutionParams {
    pub fn for_genre(g: Genre) -> Self {
        use Genre::*;
        match g {
            Developer => Self {
                genre: g,
                improvement_threshold: 0.02,
                stagnation_limit: 3,
                critic_weight: 1.0,
            },
            Creator => Self {
                genre: g,
                improvement_threshold: 0.02,
                stagnation_limit: 5, // slower creative progress
                critic_weight: 1.0,
            },
            Researcher => Self {
                genre: g,
                improvement_threshold: 0.02,
                stagnation_limit: 4,
                critic_weight: 1.0,
            },
            Business => Self {
                genre: g,
                improvement_threshold: 0.02,
                stagnation_limit: 3,
                critic_weight: 1.5, // wrong-decision evolution is costly
            },
        }
    }
}

/// A genre template (base + child). See `crate::templates` for the library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenreTemplate {
    pub name: String,
    pub extends: Option<String>,
    pub genre: Genre,
    pub mvp: bool,
    /// Ring 0-3 skill/hook/tool identifiers this template contributes/overrides.
    pub rings: TemplateRings,
    pub tool_blueprints: Vec<String>,
    pub evolution: GenreEvolutionParams,
    pub description_en: String,
    pub description_ko: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TemplateRings {
    /// Skill ids per ring (ARCH §5.2).
    #[serde(default)]
    pub ring0_hooks: Vec<String>,
    #[serde(default)]
    pub ring1_pipeline: Vec<String>,
    #[serde(default)]
    pub ring2_quality: Vec<String>,
    #[serde(default)]
    pub ring3_evolution: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ByohError;

    #[test]
    fn safety_gates_require_all_three() {
        assert!(SafetyGate::validate_all_present(&[
            "critic".into(),
            "seesaw".into(),
            "stagnation".into()
        ])
        .is_ok());

        let err =
            SafetyGate::validate_all_present(&["critic".into(), "seesaw".into()]).unwrap_err();
        assert!(matches!(
            err,
            ByohError::SafetyGateMissing { gate: "stagnation" }
        ));
    }

    #[test]
    fn genre_parse_roundtrip() {
        for g in Genre::all() {
            let s = g.as_str();
            let back: Genre = s.parse().unwrap();
            assert_eq!(*g, back);
        }
    }
}

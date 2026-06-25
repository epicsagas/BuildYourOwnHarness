//! Domain error type. Single enum, `thiserror`-derived, convertible to `anyhow::Error`.

use thiserror::Error;

/// All BYOH errors funnel through here. Granular variants let the compiler /
/// evolution gates reject bundles with precise reasons (R8, R11).
#[derive(Debug, Error)]
pub enum ByohError {
    #[error("invalid profile state transition: {from:?} → {to:?} (allowed: {allowed})")]
    InvalidTransition {
        from: crate::domain::profile::ProfileStatus,
        to: crate::domain::profile::ProfileStatus,
        allowed: &'static str,
    },

    #[error("static validation gate failed ({gate}): {reason}")]
    ValidationGateFailed { gate: &'static str, reason: String },

    #[error("safety gate missing: {gate} — all three (critic, seesaw, stagnation) are mandatory")]
    SafetyGateMissing { gate: &'static str },

    #[error("profile schema error: {0}")]
    Schema(String),

    #[error("required truth field missing: {field}")]
    MissingTruth { field: &'static str },

    #[error("dependency tool not installed: {tool} (graceful fallback available)")]
    DependencyMissing { tool: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serde yaml: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),

    #[error("serde json: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("serde toml: {0}")]
    SerdeToml(#[from] toml::de::Error),

    #[error("toml serialization: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("{0}")]
    Other(String),
}

impl From<ByohError> for std::io::Error {
    fn from(e: ByohError) -> Self {
        match e {
            ByohError::Io(io) => io,
            other => std::io::Error::other(other.to_string()),
        }
    }
}

/// Crate-wide `Result` alias.
pub type Result<T> = std::result::Result<T, ByohError>;

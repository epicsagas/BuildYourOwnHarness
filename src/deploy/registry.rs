//! Static registry — compiled bundles become registry entries (ARCH §9.1, B16).

use std::collections::BTreeMap;

use crate::domain::bundle::HarnessBundle;
use crate::domain::genre::Genre;
use serde::{Deserialize, Serialize};

/// One registry entry (B16 `App` structure extension, ARCH §9.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: String,
    pub slug: String,
    pub genre: Genre,
    pub bundle_version: String,
    pub source_profile_hash: String,
    pub install_methods: Vec<String>,
    pub depends_on: Vec<DependencyRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencyRef {
    pub id: String,
    pub min_version: String,
}

/// In-memory registry (persisted as `byoh-registry.json`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Registry {
    pub entries: BTreeMap<String, RegistryEntry>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Register a compiled bundle.
    pub fn register(&mut self, bundle: &HarnessBundle) -> RegistryEntry {
        let entry = RegistryEntry {
            id: format!("byoh-{}", bundle.slug),
            slug: bundle.slug.clone(),
            genre: bundle.genre,
            bundle_version: bundle.version.as_string(),
            source_profile_hash: bundle.source_profile_hash.clone(),
            install_methods: vec!["script".into(), "cargo-binstall".into()],
            depends_on: bundle
                .config
                .depends_on
                .iter()
                .map(|d| DependencyRef {
                    id: d.id.clone(),
                    min_version: d.min_version.clone(),
                })
                .collect(),
        };
        self.entries.insert(entry.id.clone(), entry.clone());
        entry
    }

    pub fn lookup(&self, slug: &str) -> Option<&RegistryEntry> {
        self.entries.get(&format!("byoh-{slug}"))
    }

    pub fn to_json(&self) -> crate::domain::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::genre::Genre;
    use crate::domain::profile::{GenreConfidence, ProfileStatus, UserProfile};

    fn bundle() -> HarnessBundle {
        let mut p = UserProfile::new_draft("dev1", "en");
        p.candidates.identity.genre = Some(GenreConfidence {
            value: Genre::Developer,
            confidence: 1.0,
            provenance: vec![],
        });
        p.status = ProfileStatus::Confirmed;
        crate::compiler::compile_profile(&p).unwrap()
    }

    #[test]
    fn register_and_lookup() {
        let mut reg = Registry::new();
        let b = bundle();
        let entry = reg.register(&b);
        assert_eq!(entry.slug, "dev1");
        assert_eq!(entry.genre, Genre::Developer);
        assert!(reg.lookup("dev1").is_some());
    }

    #[test]
    fn registry_serializes_to_json() {
        let mut reg = Registry::new();
        reg.register(&bundle());
        let json = reg.to_json().unwrap();
        assert!(json.contains("byoh-dev1"));
    }
}

//! Index manifest — tracks what's in a persisted genre index so the next
//! `index` run can re-embed only what changed (incremental reindex) and report
//! staleness.
//!
//! Sidecar at `<root>/indexes/<genre>.manifest.json`: `doc_id → {hash, chunks}`,
//! where `hash` is the SHA-256 of the doc's text. Content-hash based (not
//! mtime) so it's stable across checkouts/copies.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::domain::genre::Genre;
use crate::rag::pipeline::InputDoc;

/// Current manifest schema version.
pub const MANIFEST_VERSION: u32 = 1;

/// Per-document record in the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocEntry {
    /// SHA-256 hex of the document text.
    pub hash: String,
    /// Number of chunks this doc produced.
    #[serde(default)]
    pub chunks: usize,
}

/// What an index knows about its source docs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexManifest {
    #[serde(default = "default_version")]
    pub version: u32,
    /// doc_id → entry.
    #[serde(default)]
    pub entries: BTreeMap<String, DocEntry>,
}

fn default_version() -> u32 {
    MANIFEST_VERSION
}

impl Default for IndexManifest {
    fn default() -> Self {
        Self {
            version: MANIFEST_VERSION,
            entries: BTreeMap::new(),
        }
    }
}

/// SHA-256 hex of a document's text — the change-detection key.
pub fn doc_hash(text: &str) -> String {
    let mut h = Sha256::new();
    h.update(text.as_bytes());
    format!("{:x}", h.finalize())
}

/// Manifest sidecar path next to the genre index.
pub fn manifest_path(root: &Path, genre: Genre) -> PathBuf {
    root.join("indexes")
        .join(format!("{}.manifest.json", genre.as_str()))
}

impl IndexManifest {
    /// Build a manifest from docs + their chunk counts.
    pub fn from_docs(docs: &[InputDoc], chunk_counts: &BTreeMap<String, usize>) -> Self {
        let mut entries = BTreeMap::new();
        for doc in docs {
            entries.insert(
                doc.id.clone(),
                DocEntry {
                    hash: doc_hash(&doc.text),
                    chunks: chunk_counts.get(&doc.id).copied().unwrap_or(0),
                },
            );
        }
        Self {
            version: MANIFEST_VERSION,
            entries,
        }
    }

    /// Load the manifest sidecar, or `None` if absent.
    pub fn load(root: &Path, genre: Genre) -> crate::domain::Result<Option<Self>> {
        let path = manifest_path(root, genre);
        if !path.exists() {
            return Ok(None);
        }
        let body = std::fs::read_to_string(&path)?;
        Ok(Some(serde_json::from_str(&body)?))
    }

    /// Persist the manifest sidecar (atomic temp+rename).
    pub fn save(&self, root: &Path, genre: Genre) -> crate::domain::Result<()> {
        let path = manifest_path(root, genre);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }
}

/// The diff between a manifest and the current set of docs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IndexDelta {
    /// doc ids present now but not in the manifest.
    pub added: Vec<String>,
    /// doc ids present in both, with a different hash.
    pub changed: Vec<String>,
    /// doc ids in the manifest but absent now.
    pub removed: Vec<String>,
    /// doc ids present in both with the same hash.
    pub unchanged: Vec<String>,
}

impl IndexDelta {
    /// True when nothing changed (index is fresh w.r.t. the docs).
    pub fn is_fresh(&self) -> bool {
        self.added.is_empty() && self.changed.is_empty() && self.removed.is_empty()
    }

    /// doc ids that must be (re-)embedded: added ∪ changed.
    pub fn to_embed(&self) -> Vec<String> {
        self.added
            .iter()
            .chain(self.changed.iter())
            .cloned()
            .collect()
    }

    /// One-line summary, e.g. "+2 ~1 -0 (3 unchanged)".
    pub fn summary(&self) -> String {
        format!(
            "+{} ~{} -{} ({} unchanged)",
            self.added.len(),
            self.changed.len(),
            self.removed.len(),
            self.unchanged.len()
        )
    }
}

/// Compare a (loaded or empty) manifest against the current docs.
pub fn diff(manifest: &IndexManifest, docs: &[InputDoc]) -> IndexDelta {
    let mut delta = IndexDelta::default();
    let current: BTreeMap<&str, String> = docs
        .iter()
        .map(|d| (d.id.as_str(), doc_hash(&d.text)))
        .collect();

    for (id, hash) in &current {
        match manifest.entries.get(*id) {
            None => delta.added.push((*id).to_string()),
            Some(e) if &e.hash != hash => delta.changed.push((*id).to_string()),
            Some(_) => delta.unchanged.push((*id).to_string()),
        }
    }
    for id in manifest.entries.keys() {
        if !current.contains_key(id.as_str()) {
            delta.removed.push(id.clone());
        }
    }
    delta.added.sort();
    delta.changed.sort();
    delta.removed.sort();
    delta.unchanged.sort();
    delta
}

/// Convenience: load the manifest for `genre` (empty if none) and diff it
/// against `docs` — the public "staleness check".
pub fn index_status(
    root: &Path,
    genre: Genre,
    docs: &[InputDoc],
) -> crate::domain::Result<IndexDelta> {
    let manifest = IndexManifest::load(root, genre)?.unwrap_or_default();
    Ok(diff(&manifest, docs))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn docs(pairs: &[(&str, &str)]) -> Vec<InputDoc> {
        pairs
            .iter()
            .map(|(id, text)| InputDoc {
                id: (*id).into(),
                text: (*text).into(),
            })
            .collect()
    }

    #[test]
    fn manifest_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let mut counts = BTreeMap::new();
        counts.insert("a".to_string(), 2usize);
        let m = IndexManifest::from_docs(&docs(&[("a", "hello")]), &counts);
        m.save(dir.path(), Genre::Developer).unwrap();
        let loaded = IndexManifest::load(dir.path(), Genre::Developer)
            .unwrap()
            .unwrap();
        assert_eq!(loaded, m);
        assert_eq!(loaded.entries["a"].chunks, 2);
    }

    #[test]
    fn load_none_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        assert!(IndexManifest::load(dir.path(), Genre::Developer)
            .unwrap()
            .is_none());
    }

    #[test]
    fn diff_classifies_added_changed_removed_unchanged() {
        // manifest has a(v1), b, c
        let mut counts = BTreeMap::new();
        for id in ["a", "b", "c"] {
            counts.insert(id.to_string(), 1usize);
        }
        let manifest =
            IndexManifest::from_docs(&docs(&[("a", "a-v1"), ("b", "b"), ("c", "c")]), &counts);
        // now: a(v2 changed), b(unchanged), d(added); c removed
        let now = docs(&[("a", "a-v2"), ("b", "b"), ("d", "d")]);
        let delta = diff(&manifest, &now);
        assert_eq!(delta.added, vec!["d"]);
        assert_eq!(delta.changed, vec!["a"]);
        assert_eq!(delta.removed, vec!["c"]);
        assert_eq!(delta.unchanged, vec!["b"]);
        assert!(!delta.is_fresh());
        // to_embed = added ∪ changed (added first): ["d", "a"]
        assert_eq!(delta.to_embed(), vec!["d".to_string(), "a".to_string()]);
    }

    #[test]
    fn fresh_when_identical() {
        let mut counts = BTreeMap::new();
        counts.insert("a".to_string(), 1usize);
        let manifest = IndexManifest::from_docs(&docs(&[("a", "x")]), &counts);
        let delta = diff(&manifest, &docs(&[("a", "x")]));
        assert!(delta.is_fresh());
        assert_eq!(delta.unchanged, vec!["a"]);
    }

    #[test]
    fn empty_manifest_all_added() {
        let delta = diff(&IndexManifest::default(), &docs(&[("a", "x"), ("b", "y")]));
        assert_eq!(delta.added.len(), 2);
        assert!(delta.unchanged.is_empty());
    }
}

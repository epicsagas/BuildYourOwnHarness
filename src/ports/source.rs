//! Profile source port — S1 autoscan (B5 hybrid search, B6 derived tagging).

use std::path::Path;

use crate::domain::profile::DataSource;

/// One scan hit: a keyword/topic candidate extracted non-destructively.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanHit {
    pub term: String,
    pub provenance: String,
    pub kind: String,
    pub tags: Vec<String>,
}

/// Non-destructive local resource scan (B1: read-only; never move/modify).
pub trait ProfileSource {
    /// Scan the given paths, returning keyword/topic candidates with provenance.
    /// `vector` → `bm25` → `grep` fallback is the implementation's concern.
    fn scan(&self, paths: &[&Path]) -> crate::domain::Result<Vec<ScanHit>>;

    /// Classify a path into a `DataSource` kind.
    fn classify(&self, path: &Path) -> DataSource;
}

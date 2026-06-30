//! awesomeclaudeplugins.com catalog — offline-first local cache.
//!
//! Network is only touched by `catalog index` (`src/catalog/index.rs`, feature-gated
//! to `catalog`). Everything else — search, vendor — reads `~/.byoh/catalog.json`.

pub mod search;
pub mod vendor_from_catalog;

pub mod index;

use crate::domain::genre::Genre;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// One awesomeclaudeplugins.com plugin entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogEntry {
    /// "owner/repo" slug (matches the site URL path).
    pub id: String,
    pub name: String,
    pub description: String,
    /// GitHub topic keywords (comma-split from JSON-LD `keywords` field).
    pub keywords: Vec<String>,
    pub github_url: String,
    pub stars: Option<u64>,
    pub license: String,
    /// BYOH genre inferred from keywords (`None` when nothing matched).
    pub byoh_genre: Option<Genre>,
    /// Unix seconds — when this entry was fetched (TTL tracking).
    pub fetched_at: u64,
}

/// Full catalog cache persisted at `~/.byoh/catalog.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CatalogCache {
    /// Cache-format schema version. `0` means unknown/legacy (a cache written
    /// before this field existed) — treated as stale so it naturally rebuilds.
    /// Remote bundles set this to [`crate::catalog::index::CATALOG_SCHEMA_VERSION`].
    #[serde(default)]
    pub schema_version: u32,
    /// Unix seconds — when the index was last built.
    pub built_at: u64,
    pub entries: Vec<CatalogEntry>,
}

/// `~/.byoh/catalog.json` path.
pub fn catalog_path(home: &Path) -> PathBuf {
    home.join("catalog.json")
}

/// Load cache — missing/empty file → empty cache (never an error).
pub fn load_cache(home: &Path) -> crate::Result<CatalogCache> {
    let p = catalog_path(home);
    match std::fs::read_to_string(&p) {
        Ok(s) if s.trim().is_empty() => Ok(CatalogCache::default()),
        Ok(s) => serde_json::from_str(&s)
            .map_err(|e| crate::domain::ByohError::Schema(format!("catalog.json parse: {e}"))),
        Err(_) => Ok(CatalogCache::default()),
    }
}

/// Save cache (creates parent dir as needed).
pub fn save_cache(home: &Path, cache: &CatalogCache) -> crate::Result<()> {
    std::fs::create_dir_all(home)?;
    let p = catalog_path(home);
    let body = serde_json::to_vec_pretty(cache)
        .map_err(|e| crate::domain::ByohError::Schema(format!("catalog.json serialize: {e}")))?;
    std::fs::write(&p, body)?;
    Ok(())
}

/// Returns true when `built_at + ttl_hours * 3600 > now`.
/// `ttl_hours = 0` always returns false (force-refresh semantics).
pub fn cache_is_fresh(cache: &CatalogCache, ttl_hours: u64) -> bool {
    if cache.built_at == 0 || ttl_hours == 0 {
        return false;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    cache.built_at + ttl_hours * 3600 > now
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_cache_missing_file_returns_default() {
        let dir = tempdir().unwrap();
        let c = load_cache(dir.path()).unwrap();
        assert!(c.entries.is_empty());
        assert_eq!(c.built_at, 0);
    }

    #[test]
    fn save_load_roundtrip() {
        let dir = tempdir().unwrap();
        let cache = CatalogCache {
            built_at: 9999,
            schema_version: crate::catalog::index::CATALOG_SCHEMA_VERSION,
            entries: vec![CatalogEntry {
                id: "obra/superpowers".into(),
                name: "superpowers".into(),
                description: "An agentic skills framework".into(),
                keywords: vec!["ai".into(), "skills".into()],
                github_url: "https://github.com/obra/superpowers".into(),
                stars: Some(241_000),
                license: "MIT".into(),
                byoh_genre: Some(Genre::Developer),
                fetched_at: 9999,
            }],
        };
        save_cache(dir.path(), &cache).unwrap();
        let loaded = load_cache(dir.path()).unwrap();
        assert_eq!(loaded.built_at, 9999);
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].id, "obra/superpowers");
        assert_eq!(loaded.entries[0].byoh_genre, Some(Genre::Developer));
    }

    #[test]
    fn cache_is_fresh_zero_built_at() {
        let c = CatalogCache::default();
        assert!(!cache_is_fresh(&c, 24));
    }

    #[test]
    fn cache_is_fresh_future_built_at() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let c = CatalogCache {
            built_at: now,
            entries: vec![],
            ..Default::default()
        };
        assert!(cache_is_fresh(&c, 24));
        assert!(!cache_is_fresh(&c, 0));
    }
}

//! Plugin catalog — offline-first local cache.
//!
//! Source: the curated `quemsah/awesome-claude-plugins` README (top 100 by
//! stars). Network is only touched by `catalog index` (`src/catalog/index.rs`)
//! and the remote-bundle path. Everything else — search, vendor — reads
//! `~/.byoh/catalog.json`.

pub mod search;
pub mod vendor_from_catalog;

pub mod index;

use crate::domain::genre::Genre;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// One plugin entry from the curated catalog.
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

/// Curated companion-tool entries — the epiccounty execution-layer tools that
/// BYOH-generated MCP tool descriptions reference as example backends (e.g.
/// "Backed by the user's knowledge base (e.g. alcove)"). These are **reference
/// material**, not hardcoded dependencies: they surface in `catalog_search`
/// results so an agent can recommend them contextually, and the user can vendor
/// one with `catalog vendor` / `vendor add` if they actually want it.
///
/// `byoh_genre = None` on the cross-cutting tools (alcove, obsidian-forge) so
/// they pass through every genre filter — researchers and developers both use
/// doc servers and vault automation.
pub fn curated_seeds() -> Vec<CatalogEntry> {
    vec![
        CatalogEntry {
            id: "epicsagas/alcove".into(),
            name: "alcove".into(),
            description: "Private doc server with MCP tools — search backend for the \
                          search_citations / search_code tools a BYOH harness can declare."
                .into(),
            keywords: vec!["docs".into(), "search".into(), "mcp".into(), "rag".into()],
            github_url: "https://github.com/epicsagas/alcove".into(),
            stars: None,
            license: "Apache-2.0".into(),
            byoh_genre: None,
            fetched_at: 0,
        },
        CatalogEntry {
            id: "epicsagas/obsidian-forge".into(),
            name: "obsidian-forge".into(),
            description: "Obsidian vault automation — collection, PARA routing, graph \
                          strengthening. The collection half of a BYOH harness's gather loop."
                .into(),
            keywords: vec![
                "vault".into(),
                "obsidian".into(),
                "notes".into(),
                "para".into(),
            ],
            github_url: "https://github.com/epicsagas/obsidian-forge".into(),
            stars: None,
            license: "Apache-2.0".into(),
            byoh_genre: None,
            fetched_at: 0,
        },
        CatalogEntry {
            id: "epicsagas/epic-harness".into(),
            name: "epic-harness".into(),
            description: "Central AI agent harness — Ring 0 hooks and the 4-section skill \
                          format BYOH bundles follow. A reference runtime, not a requirement."
                .into(),
            keywords: vec![
                "harness".into(),
                "hooks".into(),
                "skills".into(),
                "runtime".into(),
            ],
            github_url: "https://github.com/epicsagas/epic-harness".into(),
            stars: None,
            license: "Apache-2.0".into(),
            byoh_genre: Some(Genre::Developer),
            fetched_at: 0,
        },
    ]
}

/// Merge [`curated_seeds`] into `cache` in place, skipping any seed whose `id`
/// is already present (the user's indexed data wins). Used as a read-time
/// overlay in `catalog_search` only — the on-disk cache stays pure indexed data.
pub fn merge_curated_seeds(cache: &mut CatalogCache) {
    for seed in curated_seeds() {
        if !cache.entries.iter().any(|e| e.id == seed.id) {
            cache.entries.push(seed);
        }
    }
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
    fn curated_seeds_merge_without_polluting_disk() {
        // Seeds are a read-time overlay: an empty on-disk cache gains the seeds
        // after merge, but the file on disk is never written by merge alone.
        let dir = tempdir().unwrap();
        let on_disk = load_cache(dir.path()).unwrap();
        assert!(on_disk.entries.is_empty(), "no seed has been persisted");

        let mut merged = on_disk.clone();
        merge_curated_seeds(&mut merged);
        let ids: Vec<&str> = merged.entries.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"epicsagas/alcove"));
        assert!(ids.contains(&"epicsagas/obsidian-forge"));
        assert!(ids.contains(&"epicsagas/epic-harness"));

        // The disk file is still absent / empty — merge never wrote it.
        assert!(load_cache(dir.path()).unwrap().entries.is_empty());
    }

    #[test]
    fn merge_does_not_duplicate_an_already_indexed_seed() {
        // If the user indexed alcove themselves, the merge must not duplicate it.
        let mut cache = CatalogCache::default();
        cache.entries.push(CatalogEntry {
            id: "epicsagas/alcove".into(),
            name: "alcove (user-indexed)".into(),
            description: "mine".into(),
            keywords: vec![],
            github_url: "https://github.com/epicsagas/alcove".into(),
            stars: Some(42),
            license: "Apache-2.0".into(),
            byoh_genre: None,
            fetched_at: 1,
        });
        merge_curated_seeds(&mut cache);
        let alcoves: Vec<_> = cache
            .entries
            .iter()
            .filter(|e| e.id == "epicsagas/alcove")
            .collect();
        assert_eq!(
            alcoves.len(),
            1,
            "user's indexed entry must win, no duplicate"
        );
        assert_eq!(
            alcoves[0].stars,
            Some(42),
            "the kept entry is the user's, not the seed"
        );
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

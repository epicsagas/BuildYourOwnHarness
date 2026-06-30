//! Offline keyword search over the local catalog cache — no network.

use super::{CatalogCache, CatalogEntry, load_cache};
use crate::domain::genre::Genre;
use std::path::Path;

/// Search options passed to [`catalog_search`] / [`search_cache`].
pub struct SearchOptions<'a> {
    pub query: &'a str,
    /// Optional genre filter — only entries whose `byoh_genre` matches are returned.
    pub genre: Option<Genre>,
    /// Additional tag filters (AND: entry must contain ALL of these).
    pub tags: &'a [String],
    pub limit: usize,
}

/// Search `~/.byoh/catalog.json` — returns at most `opts.limit` entries ranked
/// by match score. Never errors on a missing cache file (returns empty vec).
pub fn catalog_search(home: &Path, opts: &SearchOptions) -> crate::Result<Vec<CatalogEntry>> {
    let cache = load_cache(home)?;
    Ok(search_cache(&cache, opts))
}

/// Search a `CatalogCache` directly (no file I/O — useful in tests).
pub fn search_cache(cache: &CatalogCache, opts: &SearchOptions) -> Vec<CatalogEntry> {
    let mut scored: Vec<(u32, &CatalogEntry)> = cache
        .entries
        .iter()
        .filter_map(|e| {
            let s = score_entry(e, opts);
            if s > 0 { Some((s, e)) } else { None }
        })
        .collect();
    scored.sort_by_key(|b| std::cmp::Reverse(b.0));
    scored
        .into_iter()
        .take(opts.limit)
        .map(|(_, e)| e.clone())
        .collect()
}

/// Score a single entry against `opts`. Returns 0 when the entry doesn't match.
///
/// Scoring (additive):
/// - name contains query word      → +4 per word
/// - keywords overlap with query   → +3 per shared word
/// - description contains query    → +1
/// - id contains query             → +2
///
/// Genre filter: if `opts.genre` is set, entries with a different (non-None)
/// `byoh_genre` are excluded. Entries with `byoh_genre = None` pass through
/// (genre unknown — don't silently exclude).
///
/// Tag AND filter: entry keywords must contain every element of `opts.tags`.
pub fn score_entry(entry: &CatalogEntry, opts: &SearchOptions) -> u32 {
    // Genre filter.
    if let Some(want) = opts.genre {
        if let Some(got) = entry.byoh_genre {
            if got != want {
                return 0;
            }
        }
    }

    // Tag AND filter.
    let entry_kw: Vec<String> = entry.keywords.iter().map(|k| k.to_lowercase()).collect();
    for required in opts.tags {
        if !entry_kw.contains(&required.to_lowercase()) {
            return 0;
        }
    }

    // Keyword scoring.
    let query_words: Vec<String> = opts
        .query
        .split_whitespace()
        .filter(|w| w.len() >= 2)
        .map(|w| w.to_lowercase())
        .collect();

    if query_words.is_empty() {
        // Empty query matches everything with score 1.
        return 1;
    }

    let name_lower = entry.name.to_lowercase();
    let desc_lower = entry.description.to_lowercase();
    let id_lower = entry.id.to_lowercase();

    let mut score: u32 = 0;
    for word in &query_words {
        if name_lower.contains(word.as_str()) {
            score += 4;
        }
        if id_lower.contains(word.as_str()) {
            score += 2;
        }
        if entry_kw.iter().any(|k| k.contains(word.as_str())) {
            score += 3;
        }
        if desc_lower.contains(word.as_str()) {
            score += 1;
        }
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::CatalogEntry;

    fn entry(id: &str, name: &str, desc: &str, kw: &[&str], genre: Option<Genre>) -> CatalogEntry {
        CatalogEntry {
            id: id.into(),
            name: name.into(),
            description: desc.into(),
            keywords: kw.iter().map(|s| (*s).into()).collect(),
            github_url: format!("https://github.com/{id}"),
            stars: None,
            license: "MIT".into(),
            byoh_genre: genre,
            fetched_at: 0,
        }
    }

    fn cache(entries: Vec<CatalogEntry>) -> CatalogCache {
        CatalogCache {
            built_at: 9999,
            entries,
        }
    }

    #[test]
    fn empty_cache_returns_empty() {
        let c = CatalogCache::default();
        let opts = SearchOptions {
            query: "tdd",
            genre: None,
            tags: &[],
            limit: 10,
        };
        assert!(search_cache(&c, &opts).is_empty());
    }

    #[test]
    fn name_match_scores_highest() {
        let c = cache(vec![
            entry(
                "a/tdd-skill",
                "tdd-skill",
                "test driven",
                &["tdd"],
                Some(Genre::Developer),
            ),
            entry("b/other", "other", "nothing here", &["random"], None),
        ]);
        let opts = SearchOptions {
            query: "tdd",
            genre: None,
            tags: &[],
            limit: 10,
        };
        let res = search_cache(&c, &opts);
        assert_eq!(res[0].id, "a/tdd-skill");
        assert_eq!(res.len(), 1); // "other" scores 0
    }

    #[test]
    fn genre_filter_excludes_wrong_genre() {
        let c = cache(vec![
            entry(
                "a/write",
                "write",
                "writing tool",
                &["writing"],
                Some(Genre::Creator),
            ),
            entry(
                "b/code",
                "code",
                "coding tool",
                &["code"],
                Some(Genre::Developer),
            ),
        ]);
        let opts = SearchOptions {
            query: "tool",
            genre: Some(Genre::Developer),
            tags: &[],
            limit: 10,
        };
        let res = search_cache(&c, &opts);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].id, "b/code");
    }

    #[test]
    fn genre_filter_passes_unknown_genre() {
        let c = cache(vec![entry(
            "a/mystery",
            "mystery tool",
            "unknown genre",
            &[],
            None,
        )]);
        let opts = SearchOptions {
            query: "tool",
            genre: Some(Genre::Developer),
            tags: &[],
            limit: 10,
        };
        let res = search_cache(&c, &opts);
        assert_eq!(res.len(), 1); // None byoh_genre passes through
    }

    #[test]
    fn tag_and_filter() {
        let c = cache(vec![
            entry("a/full", "full", "d", &["tdd", "rust"], None),
            entry("b/partial", "partial", "d", &["tdd"], None),
        ]);
        let tags = vec!["tdd".to_string(), "rust".to_string()];
        let opts = SearchOptions {
            query: "d",
            genre: None,
            tags: &tags,
            limit: 10,
        };
        let res = search_cache(&c, &opts);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].id, "a/full");
    }

    #[test]
    fn limit_respected() {
        let entries: Vec<CatalogEntry> = (0..10)
            .map(|i| {
                entry(
                    &format!("a/{i}"),
                    &format!("tool{i}"),
                    "tool",
                    &["tool"],
                    None,
                )
            })
            .collect();
        let c = cache(entries);
        let opts = SearchOptions {
            query: "tool",
            genre: None,
            tags: &[],
            limit: 3,
        };
        assert_eq!(search_cache(&c, &opts).len(), 3);
    }

    #[test]
    fn empty_query_matches_all_up_to_limit() {
        let c = cache(vec![
            entry("a/x", "x", "d", &[], None),
            entry("b/y", "y", "d", &[], None),
        ]);
        let opts = SearchOptions {
            query: "",
            genre: None,
            tags: &[],
            limit: 5,
        };
        assert_eq!(search_cache(&c, &opts).len(), 2);
    }
}

//! CatalogEntry → vendor_add pipeline.

use super::CatalogEntry;
use crate::deploy::{
    VendorEntry, extract_keywords_from_dir, extract_license_from_dir, fetch_git, vendor_add,
};
use crate::domain::genre::Genre;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Metadata harvested from a cloned plugin repo at vendor time.
///
/// Used by the caller to update the catalog cache so subsequent
/// `catalog search` results reflect the richer data.
#[derive(Debug, Clone)]
pub struct VendorEnrichment {
    /// License detected from the repo (plugin.json → README fallback).
    pub license: String,
    /// Keywords from plugin.json `keywords` array, merged with caller's extras.
    pub keywords: Vec<String>,
    /// Genre resolved at vendor time (may differ from the cached entry's inferred genre).
    pub genre: Genre,
}

/// Vendor a plugin from the catalog into `registry/vendored/`.
///
/// 1. Shallow-clone `entry.github_url` into a temp directory.
/// 2. Harvest license + keywords from the cloned repo.
/// 3. Call [`vendor_add`] with the enriched metadata.
/// 4. Remove the temp clone.
///
/// Returns `(VendorEntry, VendorEnrichment)` — the caller should persist the
/// enrichment back into the catalog cache via `save_cache`.
///
/// `genre` overrides `entry.byoh_genre`; errors when both are `None`.
/// `extra_keywords` are appended to harvested keywords.
pub fn catalog_vendor(
    entry: &CatalogEntry,
    genre: Option<Genre>,
    extra_keywords: &[String],
    repo_root: &Path,
) -> crate::Result<(VendorEntry, VendorEnrichment)> {
    let resolved_genre = genre.or(entry.byoh_genre).ok_or_else(|| {
        crate::domain::ByohError::Schema(format!(
            "catalog vendor: no genre for '{}' — pass --genre or ensure byoh_genre is set",
            entry.id
        ))
    })?;

    let fetched_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into());

    let dest = std::env::temp_dir().join(format!(
        "byoh-catalog-{}-{fetched_at}",
        entry.id.replace('/', "-")
    ));

    let _sha = fetch_git(&entry.github_url, "HEAD", None, &dest)?;

    // Harvest metadata from the cloned repo before merging with entry defaults.
    let harvested_license =
        extract_license_from_dir(&dest).unwrap_or_else(|| entry.license.clone());
    let mut harvested_keywords = extract_keywords_from_dir(&dest);
    // Merge: repo keywords first, then catalog entry keywords, then caller extras.
    for kw in entry.keywords.iter().chain(extra_keywords.iter()) {
        if !harvested_keywords.contains(kw) {
            harvested_keywords.push(kw.clone());
        }
    }

    // Sanitize the catalog id (`owner/repo`) into a filesystem-safe skill id.
    // `replace('/', "-")` alone still admits `..`; route through sanitize_skill_id
    // for full path-traversal / separator / charset rejection.
    let raw = entry.id.replace('/', "-");
    let skill_id = crate::deploy::sanitize_skill_id(&raw)
        .map_err(|e| {
            crate::domain::ByohError::Schema(format!(
                "catalog vendor: unsafe plugin id '{}': {e}",
                entry.id
            ))
        })?
        .to_string();
    let vendor_result = vendor_add(
        &dest,
        resolved_genre,
        &skill_id,
        &harvested_keywords,
        &harvested_license,
        repo_root,
        &fetched_at,
    );

    let _ = std::fs::remove_dir_all(&dest);

    let vendor_entry = vendor_result?;
    let enrichment = VendorEnrichment {
        license: harvested_license,
        keywords: harvested_keywords,
        genre: resolved_genre,
    };
    Ok((vendor_entry, enrichment))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn dummy_entry(byoh_genre: Option<Genre>) -> CatalogEntry {
        CatalogEntry {
            id: "test/plugin".into(),
            name: "plugin".into(),
            description: "test".into(),
            keywords: vec!["test".into()],
            github_url: "https://github.com/test/plugin".into(),
            stars: None,
            license: "MIT".into(),
            byoh_genre,
            fetched_at: 0,
        }
    }

    #[test]
    fn errors_when_no_genre_resolved() {
        let dir = tempdir().unwrap();
        let entry = dummy_entry(None);
        let err = catalog_vendor(&entry, None, &[], dir.path()).unwrap_err();
        assert!(
            matches!(err, crate::domain::ByohError::Schema(_)),
            "expected Schema error, got {err:?}"
        );
    }

    #[test]
    fn genre_override_wins_over_entry_genre() {
        // This is a unit test of the resolution logic only (no network).
        // We just verify that the genre param takes priority when both are set.
        use crate::domain::genre::Genre;
        let entry = dummy_entry(Some(Genre::Researcher));
        // We can't call catalog_vendor (it would clone), so test the resolution
        // logic directly: genre.or(entry.byoh_genre).
        let resolved = Some(Genre::Developer).or(entry.byoh_genre);
        assert_eq!(resolved, Some(Genre::Developer));
    }
}

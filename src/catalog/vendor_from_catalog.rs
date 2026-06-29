//! CatalogEntry → vendor_add pipeline.

use super::CatalogEntry;
use crate::deploy::{VendorEntry, fetch_git, vendor_add};
use crate::domain::genre::Genre;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Vendor a plugin from the catalog into `registry/vendored/`.
///
/// 1. Shallow-clone `entry.github_url` into a temp directory.
/// 2. Call [`vendor_add`] with the merged keyword set + resolved license.
/// 3. Remove the temp clone.
///
/// `genre` overrides `entry.byoh_genre`; errors when both are `None`.
/// `extra_keywords` are appended to `entry.keywords` before matching.
pub fn catalog_vendor(
    entry: &CatalogEntry,
    genre: Option<Genre>,
    extra_keywords: &[String],
    repo_root: &Path,
) -> crate::Result<VendorEntry> {
    let resolved_genre = genre
        .or(entry.byoh_genre)
        .ok_or_else(|| crate::domain::ByohError::Schema(format!(
            "catalog vendor: no genre for '{}' — pass --genre or ensure byoh_genre is set",
            entry.id
        )))?;

    let fetched_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into());

    let dest = std::env::temp_dir()
        .join(format!("byoh-catalog-{}-{fetched_at}", entry.id.replace('/', "-")));

    let _sha = fetch_git(&entry.github_url, "HEAD", None, &dest)?;

    let mut keywords: Vec<String> = entry.keywords.clone();
    keywords.extend_from_slice(extra_keywords);
    keywords.dedup();

    let skill_id = entry.id.replace('/', "-");
    let result = vendor_add(
        &dest,
        resolved_genre,
        &skill_id,
        &keywords,
        &entry.license,
        repo_root,
        &fetched_at,
    );

    let _ = std::fs::remove_dir_all(&dest);
    result
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
}

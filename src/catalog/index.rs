//! Catalog indexer — parses the curated `quemsah/awesome-claude-plugins`
//! README (top 100 by stars) → CatalogCache. Network is touched only by
//! `catalog index` (this module) and the remote-bundle path.

use super::{CatalogCache, CatalogEntry, save_cache};
use crate::deploy::genre_map::infer_genre;
use crate::domain::ByohError;
use regex::Regex;
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Raw README that lists the top 100 Claude plugin repos by stars. Refreshed
/// daily upstream (`chore(data): refresh dataset`). A single GET + parse beats
/// crawling ~24 000 per-page HTML and gives us real ranking (stars).
const QUEMSAH_README_URL: &str =
    "https://raw.githubusercontent.com/quemsah/awesome-claude-plugins/main/README.md";

/// Catalog cache schema version this build understands. Bumped whenever the
/// `CatalogCache` shape changes in a backwards-incompatible way.
pub const CATALOG_SCHEMA_VERSION: u32 = 1;

/// Maintainer-built, gzip-compressed catalog bundle shipped as a GitHub Release
/// asset under the moving `catalog-latest` tag. `byoh catalog index` fetches
/// this first (seconds) and only falls back to re-parsing the README when the
/// bundle is unreachable. Hardcoded (no external input) so there is no SSRF
/// surface — the URL never varies.
const REMOTE_BUNDLE_URL: &str =
    "https://github.com/epicsagas/BuildYourOwnHarness/releases/download/catalog-latest/catalog.json.gz";

/// Accept only `https://github.com/...` URLs (SSRF allowlist for catalog
/// `github_url` values, whether from the quemsah README or a remote bundle).
/// Rejects `file://`, link-local/loopback hosts, non-https schemes, and hosts
/// that merely contain "github.com". Empty input is rejected.
pub fn is_safe_github_url(url: &str) -> bool {
    let url = url.trim();
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };
    // Strip userinfo / port: authority is everything up to the first '/'.
    let authority = rest.split('/').next().unwrap_or(rest);
    let host = authority.rsplit('@').next().unwrap_or(authority);
    let host = host.split(':').next().unwrap_or(host);
    host.eq_ignore_ascii_case("github.com")
}

/// Parse the quemsah `awesome-claude-plugins` README into catalog entries.
///
/// The README is a Markdown table sorted by stars:
/// `| # | [name](github_url) | description | Stars | Subs | Plugins |`.
/// Each data row becomes a `CatalogEntry`. The `#`, separator, and header rows
/// are ignored. `github_url` is validated with [`is_safe_github_url`]; any row
/// pointing elsewhere (`file://`, look-alike host) is skipped — untrusted
/// upstream markdown can never steer a later `git clone` at an internal target.
/// `Stars` populates `CatalogEntry.stars` (real ranking, finally). `Subs` and
/// `Plugins` have no target field and are dropped. Genre is inferred from the
/// `name + description` text via [`infer_genre`].
///
/// Pure function — no network — so it is unit-tested with embedded fixtures.
pub fn parse_quemsah_readme(md: &str) -> crate::Result<Vec<CatalogEntry>> {
    let fetched_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Data row: leading `| <digits> |`, then a `[name](url)` cell. The digits
    // anchor excludes the header (`| # |`) and separator (`|---|`) rows.
    // `(?s)` lets `.*?` cross newlines inside the description cell — but
    // descriptions are single-line in practice, so we bound to the line.
    let row_re = Regex::new(
        r#"(?m)^\|\s*\d+\s*\|\s*\[([^\]]*)\]\((https://[^)]+)\)\s*\|([^|]*)\|\s*(\d+)\s*\|"#,
    )
    .map_err(|e| ByohError::Other(format!("row regex: {e}")))?;

    let mut entries = Vec::new();
    for caps in row_re.captures_iter(md) {
        let name = caps[1].trim().to_string();
        let github_url = caps[2].trim().to_string();
        let description = caps[3].trim().to_string();
        let stars: u64 = match caps[4].parse() {
            Ok(n) => n,
            Err(_) => continue, // malformed stars cell — skip row
        };

        // SSRF gate: only https://github.com URLs survive.
        if !is_safe_github_url(&github_url) {
            continue;
        }

        // id = owner/repo from the GitHub URL path.
        let id = match github_url
            .strip_prefix("https://github.com/")
            .and_then(|p| p.split('?').next())
            .map(|p| p.trim_end_matches('/'))
        {
            Some(p) if !p.is_empty() && p.contains('/') => p.to_string(),
            _ => continue,
        };

        let byoh_genre = infer_genre(&format!("{name} {description}"));

        entries.push(CatalogEntry {
            id,
            name,
            description,
            keywords: Vec::new(),
            github_url,
            stars: Some(stars),
            license: "unknown".to_string(),
            byoh_genre,
            fetched_at,
        });
    }
    Ok(entries)
}

/// Decompress + parse a gzip-compressed catalog bundle (bytes of
/// `catalog.json.gz`) into a `CatalogCache`. Pure function — no network — so it
/// is unit-testable with embedded fixtures.
///
/// Rejects corrupt gzip, malformed JSON, and any `schema_version` other than
/// [`CATALOG_SCHEMA_VERSION`] (so a future incompatible bundle is not silently
/// loaded over a good local cache).
pub fn parse_remote_bundle(bytes: &[u8]) -> crate::Result<CatalogCache> {
    let decoder = flate2::read::GzDecoder::new(bytes);
    let cache: CatalogCache = serde_json::from_reader(decoder)
        .map_err(|e| ByohError::Schema(format!("bundle parse: {e}")))?;
    if cache.schema_version != CATALOG_SCHEMA_VERSION {
        return Err(ByohError::Schema(format!(
            "bundle schema version {} unsupported (expected {})",
            cache.schema_version, CATALOG_SCHEMA_VERSION
        )));
    }
    Ok(cache)
}

/// Resolve the bundle URL. Defaults to [`REMOTE_BUNDLE_URL`] but can be
/// overridden with the `BYOH_BUNDLE_URL` env var — this lets maintainers test
/// a locally-served bundle (e.g. `python3 -m http.server`) end-to-end without
/// waiting for a Release, and lets self-hosters point at their own mirror.
/// An empty env value falls back to the default.
fn bundle_url() -> String {
    bundle_url_from(std::env::var("BYOH_BUNDLE_URL").ok())
}

/// Pure core of [`bundle_url`]: given an optional env value, return the
/// override when non-empty (after trim), else the default. Separated so the
/// resolution logic is unit-testable without mutating process env (Edition
/// 2024 makes `set_var` `unsafe`, which this crate forbids).
fn bundle_url_from(env_value: Option<String>) -> String {
    match env_value {
        Some(v) if !v.trim().is_empty() => v,
        _ => REMOTE_BUNDLE_URL.to_string(),
    }
}

/// Fetch the raw bytes of the maintainer-built remote bundle. Thin network
/// wrapper around `ureq` — not unit-tested (matches the `catalog_index` /
/// `fetch_and_parse_entry` convention of keeping the pure logic separate).
fn fetch_remote_bundle() -> crate::Result<Vec<u8>> {
    let url = bundle_url();
    let resp = ureq::get(&url)
        .call()
        .map_err(|e| ByohError::Other(format!("bundle fetch: {e}")))?;
    let mut buf = Vec::new();
    resp.into_reader()
        .read_to_end(&mut buf)
        .map_err(|e| ByohError::Other(format!("bundle read: {e}")))?;
    Ok(buf)
}

/// Try to load the remote prebuilt catalog bundle.
///
/// Returns `Ok(Some(cache))` on success. Any network or parse failure returns
/// `Ok(None)` after logging a one-line notice — the caller then falls back to a
/// full crawl. This keeps `catalog index` resilient: a missing/broken bundle
/// never blocks the user, it just costs them a crawl.
pub fn try_remote_bundle() -> crate::Result<Option<CatalogCache>> {
    match fetch_remote_bundle() {
        Ok(bytes) => match parse_remote_bundle(&bytes) {
            Ok(cache) => Ok(Some(cache)),
            Err(e) => {
                eprintln!("[byoh catalog] bundle parse failed ({e}) — falling back to crawl");
                Ok(None)
            }
        },
        Err(e) => {
            eprintln!("[byoh catalog] bundle fetch failed ({e}) — falling back to crawl");
            Ok(None)
        }
    }
}

/// Index the plugin catalog (quemsah top-100 by stars).
///
/// 1. Fetches `sitemap.xml` → extracts plugin URLs (up to `limit`).
/// 2. For each URL: fetches the page, parses JSON-LD → `CatalogEntry`.
///    Failed pages are skipped with a warning (not a hard error).
/// 3. Saves `CatalogCache` to `~/.byoh/catalog.json`.
///
/// `progress_fn(fetched, total)` is called after each page.
pub fn catalog_index(
    home: &Path,
    limit: usize,
    _ttl_hours: u64,
    progress_fn: impl Fn(usize, usize),
) -> crate::Result<CatalogCache> {
    // 1. Fetch the curated README (top 100 by stars). One GET — no per-page crawl.
    let body = ureq::get(QUEMSAH_README_URL)
        .call()
        .map_err(|e| crate::domain::ByohError::Other(format!("readme fetch: {e}")))?
        .into_string()
        .map_err(|e| crate::domain::ByohError::Other(format!("readme read: {e}")))?;

    // 2. Parse the Markdown table → entries (sorted by stars upstream).
    let mut entries = parse_quemsah_readme(&body)?;
    // `--limit 0` means "all" (every row). Any other value caps to the top N.
    if limit > 0 {
        entries.truncate(limit);
    }
    let total = entries.len();
    progress_fn(total, total);

    let fetched_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let cache = CatalogCache {
        schema_version: CATALOG_SCHEMA_VERSION,
        built_at: fetched_at,
        entries,
    };
    save_cache(home, &cache)?;
    Ok(cache)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_safe_github_url (SSRF gate, now called directly by quemsah parser) ---

    #[test]
    fn is_safe_github_url_accepts_github() {
        assert!(is_safe_github_url("https://github.com/owner/repo"));
        assert!(is_safe_github_url("https://github.com/owner/repo.git"));
    }

    #[test]
    fn is_safe_github_url_rejects_ssrf() {
        for bad in [
            "file:///etc/passwd",
            "http://github.com/a/b",
            "https://evil.com/?x=github.com/y",
            "https://notgithub.com/a/b",
            "https://169.254.169.254/latest/meta-data/",
            "",
        ] {
            assert!(!is_safe_github_url(bad), "{bad} should be rejected");
        }
    }

    // --- quemsah README parser ---

    /// Minimal table fixture mimicking the real README shape.
    const QUEMSAH_FIXTURE: &str = "\
# Awesome Claude Code Plugins: Top 100 Repositories

| # | Repo Name | Description | Stars | Subs | Plugins |
|---|-----------|-------------|-------|------|---------|
| 1 | [superpowers](https://github.com/obra/superpowers) | An agentic skills framework. | 240626 | 899 | 1 |
| 2 | [context7](https://github.com/upstash/context7) | Up-to-date code documentation for LLMs. | 58251 | 151 | 1 |
| 3 | [malicious](file:///etc/passwd) | should be skipped (SSRF). | 999 | 1 | 1 |
| 4 | [notgithub](https://notgithub.com/a/b) | should be skipped (look-alike host). | 999 | 1 | 1 |
";

    #[test]
    fn parse_quemsah_readme_extracts_entries() {
        let entries = parse_quemsah_readme(QUEMSAH_FIXTURE).unwrap();
        // SSRF rows (file://, look-alike) are skipped → only 2 survive.
        assert_eq!(entries.len(), 2);
        let first = &entries[0];
        assert_eq!(first.id, "obra/superpowers");
        assert_eq!(first.name, "superpowers");
        assert_eq!(first.github_url, "https://github.com/obra/superpowers");
        assert_eq!(first.stars, Some(240626));
        assert!(first.description.contains("agentic skills"));
        assert_eq!(first.license, "unknown");
        assert!(first.keywords.is_empty());
    }

    #[test]
    fn parse_quemsah_readme_skips_non_github_rows() {
        let entries = parse_quemsah_readme(QUEMSAH_FIXTURE).unwrap();
        assert!(entries.iter().all(|e| e.github_url.starts_with("https://github.com/")));
        assert!(!entries.iter().any(|e| e.id.contains("passwd")));
    }

    #[test]
    fn parse_quemsah_readme_ignores_header_and_separator() {
        let entries = parse_quemsah_readme(QUEMSAH_FIXTURE).unwrap();
        assert!(entries.iter().all(|e| !e.name.is_empty()));
        assert!(!entries.iter().any(|e| e.name == "Repo Name"));
    }

    #[test]
    fn parse_quemsah_readme_infers_genre_from_description() {
        let entries = parse_quemsah_readme(QUEMSAH_FIXTURE).unwrap();
        let ctx = entries
            .iter()
            .find(|e| e.id == "upstash/context7")
            .expect("context7 entry");
        assert_eq!(
            ctx.byoh_genre,
            Some(crate::domain::genre::Genre::Developer)
        );
    }

    #[test]
    fn parse_quemsah_readme_returns_empty_for_non_table() {
        let entries = parse_quemsah_readme("# just a title\n\nno table here").unwrap();
        assert!(entries.is_empty());
    }

    // --- remote prebuilt bundle tests ---

    /// Gzip-encode a JSON string into bytes (mirrors how the CI workflow
    /// produces `catalog.json.gz`).
    fn gz(json: &str) -> Vec<u8> {
        use flate2::{Compression, write::GzEncoder};
        use std::io::Write;
        let mut enc = GzEncoder::new(Vec::new(), Compression::default());
        enc.write_all(json.as_bytes()).unwrap();
        enc.finish().unwrap()
    }

    #[test]
    fn parse_remote_bundle_parses_valid_gz() {
        let cache = CatalogCache {
            schema_version: CATALOG_SCHEMA_VERSION,
            built_at: 1234,
            entries: vec![CatalogEntry {
                id: "owner/repo".into(),
                name: "owner/repo".into(),
                description: "d".into(),
                keywords: vec!["coding".into()],
                github_url: "https://github.com/owner/repo".into(),
                stars: None,
                license: "unknown".into(),
                byoh_genre: None,
                fetched_at: 0,
            }],
        };
        let bytes = gz(&serde_json::to_string(&cache).unwrap());
        let parsed = parse_remote_bundle(&bytes).unwrap();
        assert_eq!(parsed.schema_version, CATALOG_SCHEMA_VERSION);
        assert_eq!(parsed.built_at, 1234);
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].id, "owner/repo");
    }

    #[test]
    fn parse_remote_bundle_rejects_bad_schema_version() {
        let json = r#"{"schema_version":99,"built_at":0,"entries":[]}"#;
        let err = parse_remote_bundle(&gz(json)).unwrap_err();
        assert!(matches!(err, ByohError::Schema(_)));
        assert!(format!("{err}").contains("unsupported"));
    }

    #[test]
    fn parse_remote_bundle_rejects_corrupt_gzip() {
        // Not a gzip stream at all.
        let err = parse_remote_bundle(b"definitely not gzip").unwrap_err();
        assert!(matches!(err, ByohError::Schema(_)));
    }

    #[test]
    fn parse_remote_bundle_rejects_bad_json() {
        // Valid gzip wrapping of non-JSON.
        let err = parse_remote_bundle(&gz("not json {")).unwrap_err();
        assert!(matches!(err, ByohError::Schema(_)));
    }

    #[test]
    fn cache_schema_version_default_is_zero() {
        // Default (legacy/unknown) is 0 — load_cache treats it as stale.
        assert_eq!(CatalogCache::default().schema_version, 0);
    }

    #[test]
    fn catalog_cache_deserializes_legacy_missing_schema_version() {
        // A cache written before schema_version existed has no such field.
        // serde default fills 0 → caller rebuilds. Must not error.
        let legacy = r#"{"built_at":100,"entries":[]}"#;
        let cache: CatalogCache = serde_json::from_str(legacy).unwrap();
        assert_eq!(cache.schema_version, 0);
        assert_eq!(cache.built_at, 100);
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn bundle_url_resolves_override_or_default() {
        // Non-empty override wins.
        assert_eq!(
            bundle_url_from(Some("http://localhost:9999/test.gz".into())),
            "http://localhost:9999/test.gz"
        );
        // Whitespace-only falls back to default.
        assert_eq!(bundle_url_from(Some("   ".into())), REMOTE_BUNDLE_URL);
        // Absent env falls back to default.
        assert_eq!(bundle_url_from(None), REMOTE_BUNDLE_URL);
    }
}

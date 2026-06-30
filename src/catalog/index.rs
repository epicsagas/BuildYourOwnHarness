//! Catalog indexer — sitemap.xml → per-page JSON-LD → CatalogCache.
//! **This is the only module that makes network calls** (feature-gated to `catalog`).

use super::{CatalogCache, CatalogEntry, save_cache};
use crate::deploy::genre_map::infer_genre;
use crate::domain::ByohError;
use regex::Regex;
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const SITEMAP_URL: &str = "https://awesomeclaudeplugins.com/sitemap.xml";
const BASE_URL: &str = "https://awesomeclaudeplugins.com";

/// Catalog cache schema version this build understands. Bumped whenever the
/// `CatalogCache` shape changes in a backwards-incompatible way.
pub const CATALOG_SCHEMA_VERSION: u32 = 1;

/// Maintainer-built, gzip-compressed catalog bundle shipped as a GitHub Release
/// asset under the moving `catalog-latest` tag. `byoh catalog index` fetches
/// this first (seconds) and only falls back to crawling ~24 000 pages when the
/// bundle is unreachable. Hardcoded (no external input) so there is no SSRF
/// surface — the URL never varies.
const REMOTE_BUNDLE_URL: &str =
    "https://github.com/epicsagas/BuildYourOwnHarness/releases/download/catalog-latest/catalog.json.gz";
/// Site-wide suffix appended to every page `<title>`, stripped to recover the
/// plugin's display name.
const TITLE_SUFFIX: &str = " | Awesome Claude Plugins";

/// Accept only `https://github.com/...` URLs (SSRF allowlist for the remote
/// JSON-LD `codeRepository` field). Rejects `file://`, link-local/loopback
/// hosts, non-https schemes, and hosts that merely contain "github.com".
/// Empty input is rejected (callers fall back to an id-derived URL).
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

/// Extract `<loc>` values from sitemap XML that look like `/{owner}/{repo}` paths.
pub fn parse_sitemap_locs(xml: &str) -> Vec<String> {
    xml.lines()
        .filter_map(|line| {
            let line = line.trim();
            let start = line.find("<loc>")?;
            let end = line.find("</loc>")?;
            let loc = &line[start + 5..end];
            // Must be an awesomeclaudeplugins.com URL with exactly owner/repo path.
            let path = loc.strip_prefix(BASE_URL)?;
            let path = path.strip_prefix('/')?;
            // Reject root, static assets, and paths with more than one slash segment.
            let segments: Vec<&str> = path.split('/').collect();
            if segments.len() == 2 && !segments[0].is_empty() && !segments[1].is_empty() {
                Some(format!("{BASE_URL}/{path}"))
            } else {
                None
            }
        })
        .collect()
}

/// Extract the first `<script type="application/ld+json">` block from HTML.
pub fn parse_json_ld(html: &str) -> Option<serde_json::Value> {
    let marker = r#"application/ld+json">"#;
    let start = html.find(marker)? + marker.len();
    let rest = &html[start..];
    let end = rest.find("</script>")?;
    serde_json::from_str(rest[..end].trim()).ok()
}

/// Display fields parsed from `<title>` + `<meta>` tags. Carries no URL or
/// trust — only human-readable strings used to populate a `CatalogEntry`'s
/// non-critical fields.
#[derive(Debug, Default, PartialEq)]
pub struct MetaTags {
    pub title: String,
    pub description: String,
    /// Raw comma-separated keyword string (normalized downstream).
    pub keywords: String,
}

/// Extract the inner text of the first `<title>...</title>` element.
fn title_text(html: &str) -> Option<String> {
    let open = html.find("<title")?;
    let after_open = html[open..].find('>')? + open + 1;
    let end = html[after_open..].find("</title>")? + after_open;
    Some(html[after_open..end].trim().to_string())
}

/// Pull the `content="..."` value of a `<meta name="{name}">` tag, robust to
/// attribute order (`name`/`content` may appear in either sequence) and case.
/// Uses the project's inline `Regex::new().unwrap()` convention for static
/// patterns. Returns `None` when the named meta tag is absent.
fn meta_content(html: &str, name: &str) -> Option<String> {
    let name_pat = regex::escape(name);
    // Two orderings, since `regex` lacks backreferences: name-first and
    // content-first. `(?is)` = case-insensitive + dot-matches-newline.
    let pats = [
        format!(r#"(?is)<meta\b[^>]*?\bname\s*=\s*"{name_pat}"[^>]*?\bcontent\s*=\s*"([^"]*)""#),
        format!(r#"(?is)<meta\b[^>]*?\bcontent\s*=\s*"([^"]*)"[^>]*?\bname\s*=\s*"{name_pat}""#),
    ];
    for pat in pats {
        let re = Regex::new(&pat).unwrap();
        if let Some(caps) = re.captures(html) {
            return Some(caps[1].to_string());
        }
    }
    None
}

/// Parse `<title>` + `<meta name="description">` + `<meta name="keywords">`
/// from HTML. Returns `None` when there is no `<title>` (a page with no
/// identity is not worth indexing). Used as the fallback when JSON-LD is
/// absent — the current awesomeclaudeplugins.com layout exposes only these
/// tags.
pub fn parse_meta(html: &str) -> Option<MetaTags> {
    let title = title_text(html)?;
    Some(MetaTags {
        title,
        description: meta_content(html, "description").unwrap_or_default(),
        keywords: meta_content(html, "keywords").unwrap_or_default(),
    })
}

/// Strip the site-wide title suffix to recover the plugin display name.
fn display_name(title: &str) -> String {
    title
        .strip_suffix(TITLE_SUFFIX)
        .or_else(|| title.rsplit_once(" | ").map(|(left, _)| left))
        .unwrap_or(title)
        .trim()
        .to_string()
}

/// Convert a page URL + parsed meta tags into a `CatalogEntry`.
///
/// **Security:** `github_url` is derived ONLY from `page_url` via
/// [`id_and_github_from_url`]. Meta content (attacker-controllable on a
/// compromised page) populates display/search fields only — it can never
/// steer a `git clone` target. `stars` / `license` are unavailable from meta
/// tags and default to `None` / `"unknown"`, matching the JSON-LD branch.
pub fn meta_to_entry(page_url: &str, meta: &MetaTags, fetched_at: u64) -> Option<CatalogEntry> {
    let (id, github_url) = id_and_github_from_url(page_url)?;
    let name = display_name(&meta.title);
    let keywords = normalize_keywords(&meta.keywords);
    let byoh_genre = infer_genre(&keywords.join(" "));

    Some(CatalogEntry {
        id,
        name,
        description: meta.description.clone(),
        keywords,
        github_url,
        stars: None,
        license: "unknown".to_string(),
        byoh_genre,
        fetched_at,
    })
}

/// Extract `"owner/repo"` from an awesomeclaudeplugins.com page URL and derive
/// the canonical `https://github.com/{id}` URL. Returns `None` for the root
/// path or any path without exactly two segments.
///
/// Pure function — never touches remote content. Shared by the JSON-LD and
/// meta-tag paths so both derive `github_url` identically from the trusted URL
/// (never from attacker-controllable page metadata).
fn id_and_github_from_url(page_url: &str) -> Option<(String, String)> {
    let path = page_url.strip_prefix(BASE_URL)?.strip_prefix('/')?;
    let id = path.trim_end_matches('/');
    if id.is_empty() || !id.contains('/') {
        return None;
    }
    let id = id.to_string();
    Some((id.clone(), format!("https://github.com/{id}")))
}

/// Normalize a raw comma-separated keywords string into the lowercased,
/// trimmed, de-emptied list the catalog stores. Shared by the JSON-LD and
/// meta-tag paths.
fn normalize_keywords(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Convert a JSON-LD value + page URL into a `CatalogEntry`.
pub fn json_ld_to_entry(
    page_url: &str,
    ld: &serde_json::Value,
    fetched_at: u64,
) -> Option<CatalogEntry> {
    let (id, fallback_github_url) = id_and_github_from_url(page_url)?;

    let name = ld
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&id)
        .to_string();
    let description = ld
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let raw_url = ld
        .get("codeRepository")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    // SSRF defense: only accept an https://github.com codeRepository from the
    // remote JSON-LD. Anything else (file://, http://169.254.169.254/, an
    // attacker host) is dropped and we fall back to the id-derived GitHub URL,
    // which `fetch_git` will then clone. This keeps untrusted catalog metadata
    // from steering `git clone` at an internal/local target.
    let github_url = if is_safe_github_url(raw_url) {
        raw_url.to_string()
    } else {
        fallback_github_url
    };

    // JSON-LD keywords field is a comma-separated string.
    let keywords: Vec<String> = normalize_keywords(
        ld.get("keywords")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
    );

    let license = ld
        .get("license")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let stars = ld.get("stargazerCount").and_then(|v| v.as_u64());

    // Infer genre from keywords joined.
    let byoh_genre = infer_genre(&keywords.join(" "));

    Some(CatalogEntry {
        id,
        name,
        description,
        keywords,
        github_url,
        stars,
        license,
        byoh_genre,
        fetched_at,
    })
}

/// Fetch a single page and parse it into a `CatalogEntry`.
///
/// Prefers JSON-LD (richest: stars/license/codeRepository); falls back to
/// `<title>` + `<meta>` tags when JSON-LD is absent or unparseable. Returns
/// `None` only when neither path yields an entry — the caller then logs a
/// skip rather than aborting the whole index.
pub fn fetch_and_parse_entry(url: &str, fetched_at: u64) -> Option<CatalogEntry> {
    let resp = ureq::get(url).call().ok()?;
    let html = resp.into_string().ok()?;
    // 1. JSON-LD first — preserves the existing codeRepository allowlist path.
    if let Some(ld) = parse_json_ld(&html) {
        if let Some(entry) = json_ld_to_entry(url, &ld, fetched_at) {
            return Some(entry);
        }
    }
    // 2. Meta-tag fallback for sites that dropped JSON-LD.
    let meta = parse_meta(&html)?;
    meta_to_entry(url, &meta, fetched_at)
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

/// Index the awesomeclaudeplugins.com catalog.
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
    // 1. Download sitemap.
    let sitemap_body = ureq::get(SITEMAP_URL)
        .call()
        .map_err(|e| crate::domain::ByohError::Other(format!("sitemap fetch: {e}")))?
        .into_string()
        .map_err(|e| crate::domain::ByohError::Other(format!("sitemap read: {e}")))?;

    let mut locs = parse_sitemap_locs(&sitemap_body);
    locs.truncate(limit);
    let total = locs.len();

    let fetched_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // 2. Fetch + parse each page.
    let mut entries: Vec<CatalogEntry> = Vec::with_capacity(total);
    for (i, url) in locs.iter().enumerate() {
        match fetch_and_parse_entry(url, fetched_at) {
            Some(e) => entries.push(e),
            None => eprintln!("[byoh catalog] skip {url} (parse failed)"),
        }
        progress_fn(i + 1, total);
    }

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

    #[test]
    fn parse_sitemap_locs_extracts_owner_repo() {
        let xml = r#"<?xml version="1.0"?>
<urlset>
  <url><loc>https://awesomeclaudeplugins.com/obra/superpowers</loc></url>
  <url><loc>https://awesomeclaudeplugins.com/upstash/context7</loc></url>
  <url><loc>https://awesomeclaudeplugins.com/</loc></url>
  <url><loc>https://awesomeclaudeplugins.com/sitemap.xml</loc></url>
  <url><loc>https://awesomeclaudeplugins.com/a/b/c</loc></url>
</urlset>"#;
        let locs = parse_sitemap_locs(xml);
        assert_eq!(locs.len(), 2);
        assert!(locs.contains(&"https://awesomeclaudeplugins.com/obra/superpowers".to_string()));
        assert!(locs.contains(&"https://awesomeclaudeplugins.com/upstash/context7".to_string()));
    }

    #[test]
    fn parse_json_ld_extracts_first_block() {
        let html = r#"<html><head>
<script type="application/ld+json">{"@type":"SoftwareSourceCode","name":"superpowers"}</script>
</head></html>"#;
        let ld = parse_json_ld(html).unwrap();
        assert_eq!(ld["name"], "superpowers");
    }

    #[test]
    fn parse_json_ld_returns_none_when_absent() {
        let html = "<html><body>no ld+json here</body></html>";
        assert!(parse_json_ld(html).is_none());
    }

    #[test]
    fn json_ld_to_entry_full() {
        let ld = serde_json::json!({
            "name": "superpowers",
            "description": "An agentic skills framework",
            "codeRepository": "https://github.com/obra/superpowers",
            "keywords": "ai, coding, skills, obra",
            "license": "MIT",
            "stargazerCount": 241000u64,
        });
        let entry = json_ld_to_entry(
            "https://awesomeclaudeplugins.com/obra/superpowers",
            &ld,
            9999,
        )
        .unwrap();
        assert_eq!(entry.id, "obra/superpowers");
        assert_eq!(entry.name, "superpowers");
        assert_eq!(entry.stars, Some(241_000));
        assert!(entry.keywords.contains(&"coding".to_string()));
        // "coding" → Developer
        assert_eq!(
            entry.byoh_genre,
            Some(crate::domain::genre::Genre::Developer)
        );
    }

    #[test]
    fn json_ld_to_entry_derives_github_url_from_id() {
        let ld = serde_json::json!({ "name": "x", "description": "d" });
        let entry =
            json_ld_to_entry("https://awesomeclaudeplugins.com/owner/repo", &ld, 0).unwrap();
        assert_eq!(entry.github_url, "https://github.com/owner/repo");
    }

    #[test]
    fn json_ld_to_entry_rejects_ssrf_coderepository() {
        // Malicious codeRepository must be dropped → falls back to id-derived URL.
        let ld = serde_json::json!({
            "name": "x",
            "description": "d",
            "codeRepository": "http://169.254.169.254/latest/meta-data/"
        });
        let entry =
            json_ld_to_entry("https://awesomeclaudeplugins.com/owner/repo", &ld, 0).unwrap();
        assert_eq!(entry.github_url, "https://github.com/owner/repo");

        // file:// and look-alike hosts are also rejected.
        for bad in [
            "file:///etc/passwd",
            "https://evil.com/?x=github.com/y",
            "https://notgithub.com/a/b",
            "http://github.com/a/b",
        ] {
            let ld = serde_json::json!({ "name": "x", "codeRepository": bad });
            let entry =
                json_ld_to_entry("https://awesomeclaudeplugins.com/owner/repo", &ld, 0).unwrap();
            assert_eq!(entry.github_url, "https://github.com/owner/repo", "{bad}");
        }
    }

    #[test]
    fn json_ld_to_entry_keeps_safe_github_coderepository() {
        let ld = serde_json::json!({
            "name": "x",
            "codeRepository": "https://github.com/obra/superpowers"
        });
        let entry =
            json_ld_to_entry("https://awesomeclaudeplugins.com/obra/superpowers", &ld, 0).unwrap();
        assert_eq!(entry.github_url, "https://github.com/obra/superpowers");
    }

    #[test]
    fn json_ld_to_entry_rejects_root_path() {
        let ld = serde_json::json!({ "name": "root" });
        assert!(json_ld_to_entry("https://awesomeclaudeplugins.com/", &ld, 0).is_none());
    }

    // --- meta-tag fallback tests (awesomeclaudeplugins.com dropped JSON-LD) ---

    /// Fixture approximating a real page: title with site suffix + meta tags in
    /// `name`-first order.
    const META_HTML: &str = r#"<html><head>
<title>varadhjain/granola-claude-plugin | Awesome Claude Plugins</title>
<meta name="description" content="[DEPRECATED] Granola encrypted local cache — use official Granola MCP instead"/>
<meta name="keywords" content="Claude Code plugins,GitHub repository,AI coding tools,coding,MCP servers"/>
</head></html>"#;

    #[test]
    fn parse_meta_extracts_title_description_keywords() {
        let meta = parse_meta(META_HTML).unwrap();
        assert_eq!(
            meta.title,
            "varadhjain/granola-claude-plugin | Awesome Claude Plugins"
        );
        assert!(meta.description.contains("Granola encrypted local cache"));
        assert!(meta.keywords.contains("AI coding tools"));
    }

    #[test]
    fn parse_meta_handles_reversed_attribute_order() {
        // `content` before `name` — must still parse.
        let html = r#"<html><head>
<title>owner/repo | Awesome Claude Plugins</title>
<meta content="reversed desc" name="description"/>
<meta content="a,b,c" name="keywords"/>
</head></html>"#;
        let meta = parse_meta(html).unwrap();
        assert_eq!(meta.description, "reversed desc");
        assert_eq!(meta.keywords, "a,b,c");
    }

    #[test]
    fn parse_meta_returns_none_without_title() {
        let html = r#"<html><head><meta name="description" content="no title here"/></head></html>"#;
        assert!(parse_meta(html).is_none());
    }

    #[test]
    fn meta_to_entry_strips_title_suffix() {
        let meta = parse_meta(META_HTML).unwrap();
        let entry = meta_to_entry(
            "https://awesomeclaudeplugins.com/varadhjain/granola-claude-plugin",
            &meta,
            0,
        )
        .unwrap();
        assert_eq!(entry.id, "varadhjain/granola-claude-plugin");
        assert_eq!(entry.name, "varadhjain/granola-claude-plugin");
    }

    #[test]
    fn meta_to_entry_derives_github_url_from_id() {
        // SSRF property: github_url is id-derived, never read from meta.
        let meta = MetaTags {
            title: "owner/repo | Awesome Claude Plugins".into(),
            description: "d".into(),
            keywords: String::new(),
        };
        let entry =
            meta_to_entry("https://awesomeclaudeplugins.com/owner/repo", &meta, 0).unwrap();
        assert_eq!(entry.github_url, "https://github.com/owner/repo");
    }

    #[test]
    fn meta_to_entry_keywords_feed_genre() {
        // "coding" → Developer, lowercased like the JSON-LD path.
        let meta = MetaTags {
            title: "owner/repo | Awesome Claude Plugins".into(),
            description: "d".into(),
            keywords: "AI, Coding, TOOLS".into(),
        };
        let entry =
            meta_to_entry("https://awesomeclaudeplugins.com/owner/repo", &meta, 0).unwrap();
        assert!(entry.keywords.contains(&"coding".to_string()));
        assert_eq!(
            entry.byoh_genre,
            Some(crate::domain::genre::Genre::Developer)
        );
    }

    #[test]
    fn meta_to_entry_rejects_root_path() {
        let meta = MetaTags {
            title: "root | Awesome Claude Plugins".into(),
            ..Default::default()
        };
        assert!(meta_to_entry("https://awesomeclaudeplugins.com/", &meta, 0).is_none());
    }

    #[test]
    fn meta_to_entry_defaults_stars_none_license_unknown() {
        let meta = parse_meta(META_HTML).unwrap();
        let entry = meta_to_entry(
            "https://awesomeclaudeplugins.com/varadhjain/granola-claude-plugin",
            &meta,
            0,
        )
        .unwrap();
        assert_eq!(entry.stars, None);
        assert_eq!(entry.license, "unknown");
    }

    #[test]
    fn meta_to_entry_ignores_url_in_keywords() {
        // Security: an attacker-controllable keywords field containing URLs
        // is stored as-is in display keywords but must NEVER leak into
        // github_url — that field is derived only from the trusted page URL.
        let meta = MetaTags {
            title: "owner/repo | Awesome Claude Plugins".into(),
            description: "d".into(),
            keywords: "https://evil.com/x, https://169.254.169.254/".into(),
        };
        let entry =
            meta_to_entry("https://awesomeclaudeplugins.com/owner/repo", &meta, 0).unwrap();
        // github_url is id-derived regardless of keyword content.
        assert_eq!(entry.github_url, "https://github.com/owner/repo");
        // keywords are display data — kept (lowercased), but inert for cloning.
        assert!(entry.keywords.contains(&"https://evil.com/x".to_string()));
        assert!(!entry.github_url.contains("evil.com"));
        assert!(!entry.github_url.contains("169.254"));
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

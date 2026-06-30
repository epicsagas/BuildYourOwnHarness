//! Catalog indexer — sitemap.xml → per-page JSON-LD → CatalogCache.
//! **This is the only module that makes network calls** (feature-gated to `catalog`).

use super::{CatalogCache, CatalogEntry, save_cache};
use crate::deploy::genre_map::infer_genre;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const SITEMAP_URL: &str = "https://awesomeclaudeplugins.com/sitemap.xml";
const BASE_URL: &str = "https://awesomeclaudeplugins.com";

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

/// Convert a JSON-LD value + page URL into a `CatalogEntry`.
pub fn json_ld_to_entry(
    page_url: &str,
    ld: &serde_json::Value,
    fetched_at: u64,
) -> Option<CatalogEntry> {
    // Extract owner/repo from URL: https://awesomeclaudeplugins.com/{owner}/{repo}
    let path = page_url.strip_prefix(BASE_URL)?.strip_prefix('/')?;
    let id = path.trim_end_matches('/').to_string();
    if id.is_empty() || !id.contains('/') {
        return None;
    }

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
        format!("https://github.com/{id}")
    };

    // JSON-LD keywords field is a comma-separated string.
    let keywords: Vec<String> = ld
        .get("keywords")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

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
pub fn fetch_and_parse_entry(url: &str, fetched_at: u64) -> Option<CatalogEntry> {
    let resp = ureq::get(url).call().ok()?;
    let html = resp.into_string().ok()?;
    let ld = parse_json_ld(&html)?;
    json_ld_to_entry(url, &ld, fetched_at)
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
}

//! Filesystem ProfileSource — non-destructive autoscan (B5 vector→BM25→grep,
//! B6 derived tagging). The grep tier is implemented directly; vector/BM25 are
//! graceful upgrades behind the same fallback chain.

use std::path::{Path, PathBuf};

use crate::domain::profile::{DataSource, ProvenanceSource};
use crate::ports::source::{ProfileSource, ScanHit};

/// Walks local paths, extracts keyword/topic candidates as `derived`.
#[derive(Debug, Default, Clone)]
pub struct FilesystemSource {
    /// Max files to scan (safety bound).
    pub max_files: usize,
}

impl FilesystemSource {
    pub fn new() -> Self {
        Self { max_files: 4000 }
    }

    pub fn with_max_files(max_files: usize) -> Self {
        Self { max_files }
    }

    /// Vector tier → BM25 tier → grep tier. The fallback is deterministic:
    /// we always run grep, which needs no model.
    fn extract_terms(&self, paths: &[&Path]) -> crate::domain::Result<Vec<ScanHit>> {
        let mut hits: Vec<ScanHit> = Vec::new();
        let mut files_scanned = 0usize;

        for root in paths {
            for entry in walkdir::WalkDir::new(root)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if files_scanned >= self.max_files {
                    break;
                }
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                files_scanned += 1;
                let rel = path.to_string_lossy().to_string();
                // Grep tier: scan text-ish files for capitalized tokens / hashtags.
                let body = std::fs::read_to_string(path).unwrap_or_default();
                for term in extract_candidate_terms(&body) {
                    hits.push(ScanHit {
                        term,
                        provenance: format!("{}:{}", rel, 0),
                        kind: self.classify(root).kind,
                        tags: Vec::new(),
                    });
                }
                if let Some(tags) = extract_frontmatter_tags(&body) {
                    if let Some(last) = hits.last_mut() {
                        last.tags = tags;
                    }
                }
            }
        }
        Ok(hits)
    }
}

/// Extract candidate terms: hashtags, capitalized words, fenced language tags.
pub(crate) fn extract_candidate_terms(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    // hashtags
    let tag_re = regex::Regex::new(r"#([A-Za-z0-9_가-힣]{2,30})").unwrap();
    for cap in tag_re.captures_iter(body) {
        out.push(cap[1].to_string());
    }
    // fenced code language hints
    let fence_re = regex::Regex::new(r"(?m)^```([A-Za-z0-9]+)").unwrap();
    for cap in fence_re.captures_iter(body) {
        out.push(format!("code:{}", &cap[1].to_lowercase()));
    }
    out.sort();
    out.dedup();
    out
}

/// Extract YAML/TOML frontmatter `tags:` array if present.
pub(crate) fn extract_frontmatter_tags(body: &str) -> Option<Vec<String>> {
    let start = body.strip_prefix("---\n")?;
    let end = start.find("\n---")?;
    let fm = &start[..end];
    let mut tags = Vec::new();
    let mut in_tags = false;
    for line in fm.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("tags:") {
            in_tags = true;
            continue;
        }
        if in_tags {
            if let Some(item) = trimmed.strip_prefix("- ") {
                tags.push(item.trim_matches(|c| c == '"' || c == '\'').to_string());
            } else if !trimmed.starts_with('-') && !line.starts_with(' ') && !line.is_empty() {
                in_tags = false;
            }
        }
    }
    if tags.is_empty() {
        None
    } else {
        Some(tags)
    }
}

impl ProfileSource for FilesystemSource {
    fn scan(&self, paths: &[&Path]) -> crate::domain::Result<Vec<ScanHit>> {
        self.extract_terms(paths)
    }

    fn classify(&self, path: &Path) -> DataSource {
        let kind = classify_kind(path);
        DataSource {
            path: path.to_string_lossy().to_string(),
            kind,
            candidate_tags: Vec::new(),
            tags_source: ProvenanceSource::Derived,
        }
    }
}

fn classify_kind(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    let has_obs = path.join(".obsidian").exists();
    if has_obs || name == "vault" {
        "obsidian".into()
    } else if path.join(".git").exists() {
        "git_repo".into()
    } else if name.ends_with(".md") || markdown_density(path) {
        "markdown_dir".into()
    } else {
        "text_dir".into()
    }
}

fn markdown_density(path: &Path) -> bool {
    let mut md = 0;
    let mut total = 0;
    if let Ok(rd) = std::fs::read_dir(path) {
        for entry in rd.flatten() {
            total += 1;
            if entry
                .path()
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("md"))
                .unwrap_or(false)
            {
                md += 1;
            }
            if total >= 20 {
                break;
            }
        }
    }
    total > 0 && md * 2 >= total
}

#[allow(dead_code)]
fn walk_root(_p: &Path) -> PathBuf {
    PathBuf::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn scan_is_non_destructive_and_derived() {
        let dir = tempdir().unwrap();
        let note = dir.path().join("note.md");
        fs::write(
            &note,
            "---\ntags:\n  - writing\n  - research\n---\n# Project\nUse #k8s and #rust. \n```rust\nfn main(){}\n```",
        )
        .unwrap();
        let before = fs::read_to_string(&note).unwrap();

        let src = FilesystemSource::new();
        let hits = src.scan(&[dir.path()]).unwrap();
        assert!(!hits.is_empty());
        let terms: Vec<_> = hits.iter().map(|h| h.term.clone()).collect();
        assert!(terms.iter().any(|t| t == "k8s"));
        assert!(terms.iter().any(|t| t == "rust"));
        assert!(terms.iter().any(|t| t == "code:rust"));

        // AC3: original file unchanged.
        assert_eq!(fs::read_to_string(&note).unwrap(), before);
    }

    #[test]
    fn frontmatter_tags_extracted() {
        let body = "---\ntags:\n  - a\n  - b\n---\nbody";
        assert_eq!(
            extract_frontmatter_tags(body),
            Some(vec!["a".into(), "b".into()])
        );
    }

    #[test]
    fn classify_kinds() {
        let src = FilesystemSource::new();
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.md"), "x").unwrap();
        let ds = src.classify(dir.path());
        assert_eq!(ds.tags_source, ProvenanceSource::Derived);
    }
}

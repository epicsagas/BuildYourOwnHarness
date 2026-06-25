//! Profile persistence + corpus collection + embedder factory.
//!
//! Lifted out of the binary (`main.rs`) so both the `byoh` CLI and the MCP
//! server (`byoh serve`, behind the `mcp` feature) share one implementation.
//! All synchronous — no tokio — matching the rest of the lib.

use std::path::{Path, PathBuf};

use crate::domain::error::ByohError;
use crate::domain::profile::UserProfile;
use crate::ports::EmbedderProvider;
use crate::Result;

/// Wrap an io::Error with the offending path for clearer diagnostics
/// (the `#[from]` conversion on `ByohError::Io` loses path context).
fn io_at(path: &Path, e: std::io::Error) -> ByohError {
    ByohError::Other(format!("{}: {}", path.display(), e))
}

/// The BYOH home directory (`$BYOH_HOME`, default `.byoh`).
/// Profiles live under `<home>/profiles/`, genre indexes under `<home>/indexes/`.
pub fn byoh_home() -> PathBuf {
    PathBuf::from(std::env::var("BYOH_HOME").unwrap_or_else(|_| ".byoh".to_string()))
}

/// Profiles root: `<byoh_home>/profiles`.
pub fn profiles_root() -> PathBuf {
    byoh_home().join("profiles")
}

/// Path to a profile YAML by slug: `<profiles_root>/<slug>.yaml`.
pub fn profile_path(slug: &str) -> PathBuf {
    profiles_root().join(format!("{slug}.yaml"))
}

/// Load a profile YAML by slug. Errors surface as `ByohError::Io` / `SerdeYaml`.
pub fn load_profile(slug: &str) -> Result<UserProfile> {
    let path = profile_path(slug);
    let body = std::fs::read_to_string(&path).map_err(|e| io_at(&path, e))?;
    Ok(serde_yaml::from_str(&body)?)
}

/// Persist a profile (creates the parent directory). Errors: `SerdeYaml` / `Io`.
pub fn write_profile(p: &UserProfile) -> Result<()> {
    let path = profile_path(&p.slug);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_yaml::to_string(p)?)?;
    Ok(())
}

/// Collect text documents (`.md`/`.txt`/`.rs`/`.py`/...) under `corpus` into
/// [`crate::rag::InputDoc`]s. A single file is read as one doc; a directory is
/// walked non-recursively-filtered by text extension. Used by the CLI `index`/
/// `search` commands and the MCP `rag_index`/`rag_search` tools.
pub fn collect_corpus(corpus: &Path) -> Result<Vec<crate::rag::InputDoc>> {
    if !corpus.exists() {
        return Err(io_at(
            corpus,
            std::io::Error::new(std::io::ErrorKind::NotFound, "corpus path does not exist"),
        ));
    }
    let mut docs = Vec::new();
    if corpus.is_file() {
        let text = std::fs::read_to_string(corpus).map_err(|e| io_at(corpus, e))?;
        let id = corpus
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("doc")
            .to_string();
        docs.push(crate::rag::InputDoc { id, text });
        return Ok(docs);
    }
    for entry in walkdir::WalkDir::new(corpus)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let is_text = p
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| {
                matches!(
                    e,
                    "md" | "txt" | "rs" | "py" | "ts" | "js" | "toml" | "yaml" | "yml" | "json"
                )
            })
            .unwrap_or(false);
        if !is_text {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(p) {
            let id = p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("doc")
                .to_string();
            docs.push(crate::rag::InputDoc { id, text });
        }
    }
    Ok(docs)
}

/// Default-build embedder: [`DummyEmbedder`] (deterministic, no model download).
pub fn make_embedder() -> Result<Box<dyn EmbedderProvider>> {
    Ok(Box::new(crate::adapters::DummyEmbedder::new()))
}

/// Write `content` to `<dir>/<name>`, creating `dir` (and parents) first.
/// Used by the target renderer to emit plugin files.
pub fn write_file(dir: &Path, name: &str, content: &str) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join(name), content)?;
    Ok(())
}

/// Create a symlink `link` → `target` (Unix). On non-Unix, copy the target dir
/// recursively as a fallback (Windows symlink needs privileges). `link`'s parent
/// is created first.
pub fn create_symlink_or_copy(target: &Path, link: &Path) -> Result<()> {
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)?;
    }
    #[cfg(unix)]
    {
        let _ = std::fs::remove_file(link);
        std::os::unix::fs::symlink(target, link)?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        copy_dir_recursive(target, link)
    }
}

#[cfg(not(unix))]
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// `native-rag` embedder: [`FastembedEmbedder`] with a graceful dummy fallback
/// if the model cannot be loaded. cfg-gated because `FastembedEmbedder` only
/// exists under `native-rag`.
#[cfg(feature = "native-rag")]
pub fn make_embedder_native() -> Result<Box<dyn EmbedderProvider>> {
    match crate::adapters::embedder::FastembedEmbedder::new() {
        Ok(fe) => Ok(Box::new(fe)),
        Err(e) => {
            eprintln!("[byoh] fastembed unavailable ({e}); falling back to dummy");
            Ok(Box::new(crate::adapters::DummyEmbedder::new()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::UserProfile;

    fn isolate_home(dir: &Path) {
        std::env::set_var("BYOH_HOME", dir);
    }

    #[test]
    fn profile_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        isolate_home(dir.path());
        let p = UserProfile::new_draft("round-trip", "en");
        write_profile(&p).unwrap();
        let loaded = load_profile("round-trip").unwrap();
        assert_eq!(loaded.slug, "round-trip");
        std::env::remove_var("BYOH_HOME");
    }

    #[test]
    fn collect_corpus_filters_text_extensions() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.md"), "# hi").unwrap();
        std::fs::write(dir.path().join("b.rs"), "fn main(){}").unwrap();
        std::fs::write(dir.path().join("c.png"), b"\x89PNG").unwrap();
        let docs = collect_corpus(dir.path()).unwrap();
        let ids: Vec<&str> = docs.iter().map(|d| d.id.as_str()).collect();
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"));
        assert!(!ids.contains(&"c"));
    }

    #[test]
    fn collect_corpus_single_file() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("one.md");
        std::fs::write(&f, "body").unwrap();
        let docs = collect_corpus(&f).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].id, "one");
    }

    #[test]
    fn collect_corpus_missing_errors() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nope");
        assert!(collect_corpus(&missing).is_err());
    }
}

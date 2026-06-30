//! Profile persistence.
//!
//! Lifted out of the binary (`main.rs`) so both the `byoh` CLI and the MCP
//! server (`byoh serve`, behind the `mcp` feature) share one implementation.
//! All synchronous — no tokio — matching the rest of the lib.

use std::path::{Path, PathBuf};

use crate::Result;
use crate::domain::error::ByohError;
use crate::domain::profile::UserProfile;

/// Wrap an io::Error with the offending path for clearer diagnostics
/// (the `#[from]` conversion on `ByohError::Io` loses path context).
fn io_at(path: &Path, e: std::io::Error) -> ByohError {
    ByohError::Other(format!("{}: {}", path.display(), e))
}

/// Validate a slug before it is used in any filesystem path.
///
/// A slug must match `^[a-z0-9][a-z0-9-]*$` (lowercase alphanumerics + dashes,
/// not leading with a dash). This blocks path traversal (`..`), separators
/// (`/`, `\`), absolute paths, empty strings, and shell-hostile characters
/// before the slug is ever joined into a path. Shared by install / evolve / run.
pub fn sanitize_slug(slug: &str) -> Result<&str> {
    const MAX: usize = 64;
    if slug.is_empty() {
        return Err(ByohError::Schema("slug must not be empty".into()));
    }
    if slug.len() > MAX {
        return Err(ByohError::Schema(format!(
            "slug too long (>{MAX} chars): '{slug}'"
        )));
    }
    let mut chars = slug.chars();
    let first = chars.next().unwrap();
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return Err(ByohError::Schema(format!(
            "slug must start with [a-z0-9]: '{slug}'"
        )));
    }
    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(ByohError::Schema(format!(
            "slug may only contain [a-z0-9-]: '{slug}'"
        )));
    }
    Ok(slug)
}

/// The BYOH home directory (`$BYOH_HOME`, default `.byoh`).
/// Profiles live under `<home>/profiles/`.
///
/// Resolution order: thread-local test override (`set_home_override`) →
/// `$BYOH_HOME` → `.byoh`. The override exists so tests can isolate the home
/// directory **without mutating process-global env vars**, which became
/// `unsafe` in the Rust 2024 edition (`std::env::set_var`) and are incompatible
/// with this crate's `#![forbid(unsafe_code)]`.
pub fn byoh_home() -> PathBuf {
    if let Some(p) = HOME_OVERRIDE.with(|c| c.borrow().clone()) {
        return p;
    }
    PathBuf::from(std::env::var("BYOH_HOME").unwrap_or_else(|_| ".byoh".to_string()))
}

thread_local! {
    static HOME_OVERRIDE: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// Override `byoh_home()` for the current thread (tests). `None` clears it.
/// Use this instead of `set_var("BYOH_HOME", …)`: it is `unsafe`-free, scoped
/// to the calling thread, and does not leak across tests.
pub fn set_home_override(path: Option<PathBuf>) {
    HOME_OVERRIDE.with(|c| *c.borrow_mut() = path);
}

/// Profiles root: `<byoh_home>/profiles`.
pub fn profiles_root() -> PathBuf {
    byoh_home().join("profiles")
}

/// Profiles root under an explicit `home` (for callers that already resolved
/// `byoh_home()` on the originating thread — e.g. MCP tools via `spawn_blocking`,
/// where the thread-local home override is invisible on the worker thread).
pub fn profiles_root_in(home: &Path) -> PathBuf {
    home.join("profiles")
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

/// Load a profile YAML by slug from an explicit `home` directory. Use this from
/// `spawn_blocking` worker threads where the thread-local home override is not
/// visible (see [`profiles_root_in`]).
pub fn load_profile_in(home: &Path, slug: &str) -> Result<UserProfile> {
    let path = profiles_root_in(home).join(format!("{slug}.yaml"));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::UserProfile;

    fn isolate_home(dir: &Path) {
        // Thread-local override — no process-global env mutation (Rust 2024 made
        // set_var unsafe; this crate is #![forbid(unsafe_code)]).
        set_home_override(Some(dir.to_path_buf()));
    }

    #[test]
    fn sanitize_slug_accepts_valid() {
        for s in ["dev", "my-harness", "a1", "x", "team-7-backend"] {
            assert!(sanitize_slug(s).is_ok(), "should accept '{s}'");
        }
    }

    #[test]
    fn sanitize_slug_rejects_attacks_and_malformed() {
        for s in [
            "",           // empty
            "../etc",     // traversal
            "..",         // traversal
            "a/b",        // separator
            "a\\b",       // backslash
            "/abs",       // absolute
            "-leading",   // leading dash
            "UPPER",      // uppercase
            "with space", // space
            "semi;colon", // shell-hostile
        ] {
            assert!(sanitize_slug(s).is_err(), "should reject '{s}'");
        }
        // too long
        assert!(sanitize_slug(&"a".repeat(65)).is_err());
    }

    #[test]
    fn profile_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        isolate_home(dir.path());
        let p = UserProfile::new_draft("round-trip", "en");
        write_profile(&p).unwrap();
        let loaded = load_profile("round-trip").unwrap();
        assert_eq!(loaded.slug, "round-trip");
        set_home_override(None);
    }
}

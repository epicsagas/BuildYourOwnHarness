//! Command port — invoke external execution-layer tools (obsidian-forge,
//! alcove, epic-harness) via `std::process::Command`. Missing tools produce a
//! graceful fallback, not a hard error (ARCH §3.1 M0 dry-run fallback).

use std::path::Path;

/// Result of invoking an external tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOutcome {
    /// Tool ran and exited 0.
    Ran { stdout: String },
    /// Tool is not installed — caller should fall back.
    NotInstalled,
    /// Tool ran but failed; message for diagnostics.
    Failed { code: i32, stderr: String },
}

pub trait CommandPort {
    /// Run a named tool with args in `cwd`. Never panics on missing binary.
    fn run(&self, tool: &str, args: &[&str], cwd: Option<&Path>) -> CommandOutcome;

    /// Is the tool present on PATH?
    fn is_installed(&self, tool: &str) -> bool;
}

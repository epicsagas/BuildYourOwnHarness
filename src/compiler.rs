//! Compiler — ConfirmedProfile + genre template → HarnessBundle (ARCH §5).
//!
//! Pipeline: load genre skeleton → render skills → render MCP tools →
//! render hooks → validate (static gate) → [dry-run gate].

pub mod dryrun;
pub mod render;
pub mod validate;

pub use dryrun::{DryRunReport, dry_run};
pub use render::{compile_profile, is_skeleton_body};
pub use validate::{StaticGateReport, static_gate};

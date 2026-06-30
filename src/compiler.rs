//! Compiler — ConfirmedProfile + genre template → HarnessBundle (ARCH §5).
//!
//! Pipeline: load genre skeleton → render skills → render MCP tools →
//! render hooks → validate (static gate) → [dry-run gate].

pub mod dryrun;
pub mod incremental;
pub mod render;
pub mod validate;

pub use dryrun::{DryRunReport, dry_run};
pub use incremental::{ChangeClass, classify_change, recompile};
pub use render::compile_profile;
pub use validate::{StaticGateReport, static_gate};

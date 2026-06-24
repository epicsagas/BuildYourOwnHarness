//! Compiler — ConfirmedProfile + genre template → HarnessBundle (ARCH §5).
//!
//! Pipeline: load genre skeleton → render skills → render MCP tools →
//! render hooks → validate (static gate) → [dry-run gate].

pub mod dryrun;
pub mod incremental;
pub mod render;
pub mod validate;

pub use dryrun::{dry_run, DryRunReport};
pub use incremental::{classify_change, recompile, ChangeClass};
pub use render::compile_profile;
pub use validate::{static_gate, StaticGateReport};

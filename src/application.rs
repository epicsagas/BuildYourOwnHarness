//! Application layer — orchestrators wiring ports.
pub mod profiler;
pub mod render_plugin;
pub mod synthesis;
pub use profiler::ProfileOrchestrator;
pub use render_plugin::render_target;
pub use synthesis::synthesize;

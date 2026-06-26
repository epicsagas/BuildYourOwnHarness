//! Application layer — orchestrators wiring ports.
pub mod evolve_run;
pub mod goal_pipelines;
pub mod profiler;
pub mod render_plugin;
pub mod synthesis;
pub use evolve_run::evolve_one_cycle;
pub use profiler::ProfileOrchestrator;
pub use render_plugin::render_target;
pub use synthesis::synthesize;

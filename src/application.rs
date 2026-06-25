//! Application layer — orchestrators wiring ports.
pub mod profiler;
pub mod synthesis;
pub use profiler::ProfileOrchestrator;
pub use synthesis::synthesize;

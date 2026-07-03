//! Concrete adapters behind the ports.

pub mod command;
pub mod interview;
pub mod llm;
pub mod source;

pub use command::StdCommand;
pub use interview::RuleInterview;
pub use llm::RuleLlm;
pub use source::FilesystemSource;

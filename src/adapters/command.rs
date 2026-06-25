//! StdCommand adapter — invokes external tools via std::process.

use std::path::Path;
use std::process::Command;

use crate::ports::command::{CommandOutcome, CommandPort};

/// Real subprocess-backed command port.
#[derive(Debug, Default, Clone)]
pub struct StdCommand;

impl StdCommand {
    pub fn new() -> Self {
        Self
    }
}

impl CommandPort for StdCommand {
    fn run(&self, tool: &str, args: &[&str], cwd: Option<&Path>) -> CommandOutcome {
        let mut cmd = Command::new(tool);
        cmd.args(args);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        match cmd.output() {
            Ok(out) => {
                if out.status.success() {
                    CommandOutcome::Ran {
                        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                    }
                } else {
                    CommandOutcome::Failed {
                        code: out.status.code().unwrap_or(-1),
                        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => CommandOutcome::NotInstalled,
            Err(_) => CommandOutcome::NotInstalled,
        }
    }

    fn is_installed(&self, tool: &str) -> bool {
        which(tool).is_some()
    }
}

fn which(tool: &str) -> Option<std::path::PathBuf> {
    // Minimal PATH lookup — avoids a which crates dependency.
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(tool);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_tool_is_not_installed() {
        let c = StdCommand::new();
        assert!(!c.is_installed("definitely-not-a-real-tool-xyz"));
    }

    #[test]
    fn run_missing_falls_back() {
        let c = StdCommand::new();
        match c.run("definitely-not-a-real-tool-xyz", &[], None) {
            CommandOutcome::NotInstalled => {}
            other => panic!("expected NotInstalled, got {other:?}"),
        }
    }
}

//! Dry-run gate (ARCH §3.1 M0 deliverable 4 / §5.4) — read-only verification
//! that a compiled bundle would render into a usable plugin: every check here
//! can actually FAIL (the previous "simulation" was tautologically true).
//! Execution-layer dependency probes degrade gracefully into fallbacks.

use crate::domain::bundle::{HarnessBundle, Ring};
use crate::ports::command::{CommandOutcome, CommandPort};

/// Result of a dry-run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRunReport {
    /// At least one Ring 1 skill exists — the harness has a pipeline entry.
    pub pipeline_ok: bool,
    /// Every skill has a filesystem-safe id, a non-empty body, and ids are
    /// unique (duplicates would collide at `skills/<id>/SKILL.md`).
    pub skills_ok: bool,
    /// Every agent has a filesystem-safe id and a non-empty body.
    pub agents_ok: bool,
    /// Every declared MCP tool is well-formed (name/description/schema.type).
    pub tools_list_ok: bool,
    /// Human-readable failure details (empty when everything passed).
    pub errors: Vec<String>,
    /// Graceful degradations (missing execution-layer deps) — not failures.
    pub fallbacks: Vec<String>,
}

impl DryRunReport {
    pub fn passed(&self) -> bool {
        self.pipeline_ok && self.skills_ok && self.agents_ok && self.tools_list_ok
    }
}

/// An id is filesystem-safe when it cannot escape or mangle the rendered
/// `skills/<id>/` / `agents/<id>.md` layout.
fn fs_safe_id(id: &str) -> bool {
    !id.is_empty() && !id.contains(['/', '\\']) && id != "." && id != ".." && !id.contains("..")
}

/// Run the dry-run verification. `commands` is the execution-layer port
/// (obsidian-forge/alcove/epic-harness). Missing tools degrade gracefully.
pub fn dry_run<C: CommandPort>(
    bundle: &HarnessBundle,
    commands: &C,
) -> crate::domain::Result<DryRunReport> {
    let mut report = DryRunReport {
        pipeline_ok: true,
        skills_ok: true,
        agents_ok: true,
        tools_list_ok: true,
        errors: Vec::new(),
        fallbacks: Vec::new(),
    };

    // (1) Pipeline entry: a harness with no Ring 1 skill has nothing to run.
    if !bundle.skills.iter().any(|s| s.ring == Ring::Ring1) {
        report.pipeline_ok = false;
        report
            .errors
            .push("no Ring 1 (pipeline) skill in the bundle".into());
    }

    // (2) Skills: fs-safe unique ids + non-empty bodies.
    let mut seen = std::collections::HashSet::new();
    for s in &bundle.skills {
        if !fs_safe_id(&s.id) {
            report.skills_ok = false;
            report
                .errors
                .push(format!("skill id '{}' is not filesystem-safe", s.id));
        }
        if s.body_markdown.trim().is_empty() {
            report.skills_ok = false;
            report
                .errors
                .push(format!("skill '{}' has an empty body", s.id));
        }
        if !seen.insert(s.id.clone()) {
            report.skills_ok = false;
            report
                .errors
                .push(format!("duplicate skill id '{}' (render collision)", s.id));
        }
    }

    // (3) Agents: fs-safe ids + non-empty bodies.
    for a in &bundle.agents {
        if !fs_safe_id(&a.id) {
            report.agents_ok = false;
            report
                .errors
                .push(format!("agent id '{}' is not filesystem-safe", a.id));
        }
        if a.body_markdown.trim().is_empty() {
            report.agents_ok = false;
            report
                .errors
                .push(format!("agent '{}' has an empty body", a.id));
        }
    }

    // (4) MCP tool declarations must be well-formed.
    for t in &bundle.mcp_tools {
        if !t.is_well_formed() {
            report.tools_list_ok = false;
            report
                .errors
                .push(format!("mcp tool '{}' is not well-formed", t.name));
        }
    }

    // (5) Execution-layer dependencies: probe, fall back gracefully if absent.
    for dep in &bundle.config.depends_on {
        match commands.run(&dep.id, &["--version"], None) {
            CommandOutcome::Ran { .. } => {}
            CommandOutcome::NotInstalled => {
                report.fallbacks.push(format!(
                    "dependency '{}' not installed — skipped (graceful fallback)",
                    dep.id
                ));
            }
            CommandOutcome::Failed { stderr, .. } => {
                report
                    .fallbacks
                    .push(format!("dependency '{}' failed: {stderr}", dep.id));
            }
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::compile_profile;
    use crate::domain::genre::Genre;
    use crate::domain::profile::{GenreConfidence, ProfileStatus, UserProfile};
    use crate::ports::command::{CommandOutcome, CommandPort};
    use std::path::Path;

    fn bundle() -> HarnessBundle {
        let mut p = UserProfile::new_draft("d", "en");
        p.candidates.identity.genre = Some(GenreConfidence {
            value: Genre::Developer,
            confidence: 1.0,
            provenance: vec![],
        });
        p.status = ProfileStatus::Confirmed;
        compile_profile(&p).unwrap()
    }

    /// Stub command port where every tool is reported missing — deterministic
    /// AC8: dry-run PASSes via graceful fallback when deps are absent.
    struct AllMissing;
    impl CommandPort for AllMissing {
        fn run(&self, _tool: &str, _args: &[&str], _cwd: Option<&Path>) -> CommandOutcome {
            CommandOutcome::NotInstalled
        }
        fn is_installed(&self, _tool: &str) -> bool {
            false
        }
    }

    #[test]
    fn dryrun_passes_with_fallback_for_missing_deps() {
        // AC8: with deps absent, dry-run still PASSes via graceful fallback.
        let b = bundle();
        let r = dry_run(&b, &AllMissing).unwrap();
        assert!(r.passed(), "{:?}", r);
        assert!(
            !r.fallbacks.is_empty(),
            "expected fallbacks for missing deps"
        );
    }

    #[test]
    fn dryrun_fails_on_empty_pipeline() {
        // Regression: the old gate could not fail; this one must.
        let mut b = bundle();
        b.skills.retain(|s| s.ring != Ring::Ring1);
        let r = dry_run(&b, &AllMissing).unwrap();
        assert!(!r.passed());
        assert!(!r.pipeline_ok);
    }

    #[test]
    fn dryrun_fails_on_empty_skill_body_and_duplicate_ids() {
        let mut b = bundle();
        b.skills[0].body_markdown = "   ".into();
        let dup = b.skills[1].clone();
        b.skills.push(dup);
        let r = dry_run(&b, &AllMissing).unwrap();
        assert!(!r.passed());
        assert!(!r.skills_ok);
        assert!(r.errors.iter().any(|e| e.contains("empty body")));
        assert!(r.errors.iter().any(|e| e.contains("duplicate skill id")));
    }

    #[test]
    fn dryrun_fails_on_unsafe_id() {
        let mut b = bundle();
        b.skills[0].id = "../escape".into();
        let r = dry_run(&b, &AllMissing).unwrap();
        assert!(!r.passed());
        assert!(!r.skills_ok);
    }

    #[test]
    fn dryrun_fails_on_malformed_tool() {
        let mut b = bundle();
        b.mcp_tools.push(crate::domain::bundle::McpTool {
            name: "".into(),
            description: "".into(),
            input_schema: serde_json::json!({}),
        });
        let r = dry_run(&b, &AllMissing).unwrap();
        assert!(!r.passed());
        assert!(!r.tools_list_ok);
    }
}

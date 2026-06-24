//! Static validation gate (ARCH §5.4) — rejects malformed bundles before dry-run.

use crate::domain::bundle::HarnessBundle;
use crate::domain::genre::SafetyGate;

/// Per-check report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticGateReport {
    pub mcp_schema_ok: bool,
    pub hook_input_ok: bool,
    pub safety_gates_ok: bool,
    pub errors: Vec<String>,
}

impl StaticGateReport {
    pub fn passed(&self) -> bool {
        self.errors.is_empty() && self.mcp_schema_ok && self.hook_input_ok && self.safety_gates_ok
    }
}

/// Run all static checks. Returns Err on the first hard violation so the
/// compiler can abort (R8/AC7).
pub fn static_gate(bundle: &HarnessBundle) -> crate::domain::Result<StaticGateReport> {
    let mut report = StaticGateReport {
        mcp_schema_ok: true,
        hook_input_ok: true,
        safety_gates_ok: true,
        errors: Vec::new(),
    };

    // (1) MCP schema conformance: each tool needs name/description/inputSchema.type
    for tool in &bundle.mcp_tools {
        if !tool.is_well_formed() {
            report.mcp_schema_ok = false;
            report.errors.push(format!(
                "mcp tool '{}' missing name/description/inputSchema.type",
                tool.name
            ));
        }
        // inputSchema must be a JSON Schema object with "type".
        if tool.input_schema.get("type").is_none() {
            report.mcp_schema_ok = false;
            report.errors.push(format!(
                "mcp tool '{}' inputSchema missing 'type'",
                tool.name
            ));
        }
    }

    // (2) HookInput required fields present on every hook's reads.
    for hook in &bundle.hooks {
        for field in crate::domain::bundle::HOOK_REQUIRED_FIELDS {
            if !hook.reads.contains(&field.to_string()) {
                report.hook_input_ok = false;
                report.errors.push(format!(
                    "hook '{}' missing HookInput field '{}'",
                    hook.event, field
                ));
            }
        }
    }

    // (3) Safety gates: all three must be present.
    if let Err(e) = SafetyGate::validate_all_present(&bundle.safety_gates) {
        report.safety_gates_ok = false;
        report.errors.push(e.to_string());
    }

    Ok(report)
}

/// Convenience: returns Err on failure (for callers that want to hard-abort).
pub fn static_gate_or_abort(bundle: &HarnessBundle) -> crate::domain::Result<StaticGateReport> {
    let r = static_gate(bundle)?;
    if r.passed() {
        Ok(r)
    } else {
        Err(crate::domain::ByohError::ValidationGateFailed {
            gate: "static",
            reason: r.errors.join("; "),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::compile_profile;
    use crate::domain::bundle::{HookSpec, McpTool};
    use crate::domain::genre::Genre;
    use crate::domain::profile::UserProfile;

    fn good_bundle() -> HarnessBundle {
        let mut p = UserProfile::new_draft("d", "en");
        p.candidates.identity.genre = Some(crate::domain::profile::GenreConfidence {
            value: Genre::Developer,
            confidence: 1.0,
            provenance: vec![],
        });
        p.status = crate::domain::profile::ProfileStatus::Confirmed;
        compile_profile(&p).unwrap()
    }

    #[test]
    fn good_bundle_passes() {
        let b = good_bundle();
        let r = static_gate(&b).unwrap();
        assert!(r.passed(), "{:?}", r.errors);
    }

    #[test]
    fn missing_safety_gate_fails() {
        let mut b = good_bundle();
        // remove stagnation
        b.safety_gates.retain(|g| g != "stagnation");
        let r = static_gate(&b).unwrap();
        assert!(!r.passed());
        assert!(!r.safety_gates_ok);
    }

    #[test]
    fn malformed_mcp_fails() {
        let mut b = good_bundle();
        b.mcp_tools.push(McpTool {
            name: "".into(),
            description: "".into(),
            input_schema: serde_json::json!({}),
        });
        let r = static_gate(&b).unwrap();
        assert!(!r.mcp_schema_ok);
    }

    #[test]
    fn hook_missing_required_field_fails() {
        let mut b = good_bundle();
        b.hooks.push(HookSpec {
            event: "X".into(),
            command: "y".into(),
            reads: vec!["tool_name".into()], // missing the other 3
        });
        let r = static_gate(&b).unwrap();
        assert!(!r.hook_input_ok);
    }
}

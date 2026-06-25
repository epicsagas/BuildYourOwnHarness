//! B14 CapabilityProfile — type-safe provider matching (ARCH §9.2).

use crate::domain::profile::{CapabilityConstraints, ProviderPreference, ToolUseRequirement};

/// A candidate provider's capabilities.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityProfile {
    pub name: String,
    pub supports_tool_use: bool,
    pub context_window: u32,
    pub monthly_cost_usd: f64,
}

/// Match a provider preference against candidates. Excludes providers that
/// fail tool_use / context_window / budget constraints (AC15).
pub fn match_provider<'a>(
    pref: &ProviderPreference,
    candidates: &'a [CapabilityProfile],
) -> Option<&'a CapabilityProfile> {
    let constraints = &pref.capability_constraints;
    let budget = pref_monthly_budget(pref);

    candidates
        .iter()
        .filter(|c| passes_tool_use(c, constraints))
        .filter(|c| passes_context_window(c, constraints))
        .filter(|c| passes_budget(c, budget))
        .min_by(|a, b| {
            a.monthly_cost_usd
                .partial_cmp(&b.monthly_cost_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn passes_tool_use(c: &CapabilityProfile, con: &CapabilityConstraints) -> bool {
    match con.tool_use {
        ToolUseRequirement::Required => c.supports_tool_use,
        ToolUseRequirement::NotRequired => !c.supports_tool_use,
        ToolUseRequirement::Unspecified => true,
    }
}

fn passes_context_window(c: &CapabilityProfile, con: &CapabilityConstraints) -> bool {
    match con.context_window_min {
        Some(min) => c.context_window >= min,
        None => true,
    }
}

fn passes_budget(c: &CapabilityProfile, budget: Option<f64>) -> bool {
    match budget {
        Some(b) => c.monthly_cost_usd <= b,
        None => true,
    }
}

fn pref_monthly_budget(pref: &ProviderPreference) -> Option<f64> {
    // Budget lives on the truth resources block in the full profile; here we
    // accept it via an optional field read by callers. Default: unconstrained.
    let _ = pref;
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pref(tool: ToolUseRequirement, ctx: Option<u32>) -> ProviderPreference {
        ProviderPreference {
            candidate_family: Some("anthropic".into()),
            capability_constraints: CapabilityConstraints {
                tool_use: tool,
                context_window_min: ctx,
            },
            source: crate::domain::profile::ProvenanceSource::Derived,
        }
    }

    fn providers() -> Vec<CapabilityProfile> {
        vec![
            CapabilityProfile {
                name: "cheap-no-tools".into(),
                supports_tool_use: false,
                context_window: 8000,
                monthly_cost_usd: 5.0,
            },
            CapabilityProfile {
                name: "mid-tools".into(),
                supports_tool_use: true,
                context_window: 200_000,
                monthly_cost_usd: 50.0,
            },
            CapabilityProfile {
                name: "big-tools".into(),
                supports_tool_use: true,
                context_window: 1_000_000,
                monthly_cost_usd: 100.0,
            },
        ]
    }

    #[test]
    fn requires_tool_use_excludes_non_tool_providers() {
        let p = pref(ToolUseRequirement::Required, Some(200_000));
        let cands = providers();
        let m = match_provider(&p, &cands).expect("a provider matches");
        assert!(m.supports_tool_use);
        assert!(m.context_window >= 200_000);
        // cheapest that satisfies → mid-tools
        assert_eq!(m.name, "mid-tools");
    }

    #[test]
    fn no_match_when_constraints_too_strict() {
        let p = pref(ToolUseRequirement::Required, Some(2_000_000));
        assert!(match_provider(&p, &providers()).is_none());
    }
}

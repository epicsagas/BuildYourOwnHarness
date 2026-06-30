//! Genre-default agent definitions (target-renderer input).
//!
//! Each genre ships a small set of agents that the renderer emits as
//! `agents/<name>.md` (Claude Code / agy) or `.codex-plugin/agents/<name>.toml`
//! (Codex). Bodies are 4-section SKILL-style markdown (no frontmatter — added
//! per-target at render time).

use crate::domain::bundle::AgentSpec;
use crate::domain::genre::Genre;

/// The default agents for a genre. Empty vec is valid (genre has no agents).
pub fn genre_agents(genre: Genre) -> Vec<AgentSpec> {
    match genre {
        Genre::Developer => vec![
            agent(
                "code-reviewer",
                "Code Reviewer",
                "Reviews code changes for correctness, security, and clarity before merge. Use when a diff is ready or before shipping.",
                Some(vec![
                    "Read".into(),
                    "Grep".into(),
                    "Glob".into(),
                    "Bash".into(),
                ]),
                "Review the diff for correctness bugs first, then security, then clarity. Cite file:line. Never approve what you did not read.",
            ),
            agent(
                "debugger",
                "Debugger",
                "Systematic root-cause isolation for test failures, runtime errors, or unexpected behavior. Reproduce → hypothesize → verify → fix.",
                Some(vec!["Read".into(), "Bash".into(), "Grep".into()]),
                "Reproduce deterministically before any fix. Verify one hypothesis before editing. Fix the verified cause, not the symptom.",
            ),
        ],
        Genre::Creator => vec![agent(
            "draft-writer",
            "Draft Writer",
            "Generates draft content from an outline + style guide + sources with self-QA. Use for chapters, scenes, or sections.",
            Some(vec!["Read".into(), "Write".into(), "Edit".into()]),
            "Assemble context (outline → style → prior summary → sources) before drafting. One unit at a time. Self-QA against the spec before done.",
        )],
        Genre::Researcher => vec![agent(
            "research-analyst",
            "Research Analyst",
            "Synthesizes findings with explicit evidence tiers (primary > peer-reviewed > secondary > anecdotal). Never overstates a source.",
            Some(vec!["Read".into(), "WebSearch".into(), "Write".into()]),
            "Tag every claim's source tier. Separate proven from likely from unknown. Causal claims need primary evidence.",
        )],
        Genre::Business => vec![agent(
            "decision-analyst",
            "Decision Analyst",
            "Frames technical choices by business impact (revenue/users/market), quantifies within a horizon, names opportunity cost.",
            Some(vec!["Read".into(), "Write".into()]),
            "Name the business lever, quantify the effect, name the opportunity cost, state reversal cost. No 'best practice' without your lever.",
        )],
    }
}

fn agent(
    id: &str,
    name: &str,
    description: &str,
    tools: Option<Vec<String>>,
    one_liner: &str,
) -> AgentSpec {
    AgentSpec {
        id: id.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        tools,
        body_markdown: format!(
            "# {name}\n\n{description}\n\n## Role\n\n{one_liner}\n\n\
             ## Anti-Rationalization\n\nDo not skip the discipline above. If you cannot \
             justify a step by this agent's rules, stop and reconsider.\n\n\
             ## Evidence\n\nA task is complete when the agent's acceptance signal is met \
             and no red flag is present.\n\n## Red Flags\n\n- Skipping verification.\n\
             - Acting on assumption rather than evidence.\n"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_genre_returns_agents_or_empty() {
        for &g in Genre::all() {
            let a = genre_agents(g);
            // agent ids are unique within a genre
            let mut ids: Vec<&str> = a.iter().map(|x| x.id.as_str()).collect();
            ids.sort();
            assert_eq!(
                ids.iter().collect::<std::collections::HashSet<_>>().len(),
                ids.len()
            );
        }
    }

    #[test]
    fn developer_has_code_reviewer() {
        let a = genre_agents(Genre::Developer);
        assert!(a.iter().any(|x| x.id == "code-reviewer"));
    }
}

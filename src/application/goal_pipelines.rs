//! Goal-oriented pipeline catalog — skill + agent combos keyed by the user's
//! 30-day goal. Unlike the per-genre domain pipeline (fallback), a goal pipeline
//! is a *purposeful assembly*: "I want to ship a product" → the launch skill
//! ladder + the agents that guard it. [`select_goal_pipeline`] matches the
//! profile's goal tags; [`crate::application::synthesis::synthesize`] injects the
//! matched combo as an overlay on top of the genre defaults.

use crate::domain::genre::Genre;

/// A purposeful skill+agent assembly for a user goal.
pub struct GoalPipeline {
    /// Pipeline id, e.g. `"product-launch"`.
    pub id: &'static str,
    /// Goal keywords that trigger this pipeline (matched against profile tags).
    pub keywords: &'static [&'static str],
    /// Ordered skill ladder: `(skill_id, owning_genre)`.
    pub skills: &'static [(&'static str, Genre)],
    /// Agent set to include: `(agent_id, owning_genre)`.
    pub agents: &'static [(&'static str, Genre)],
    pub description: &'static str,
}

/// The goal-pipeline library. Order matters only for readability — selection is
/// keyword-based; the first match wins.
pub fn goal_pipeline_catalog() -> &'static [GoalPipeline] {
    use Genre::*;
    &[
        GoalPipeline {
            id: "product-launch",
            keywords: &["launch", "ship", "release", "product", "mvp", "startup"],
            skills: &[
                ("discover", Developer),
                ("mvp-force", Business),
                ("vuln-scan", Developer),
                ("ship-over-perfect", Business),
            ],
            agents: &[
                ("code-reviewer", Developer),
                ("debugger", Developer),
                ("decision-analyst", Business),
            ],
            description: "Take a product from idea to launch: discover → MVP → security → ship.",
        },
        GoalPipeline {
            id: "market-analysis",
            keywords: &["market", "analysis", "analyze", "competitor", "landscape"],
            skills: &[
                ("five-whys", Business),
                ("biz-risk", Business),
                ("decision", Business),
                ("document", Developer),
            ],
            agents: &[
                ("decision-analyst", Business),
                ("research-analyst", Researcher),
            ],
            description: "Frame a market question, quantify risk, decide, document.",
        },
        GoalPipeline {
            id: "decision",
            keywords: &["decision", "decide", "choose", "strategy", "prioritize"],
            skills: &[
                ("biz-risk", Business),
                ("devils-advocate", Business),
                ("decision", Business),
                ("plainlanguage", Business),
            ],
            agents: &[("decision-analyst", Business)],
            description: "Make a defensible decision: risk → counter → decide → communicate.",
        },
        GoalPipeline {
            id: "research-report",
            keywords: &["research", "report", "study", "investigate", "whitepaper"],
            skills: &[
                ("evidence", Researcher),
                ("reproducibility", Researcher),
                ("document", Developer),
            ],
            agents: &[("research-analyst", Researcher)],
            description: "Produce a cited, reproducible research report.",
        },
        GoalPipeline {
            id: "content-create",
            keywords: &["content", "write", "blog", "book", "article", "create"],
            skills: &[("continuity", Creator)],
            agents: &[("draft-writer", Creator), ("consistency-editor", Creator)],
            description: "Draft and consistency-check long-form content.",
        },
        GoalPipeline {
            id: "secure-ship",
            keywords: &["security", "secure", "vulnerability", "audit", "compliance"],
            skills: &[
                ("threat-model", Developer),
                ("vuln-scan", Developer),
                ("triage", Developer),
                ("verify", Developer),
            ],
            agents: &[("code-reviewer", Developer)],
            description: "Harden before shipping: threat-model → scan → triage → verify.",
        },
    ]
}

/// First goal pipeline whose keywords match any tag (case-insensitive substring).
pub fn select_goal_pipeline(tags: &[String]) -> Option<&'static GoalPipeline> {
    let lower: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();
    goal_pipeline_catalog().iter().find(|gp| {
        gp.keywords
            .iter()
            .any(|k| lower.iter().any(|t| t.contains(k)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_six_goal_pipelines() {
        assert_eq!(goal_pipeline_catalog().len(), 6);
    }

    #[test]
    fn launch_goal_matches() {
        let gp = select_goal_pipeline(&["ship".into(), "a product".into()]).unwrap();
        assert_eq!(gp.id, "product-launch");
        assert!(gp.skills.iter().any(|(s, _)| *s == "mvp-force"));
        assert!(gp.agents.iter().any(|(a, _)| *a == "code-reviewer"));
    }

    #[test]
    fn decision_goal_matches() {
        let gp = select_goal_pipeline(&["decide".into(), "architecture".into()]).unwrap();
        assert_eq!(gp.id, "decision");
    }

    #[test]
    fn no_match_returns_none() {
        assert!(select_goal_pipeline(&["cooking".into()]).is_none());
    }
}

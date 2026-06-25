//! Synthesis domain types (PR #4 — synthesis engine).
//!
//! Pure types, no I/O. A [`SynthesisPlan`] describes *which* registry skills to
//! assemble into *which* ordered pipelines, producing a HarnessBundle that is
//! NOT a fixed genre template but a recombination of vetted skills.
//!
//! The plan is recorded into the bundle (`config.extra["synthesis_plan"]`) so a
//! synthesized bundle is reproducible/diffable — not a black box.

use serde::{Deserialize, Serialize};

use super::bundle::Ring;

/// A complete synthesis plan: the keyword tags driving selection + the pipelines
/// to assemble. Produced by the orchestrator's matching step, consumed by the
/// assembly step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SynthesisPlan {
    /// Profile-derived keyword tags that drove skill selection (e.g.
    /// `["backend", "rust", "security"]`). Recorded for reproducibility.
    pub tags: Vec<String>,
    /// Ordered pipelines to assemble into the bundle.
    pub pipelines: Vec<PipelineDef>,
}

/// An ordered chain of skills within one ring. Steps run in `order`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelineDef {
    /// Pipeline id, e.g. `"shorts-scenario"` or `"research-report"`.
    pub id: String,
    /// Which ring the pipeline lives in.
    pub ring: Ring,
    /// Ordered steps. Validation guarantees step.skill_id resolves in the
    /// registry and no cycles across pipelines (this orbit: simple checks only).
    pub steps: Vec<PipelineStep>,
    /// Human-readable description of what this pipeline produces.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
}

/// One step in a pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelineStep {
    /// Registry skill id to run at this step.
    pub skill_id: String,
    /// 1-based position within the pipeline.
    pub order: u32,
    /// Skills this step depends on (must appear earlier). Used for the simple
    /// dependency check (references resolved against the plan, not a full DAG
    /// topological solver — that's a follow-up orbit).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
}

impl PipelineDef {
    /// Steps sorted by `order` (stable). Steps are authored in order already, but
    /// callers may supply them unordered; this normalizes.
    pub fn steps_in_order(&self) -> Vec<&PipelineStep> {
        let mut idx: Vec<&PipelineStep> = self.steps.iter().collect();
        idx.sort_by_key(|s| s.order);
        idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_round_trips_json() {
        let plan = SynthesisPlan {
            tags: vec!["backend".into(), "security".into()],
            pipelines: vec![PipelineDef {
                id: "secure-ship".into(),
                ring: Ring::Ring1,
                steps: vec![
                    PipelineStep {
                        skill_id: "tdd".into(),
                        order: 1,
                        depends_on: vec![],
                    },
                    PipelineStep {
                        skill_id: "debug".into(),
                        order: 2,
                        depends_on: vec!["tdd".into()],
                    },
                ],
                description: "test then debug".into(),
            }],
        };
        let json = serde_json::to_string(&plan).unwrap();
        let back: SynthesisPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(back, plan);
    }

    #[test]
    fn steps_in_order_sorts_unordered_input() {
        let def = PipelineDef {
            id: "p".into(),
            ring: Ring::Ring2,
            steps: vec![
                PipelineStep {
                    skill_id: "b".into(),
                    order: 2,
                    depends_on: vec![],
                },
                PipelineStep {
                    skill_id: "a".into(),
                    order: 1,
                    depends_on: vec![],
                },
            ],
            description: String::new(),
        };
        let ordered: Vec<&str> = def
            .steps_in_order()
            .iter()
            .map(|s| s.skill_id.as_str())
            .collect();
        assert_eq!(ordered, vec!["a", "b"]);
    }
}

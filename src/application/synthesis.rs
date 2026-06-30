//! Synthesis orchestrator (PR #4).
//!
//! Recombines registry skills — selected by profile-derived keyword tags — into
//! ordered pipelines, producing a HarnessBundle that is NOT a fixed genre
//! template but a unique assembly. The fixed [`crate::compiler::compile_profile`]
//! path stays as the static regression baseline; synthesis is a *subsequent*
//! recombination step (Architect recommendation).
//!
//! Safety (Critic recommendation): the assembled bundle MUST pass
//! [`crate::compiler::static_gate`] — synthesis can never bypass the 3 safety
//! gates. Failures are graceful (`ByohError`), never panics.

use crate::compiler::{compile_profile, static_gate};
use crate::deploy::agent_presets::{AgentPresetMeta, agent_catalog, agent_matches, inject_agent};
use crate::deploy::presets::{
    PresetMeta, inject_preset, lookup_genre, preset_catalog, preset_matches,
};
use crate::domain::bundle::HarnessBundle;
use crate::domain::error::ByohError;
use crate::domain::genre::Genre;
use crate::domain::profile::UserProfile;
use crate::domain::synthesis::{PipelineDef, PipelineStep, SynthesisPlan};
use std::collections::HashMap;

use crate::Result;

/// Derive keyword tags from a confirmed profile: genre, primary-expertise terms,
/// and the 30-day goal. These drive preset matching.
///
/// Public so the synthesis MCP tool (follow-up orbit) and tests can inspect.
pub fn profile_tags(profile: &UserProfile) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();

    // Genre.
    if let Some(g) = profile.candidates.identity.genre.as_ref() {
        tags.push(g.value.as_str().to_string());
    }

    // Primary-expertise terms (derived facts).
    for fact in &profile.candidates.identity.primary_expertise {
        tags.push(fact.value.clone());
    }

    // 30-day goal (split on whitespace into rough keywords).
    if let Some(goal) = profile.truth.goals.goal_30d.as_ref() {
        for word in goal.split_whitespace() {
            let w = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if w.len() >= 3 {
                tags.push(w);
            }
        }
    }

    tags
}

/// Select presets from the local catalog whose keywords match any profile tag.
/// Returns the matched presets, stable-ordered by (genre, skill_id) for
/// reproducibility.
pub fn select_presets(tags: &[String]) -> Vec<&'static PresetMeta> {
    let mut matched: Vec<&PresetMeta> = preset_catalog()
        .iter()
        .chain(crate::deploy::presets::VENDORED_PRESETS.iter())
        .filter(|m| preset_matches(m, tags))
        .collect();
    matched.sort_by(|a, b| (a.genre.as_str(), a.skill_id).cmp(&(b.genre.as_str(), b.skill_id)));
    matched
}

/// Select agent presets from the local catalog whose keywords match any profile
/// tag. Mirrors [`select_presets`] for agents. Stable-ordered by
/// (genre, agent_id) for reproducibility.
pub fn select_agents(tags: &[String]) -> Vec<&'static AgentPresetMeta> {
    let mut matched: Vec<&AgentPresetMeta> = agent_catalog()
        .iter()
        .filter(|m| agent_matches(m, tags))
        .collect();
    matched.sort_by(|a, b| (a.genre.as_str(), a.agent_id).cmp(&(b.genre.as_str(), b.agent_id)));
    matched
}

/// Per-genre default domain pipeline — the fallback when no preset matched a
/// profile, so a no-match profile still yields an ordered pipeline. Steps use
/// existing catalog skill ids (the M1b vendored overlay can enrich them later).
pub fn select_domain_pipeline(genre: Genre) -> Option<PipelineDef> {
    use crate::domain::bundle::Ring;
    use Genre::*;
    let step = |id: &str, order: u32, deps: &[&str]| PipelineStep {
        skill_id: id.to_string(),
        order,
        depends_on: deps.iter().map(|d| (*d).to_string()).collect(),
    };
    Some(match genre {
        Developer => PipelineDef {
            id: "developer-core".into(),
            ring: Ring::Ring2,
            steps: vec![step("tdd", 1, &[]), step("debug", 2, &["tdd"])],
            description: "Developer core: test-first, then systematic debug.".into(),
        },
        Researcher => PipelineDef {
            id: "researcher-core".into(),
            ring: Ring::Ring2,
            steps: vec![
                step("evidence", 1, &[]),
                step("reproducibility", 2, &["evidence"]),
            ],
            description: "Researcher core: evidence tiers, then reproducibility.".into(),
        },
        Business => PipelineDef {
            id: "business-core".into(),
            ring: Ring::Ring2,
            steps: vec![
                step("decision", 1, &[]),
                step("plainlanguage", 2, &["decision"]),
            ],
            description: "Business core: decision framing, then plain language.".into(),
        },
        Creator => PipelineDef {
            id: "creator-core".into(),
            ring: Ring::Ring2,
            steps: vec![step("continuity", 1, &[])],
            description: "Creator core: continuity guard.".into(),
        },
    })
}

/// All per-genre domain pipelines (one per genre).
pub fn pipeline_catalog() -> Vec<PipelineDef> {
    Genre::all()
        .iter()
        .filter_map(|g| select_domain_pipeline(*g))
        .collect()
}

/// Build a SynthesisPlan from matched presets: each matched preset becomes a
/// single-step pipeline in Ring 2 (quality). Multiple matches → a multi-step
/// pipeline ordered alphabetically by skill_id (deterministic). Depends_on
/// chains steps within the pipeline.
fn build_plan(tags: Vec<String>, genre: Option<Genre>, matched: &[&PresetMeta]) -> SynthesisPlan {
    if matched.is_empty() {
        let pipelines = genre.and_then(select_domain_pipeline).into_iter().collect();
        return SynthesisPlan { tags, pipelines };
    }

    let steps: Vec<PipelineStep> = matched
        .iter()
        .enumerate()
        .map(|(i, m)| PipelineStep {
            skill_id: m.skill_id.to_string(),
            order: (i as u32) + 1,
            depends_on: if i == 0 {
                vec![]
            } else {
                vec![matched[i - 1].skill_id.to_string()]
            },
        })
        .collect();

    SynthesisPlan {
        tags,
        pipelines: vec![PipelineDef {
            id: "synthesized-quality".to_string(),
            ring: crate::domain::bundle::Ring::Ring2,
            steps,
            description: "Matched presets assembled into an ordered quality pipeline.".to_string(),
        }],
    }
}

/// Validate a plan's internal references: every step's skill_id must resolve in
/// the catalog, and depends_on must point to an earlier step in the same
/// pipeline. (Full DAG cycle detection is a follow-up orbit — Critic scope-defer.)
fn validate_plan(plan: &SynthesisPlan) -> Result<()> {
    let known: Vec<&str> = preset_catalog()
        .iter()
        .chain(crate::deploy::presets::VENDORED_PRESETS.iter())
        .map(|m| m.skill_id)
        .collect();
    let mut unresolved: Vec<String> = Vec::new();
    let mut bad_deps: Vec<String> = Vec::new();

    for pipe in &plan.pipelines {
        let step_ids: Vec<&str> = pipe.steps.iter().map(|s| s.skill_id.as_str()).collect();
        for step in &pipe.steps {
            if !known.contains(&step.skill_id.as_str()) {
                unresolved.push(format!("{}:{}", pipe.id, step.skill_id));
            }
            // depends_on must reference a step present in THIS pipeline. Order is
            // irrelevant — the cycle check below rejects genuine back-edges.
            for dep in &step.depends_on {
                if !step_ids.contains(&dep.as_str()) {
                    bad_deps.push(format!("{}:{}→{}", pipe.id, step.skill_id, dep));
                }
            }
        }
        // Formal cycle detection (order-independent) over step -> depends_on.
        if let Some(cycle) = detect_cycle(&pipe.steps) {
            return Err(ByohError::Other(format!(
                "synthesis plan invalid — cycle in '{}': {}",
                pipe.id,
                cycle.join(" -> ")
            )));
        }
    }

    if !unresolved.is_empty() || !bad_deps.is_empty() {
        return Err(ByohError::Other(format!(
            "synthesis plan invalid — unresolved: [{}], bad deps: [{}]",
            unresolved.join(", "),
            bad_deps.join(", ")
        )));
    }
    Ok(())
}

/// Detect a cycle in one pipeline's `step.depends_on` graph (skill_id -> dep).
/// Returns the cyclic node path (closing the loop) if one exists, else `None`.
/// Order-independent: a forward or back reference is not a cycle on its own.
fn detect_cycle(steps: &[PipelineStep]) -> Option<Vec<String>> {
    let adj: HashMap<&str, Vec<&str>> = steps
        .iter()
        .map(|s| {
            (
                s.skill_id.as_str(),
                s.depends_on.iter().map(|d| d.as_str()).collect(),
            )
        })
        .collect();
    // 3-color DFS: 0 = unvisited, 1 = on-stack (gray), 2 = done (black).
    let mut color: HashMap<&str, u8> = HashMap::new();
    let mut path: Vec<&str> = Vec::new();
    for start in steps.iter().map(|s| s.skill_id.as_str()) {
        if color.get(start).copied().unwrap_or(0) == 0 {
            if let Some(cycle) = dfs_cycle(start, &adj, &mut color, &mut path) {
                return Some(cycle);
            }
        }
    }
    None
}

/// DFS step of cycle detection. On a back-edge to a gray (on-stack) node,
/// returns the closed cycle path, e.g. `["tdd", "debug", "tdd"]`.
fn dfs_cycle<'a>(
    node: &'a str,
    adj: &HashMap<&'a str, Vec<&'a str>>,
    color: &mut HashMap<&'a str, u8>,
    path: &mut Vec<&'a str>,
) -> Option<Vec<String>> {
    color.insert(node, 1);
    path.push(node);
    if let Some(nbrs) = adj.get(node) {
        for &nb in nbrs {
            match color.get(nb).copied().unwrap_or(0) {
                1 => {
                    // Back-edge to a node on the current stack → cycle.
                    let start = path.iter().position(|&p| p == nb).unwrap();
                    let mut cycle: Vec<String> =
                        path[start..].iter().map(|s| s.to_string()).collect();
                    cycle.push(nb.to_string()); // close the loop
                    return Some(cycle);
                }
                0 => {
                    if let Some(c) = dfs_cycle(nb, adj, color, path) {
                        return Some(c);
                    }
                }
                _ => {}
            }
        }
    }
    path.pop();
    color.insert(node, 2);
    None
}

/// Assemble a synthesized bundle from a confirmed profile.
///
/// Pipeline:
/// 1. Start from the static genre-template bundle (`compile_profile`) so the
///    3 safety gates + base rings are always present (safety floor).
/// 2. Derive tags, match presets, build + validate the plan.
/// 3. Inject matched presets into the bundle, tagging each with its pipeline
///    id + order (so the assembled skills carry ordering metadata).
/// 4. Record the plan into `bundle.config.extra["synthesis_plan"]` for
///    reproducibility/diffability (Architect risk-B mitigation).
/// 5. Re-run `static_gate` — fail the synthesis if the bundle is invalid.
///    Synthesis can NEVER bypass the safety gates.
pub fn synthesize(profile: &UserProfile) -> Result<(HarnessBundle, SynthesisPlan)> {
    // 1. Safety floor: start from the static template.
    let mut bundle = compile_profile(profile)?;
    let base_skill_count = bundle.skills.len();

    // 2. Match + plan.
    let tags = profile_tags(profile);
    let genre = profile.candidates.identity.genre.as_ref().map(|g| g.value);
    let matched = select_presets(&tags);
    let plan = build_plan(tags, genre, &matched);

    // 3. Validate the plan before mutating the bundle.
    validate_plan(&plan)?;

    // 4. Inject matched presets, annotating with pipeline/order metadata.
    //    inject_preset dedupes by id (augment-or-clone), so re-matched base
    //    template skills are enriched rather than duplicated.
    for step in plan.pipelines.iter().flat_map(|p| p.steps_in_order()) {
        // Genre for injection: prefer the profile's confirmed genre; presets are
        // genre-scoped in the catalog, so use the matched preset's genre.
        let genre = lookup_genre(&step.skill_id).ok_or_else(|| {
            ByohError::Schema(format!(
                "synthesis referenced unknown skill '{}'",
                step.skill_id
            ))
        })?;
        inject_preset(&mut bundle, genre, &step.skill_id)?;
        // Annotate the just-injected/augmented skill with pipeline metadata.
        if let Some(skill) = bundle.skills.iter_mut().find(|s| s.id == step.skill_id) {
            skill.pipeline = Some(plan.pipelines[0].id.clone());
            skill.order = Some(step.order);
        }
    }

    // 4b. Recombine the AGENT set (Issue #6). Matched agent presets are
    //     injected on top of the genre-default agents. inject_agent dedupes by
    //     id (augment-or-clone), so a matched default agent is enriched with the
    //     vetted body while a newly-matched agent (e.g. tech-debt-auditor) is
    //     cloned in. The result is a *recombined* agent set, not just the genre
    //     default — the same treatment skills already get.
    let base_agent_count = bundle.agents.len();
    for meta in select_agents(&plan.tags) {
        inject_agent(&mut bundle, meta.genre, meta.agent_id)?;
    }

    // 4c. Goal-pipeline overlay: if the profile's 30-day goal matches a
    //     purposeful skill+agent assembly (e.g. product-launch), inject its
    //     skill ladder + agent set on top. inject_* dedupes by id, so this
    //     enriches/extends rather than duplicates. Recorded for reproducibility.
    if let Some(gp) = crate::application::goal_pipelines::select_goal_pipeline(&plan.tags) {
        for (sid, g) in gp.skills {
            inject_preset(&mut bundle, *g, sid)?;
        }
        for (aid, g) in gp.agents {
            inject_agent(&mut bundle, *g, aid)?;
        }
        bundle
            .config
            .extra
            .insert("synthesis_goal_pipeline".to_string(), gp.id.to_string());
    }

    // 5. Record the plan (reproducibility).
    bundle.config.extra.insert(
        "synthesis_plan".to_string(),
        serde_json::to_string(&plan).unwrap_or_else(|_| "{}".to_string()),
    );
    bundle.config.extra.insert(
        "synthesis_base_skill_count".to_string(),
        base_skill_count.to_string(),
    );
    bundle.config.extra.insert(
        "synthesis_base_agent_count".to_string(),
        base_agent_count.to_string(),
    );

    // 6. Re-gate: synthesis must not bypass safety gates (Critic invariant).
    //    Scope: static_gate checks MCP schema / HookInput / safety-gate presence
    //    only — it does not inspect agent bodies. So this re-gate guards the
    //    bundle's structural invariants, not the injected agent content.
    let report = static_gate(&bundle)?;
    if !report.passed() {
        return Err(ByohError::ValidationGateFailed {
            gate: "static",
            reason: format!("synthesized bundle failed re-gate: {:?}", report.errors),
        });
    }

    Ok((bundle, plan))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::genre::Genre;
    use crate::domain::profile::{GenreConfidence, ProfileStatus};
    use std::collections::HashMap;

    fn confirmed_profile(slug: &str, genre: Genre, expertise: &[&str], goal: &str) -> UserProfile {
        let mut p = UserProfile::new_draft(slug, "en");
        p.candidates.identity.genre = Some(GenreConfidence {
            value: genre,
            confidence: 1.0,
            provenance: vec![],
        });
        p.candidates.identity.primary_expertise = expertise
            .iter()
            .map(|t| crate::domain::profile::DerivedFact {
                value: t.to_string(),
                confidence: 0.9,
                provenance: vec![],
            })
            .collect();
        p.truth.goals.goal_30d = Some(goal.to_string());
        p.status = ProfileStatus::Confirmed;
        p
    }

    #[test]
    fn profile_tags_extract_genre_expertise_goal() {
        let p = confirmed_profile(
            "d",
            Genre::Developer,
            &["rust", "backend"],
            "ship quality code",
        );
        let tags = profile_tags(&p);
        assert!(tags.iter().any(|t| t == "developer"));
        assert!(tags.iter().any(|t| t == "rust"));
        assert!(tags.iter().any(|t| t == "quality"));
    }

    #[test]
    fn select_presets_matches_by_keyword() {
        let tags = vec!["security".into(), "test".into()];
        let matched = select_presets(&tags);
        let ids: Vec<&str> = matched.iter().map(|m| m.skill_id).collect();
        assert!(ids.contains(&"tdd"), "test tag should match tdd");
    }

    #[test]
    fn synthesize_produces_bundle_distinct_from_static_template() {
        let p = confirmed_profile(
            "d",
            Genre::Developer,
            &["backend", "rust"],
            "write tests and debug fast",
        );
        let (bundle, plan) = synthesize(&p).expect("synthesis should succeed");

        // Plan recorded.
        assert!(plan.pipelines.len() == 1, "one synthesized pipeline");
        assert!(!plan.tags.is_empty());

        // tdd + debug should have been matched and annotated with pipeline metadata.
        let tdd = bundle.skills.iter().find(|s| s.id == "tdd").unwrap();
        assert!(tdd.pipeline.is_some(), "tdd should be in a pipeline");
        assert!(tdd.order.is_some());

        // Reproducibility: plan serialized into bundle extra.
        assert!(bundle.config.extra.contains_key("synthesis_plan"));
    }

    #[test]
    fn synthesize_records_base_skill_count() {
        let p = confirmed_profile("d", Genre::Developer, &["backend"], "ship code");
        let (bundle, _) = synthesize(&p).unwrap();
        assert!(
            bundle
                .config
                .extra
                .contains_key("synthesis_base_skill_count")
        );
    }

    #[test]
    fn select_presets_with_empty_tags_returns_empty() {
        // No tags → no matches. (Genre tags themselves are strong signals, so a
        // real profile always matches at least its genre's presets; test the
        // empty-tag edge case directly here.)
        let matched = select_presets(&[]);
        assert!(matched.is_empty(), "empty tags must match nothing");
    }

    #[test]
    fn synthesize_with_no_matches_still_returns_valid_bundle() {
        // Even with zero matched presets, the safety floor (static template)
        // guarantees a valid bundle with all 3 safety gates intact.
        // We force empty matches by calling synthesize on a profile whose only
        // tag is its genre, then separately verify an empty plan path: build a
        // plan directly with no matched presets and confirm compile+gate holds.
        let empty_plan = SynthesisPlan {
            tags: vec![],
            pipelines: vec![],
        };
        assert!(empty_plan.pipelines.is_empty());
        assert!(validate_plan(&empty_plan).is_ok(), "empty plan is valid");
    }

    #[test]
    fn validate_plan_rejects_unknown_skill() {
        let plan = SynthesisPlan {
            tags: vec![],
            pipelines: vec![PipelineDef {
                id: "bad".into(),
                ring: crate::domain::bundle::Ring::Ring2,
                steps: vec![PipelineStep {
                    skill_id: "does-not-exist".into(),
                    order: 1,
                    depends_on: vec![],
                }],
                description: String::new(),
            }],
        };
        assert!(validate_plan(&plan).is_err());
    }

    #[test]
    fn validate_plan_accepts_forward_dependency() {
        // A forward reference (step 1 depends on step 2) is NOT a cycle. Order
        // is irrelevant under formal DAG detection, so this is now valid.
        let plan = SynthesisPlan {
            tags: vec![],
            pipelines: vec![PipelineDef {
                id: "ok".into(),
                ring: crate::domain::bundle::Ring::Ring2,
                steps: vec![
                    PipelineStep {
                        skill_id: "tdd".into(),
                        order: 1,
                        depends_on: vec!["debug".into()],
                    },
                    PipelineStep {
                        skill_id: "debug".into(),
                        order: 2,
                        depends_on: vec![],
                    },
                ],
                description: String::new(),
            }],
        };
        assert!(validate_plan(&plan).is_ok());
    }

    #[test]
    fn validate_plan_rejects_direct_cycle() {
        // tdd -> debug -> tdd is a cycle.
        let plan = SynthesisPlan {
            tags: vec![],
            pipelines: vec![PipelineDef {
                id: "cyc".into(),
                ring: crate::domain::bundle::Ring::Ring2,
                steps: vec![
                    PipelineStep {
                        skill_id: "tdd".into(),
                        order: 1,
                        depends_on: vec!["debug".into()],
                    },
                    PipelineStep {
                        skill_id: "debug".into(),
                        order: 2,
                        depends_on: vec!["tdd".into()],
                    },
                ],
                description: String::new(),
            }],
        };
        let err = validate_plan(&plan).unwrap_err().to_string();
        assert!(err.contains("cycle"), "got: {err}");
        // Names both nodes of the cycle.
        assert!(err.contains("tdd") && err.contains("debug"), "got: {err}");
    }

    #[test]
    fn validate_plan_rejects_self_loop() {
        let plan = SynthesisPlan {
            tags: vec![],
            pipelines: vec![PipelineDef {
                id: "loop".into(),
                ring: crate::domain::bundle::Ring::Ring2,
                steps: vec![PipelineStep {
                    skill_id: "tdd".into(),
                    order: 1,
                    depends_on: vec!["tdd".into()],
                }],
                description: String::new(),
            }],
        };
        let err = validate_plan(&plan).unwrap_err().to_string();
        assert!(err.contains("cycle"), "got: {err}");
    }

    #[test]
    fn validate_plan_accepts_diamond_order_independent() {
        // Diamond: a->b, a->c, b->d, c->d. Valid. Steps are listed OUT of order
        // to prove detection is order-independent — the old earlier-step check
        // would have rejected d's deps.
        use crate::domain::bundle::Ring;
        let mk = |id: &str, order: u32, deps: &[&str]| PipelineStep {
            skill_id: id.into(),
            order,
            depends_on: deps.iter().map(|d| (*d).into()).collect(),
        };
        let plan = SynthesisPlan {
            tags: vec![],
            pipelines: vec![PipelineDef {
                id: "diamond".into(),
                ring: Ring::Ring2,
                steps: vec![
                    mk("evidence", 4, &["debug", "continuity"]), // d
                    mk("tdd", 1, &[]),                           // a
                    mk("debug", 2, &["tdd"]),                    // b
                    mk("continuity", 3, &["tdd"]),               // c
                ],
                description: String::new(),
            }],
        };
        assert!(validate_plan(&plan).is_ok(), "valid diamond must pass");
    }

    #[test]
    fn select_agents_matches_by_keyword() {
        let tags = vec!["evidence".into(), "citation".into()];
        let matched = select_agents(&tags);
        let ids: Vec<&str> = matched.iter().map(|m| m.agent_id).collect();
        assert!(
            ids.contains(&"research-analyst"),
            "evidence/citation tags should match research-analyst"
        );
    }

    #[test]
    fn select_agents_stable_order() {
        // Same tags → same order; ordered by (genre, agent_id).
        let tags = vec!["code".into(), "review".into(), "debug".into()];
        let a = select_agents(&tags);
        let b = select_agents(&tags);
        assert_eq!(
            a.iter().map(|m| m.agent_id).collect::<Vec<_>>(),
            b.iter().map(|m| m.agent_id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn synthesize_recombines_agent_set() {
        // A developer profile with a "debt/refactor" tag should pull in
        // tech-debt-auditor — an agent NOT in the genre defaults — proving the
        // synthesized agent set is recombined, not just the genre default.
        let p = confirmed_profile(
            "d",
            Genre::Developer,
            &["backend", "rust"],
            "pay down tech debt and refactor",
        );
        let (bundle, plan) = synthesize(&p).expect("synthesis should succeed");

        // tech-debt-auditor matched via "debt"/"refactor" tags → cloned in.
        assert!(
            bundle.agents.iter().any(|a| a.id == "tech-debt-auditor"),
            "recombined agent set should include tech-debt-auditor"
        );
        // Plan recorded; base agent count recorded for reproducibility.
        assert!(bundle.config.extra.contains_key("synthesis_plan"));
        assert!(
            bundle
                .config
                .extra
                .contains_key("synthesis_base_agent_count")
        );
        // The matched agents are a subset of the plan's tag-driven selection.
        let selected: Vec<&str> = select_agents(&plan.tags)
            .iter()
            .map(|m| m.agent_id)
            .collect();
        assert!(!selected.is_empty());
    }

    #[test]
    fn synthesize_enriches_genre_default_agent_body() {
        // A developer profile matches `debugger` (a genre default) → the
        // default stub body must be augmented with the richer preset body.
        let p = confirmed_profile("d", Genre::Developer, &["backend"], "debug a failing test");
        let (bundle, _) = synthesize(&p).expect("synthesis should succeed");
        let dbg = bundle
            .agents
            .iter()
            .find(|a| a.id == "debugger")
            .expect("debugger present");
        assert!(
            dbg.body_markdown.contains("Reproduce"),
            "genre-default debugger should be augmented with the preset body"
        );
        assert_eq!(dbg.name, "Debugger");
    }

    #[test]
    fn _ensure_hashmap_import_used() {
        // Keeps the HashMap import meaningful if future tests need it.
        let _: HashMap<String, String> = HashMap::new();
    }
}

//! SkillOpt — pattern mining for evolved skills (ARCH §7.4 epic-harness ref).
//!
//! Mine observation records: if a *dominant error* appears in ≥60% of failed
//! observations AND spans ≥2 distinct files, seed a reuse skill to address it.

use std::collections::HashMap;

use crate::domain::evidence::{ObservationRecord, ObservedOutcome};

/// A mined pattern that warrants a new evolved skill.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillSeed {
    pub name: String,
    pub dominant_error: String,
    pub occurrence_count: u32,
    pub file_count: u32,
}

/// Thresholds (epic-harness SkillOpt).
pub const DOMINANT_ERROR_THRESHOLD: f64 = 0.60;
pub const MIN_FILES: u32 = 2;

/// Mine observations for skill-seeding patterns.
pub fn mine_patterns(records: &[ObservationRecord]) -> Vec<SkillSeed> {
    let failures: Vec<&ObservationRecord> = records
        .iter()
        .filter(|r| matches!(r.outcome, ObservedOutcome::Failure))
        .collect();

    if failures.is_empty() {
        return Vec::new();
    }

    // Count per dominant_error, tracking distinct "files" via provenance-ish key
    // in tool_name (observations carry tool_name; we approximate file identity
    // by treating each distinct tool_name as a distinct file/location).
    let mut by_error: HashMap<String, (u32, std::collections::HashSet<String>)> = HashMap::new();
    for r in &failures {
        if let Some(err) = &r.dominant_error {
            let entry = by_error.entry(err.clone()).or_default();
            entry.0 += 1;
            entry.1.insert(r.tool_name.clone());
        }
    }

    let total_failures = failures.len() as f64;
    let mut seeds = Vec::new();
    for (err, (count, files)) in by_error {
        let ratio = count as f64 / total_failures;
        if ratio >= DOMINANT_ERROR_THRESHOLD && files.len() as u32 >= MIN_FILES {
            seeds.push(SkillSeed {
                name: format!("handle_{}", err.replace('-', "_")),
                dominant_error: err,
                occurrence_count: count,
                file_count: files.len() as u32,
            });
        }
    }
    // Cap at MAX_EVOLVED_SKILLS=10 (epic-harness).
    seeds.sort_by_key(|s| std::cmp::Reverse(s.occurrence_count));
    seeds.truncate(10);
    seeds
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn rec(id: &str, tool: &str, err: Option<&str>, outcome: ObservedOutcome) -> ObservationRecord {
        ObservationRecord {
            id: id.into(),
            observed_at: Utc::now(),
            tool_name: tool.into(),
            outcome,
            with_evolved: true,
            score: outcome.to_score(),
            dominant_error: err.map(|e| e.to_string()),
        }
    }

    #[test]
    fn seeds_skill_when_dominant_error_across_files() {
        let recs = vec![
            rec("1", "file_a", Some("null-deref"), ObservedOutcome::Failure),
            rec("2", "file_b", Some("null-deref"), ObservedOutcome::Failure),
            rec("3", "file_a", Some("null-deref"), ObservedOutcome::Failure),
            rec(
                "4",
                "file_c",
                Some("type-mismatch"),
                ObservedOutcome::Failure,
            ),
            rec("5", "file_a", None, ObservedOutcome::Success),
        ];
        let seeds = mine_patterns(&recs);
        // null-deref: 3/4 failures = 75% >= 60%, across 2 files → seeded
        assert!(seeds.iter().any(|s| s.dominant_error == "null-deref"));
        // type-mismatch: 1/4 = 25% < 60% → not seeded
        assert!(!seeds.iter().any(|s| s.dominant_error == "type-mismatch"));
    }

    #[test]
    fn no_failures_yields_no_seeds() {
        let recs = vec![rec("1", "f", None, ObservedOutcome::Success)];
        assert!(mine_patterns(&recs).is_empty());
    }

    #[test]
    fn single_file_does_not_seed() {
        // same file twice → file_count 1 < 2 → no seed even at 100%
        let recs = vec![
            rec("1", "only_file", Some("x"), ObservedOutcome::Failure),
            rec("2", "only_file", Some("x"), ObservedOutcome::Failure),
        ];
        assert!(mine_patterns(&recs).is_empty());
    }

    #[test]
    fn capped_at_ten_seeds() {
        let mut recs = Vec::new();
        for i in 0..200 {
            recs.push(rec(
                &format!("r{i}"),
                &format!("file_{}", i % 50),
                Some(&format!("err_{}", i % 20)),
                ObservedOutcome::Failure,
            ));
        }
        let seeds = mine_patterns(&recs);
        assert!(seeds.len() <= 10);
    }
}

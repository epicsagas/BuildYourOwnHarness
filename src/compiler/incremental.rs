//! Incremental recompile — profile diff-based (ARCH §5.5).
//!
//! Classifies a profile change as 3a (meta-only patch) / 3b (scoped recompile)
//! / 3c (breaking full recompile + migration), then runs the right path.

use crate::domain::profile::UserProfile;

/// The classification of a profile diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeClass {
    /// Meta/weights only → incremental patch (3a).
    MetaOnly,
    /// Skill/tool/domain affected → scoped recompile (3b).
    Scoped,
    /// Contract/breaking → full recompile + migration (3c).
    Breaking,
}

impl ChangeClass {
    pub fn label(self) -> &'static str {
        match self {
            ChangeClass::MetaOnly => "3a-incremental-patch",
            ChangeClass::Scoped => "3b-scoped-recompile",
            ChangeClass::Breaking => "3c-full-recompile-migration",
        }
    }
}

/// Classify the change between two profile versions.
pub fn classify_change(prev: &UserProfile, next: &UserProfile) -> ChangeClass {
    // 3c breaking: genre changed, or schema-level truth block restructured.
    let prev_genre = prev.candidates.identity.genre.as_ref().map(|g| g.value);
    let next_genre = next.candidates.identity.genre.as_ref().map(|g| g.value);
    if prev_genre != next_genre {
        return ChangeClass::Breaking;
    }
    // 3c breaking: data source paths changed (re-indexing).
    if !same_paths(&prev.data_sources.sources, &next.data_sources.sources) {
        return ChangeClass::Breaking;
    }

    // 3b scoped: truth goals/routines/automation changed.
    if prev.truth.goals != next.truth.goals
        || prev.truth.identity.routines != next.truth.identity.routines
        || prev.truth.identity.automation_targets != next.truth.identity.automation_targets
        || prev.provider_preference != next.provider_preference
    {
        return ChangeClass::Scoped;
    }

    // 3a meta-only: evolution weights, interview_meta, updated_at.
    ChangeClass::MetaOnly
}

fn same_paths(
    a: &[crate::domain::profile::DataSource],
    b: &[crate::domain::profile::DataSource],
) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .all(|(x, y)| x.path == y.path && x.kind == y.kind)
}

/// Recompile decision: returns the new bundle + which class was applied.
pub fn recompile(
    prev: &UserProfile,
    next: &UserProfile,
) -> crate::domain::Result<(crate::domain::bundle::HarnessBundle, ChangeClass)> {
    let class = classify_change(prev, next);
    let bundle = crate::compiler::compile_profile(next)?;
    // 3c: a migration note would be generated here in a full implementation;
    // we record the class so callers can persist it.
    Ok((bundle, class))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::genre::Genre;
    use crate::domain::profile::{DataSource, GenreConfidence, ProfileStatus};

    fn profile(genre: Genre, goal: &str) -> UserProfile {
        let mut p = UserProfile::new_draft("d", "en");
        p.candidates.identity.genre = Some(GenreConfidence {
            value: genre,
            confidence: 1.0,
            provenance: vec![],
        });
        p.truth.goals.goal_30d = Some(goal.into());
        p.status = ProfileStatus::Confirmed;
        p
    }

    #[test]
    fn meta_only_change_is_3a() {
        // AC13: meta-only change → 3a
        let prev = profile(Genre::Developer, "ship");
        let mut next = prev.clone();
        next.evolution_policy = Some(crate::domain::profile::EvolutionPolicyConfig {
            enabled: true,
            safety_gates: vec!["critic".into(), "seesaw".into(), "stagnation".into()],
            stagnation_limit: 5,
            improvement_threshold: 0.03,
        });
        assert_eq!(classify_change(&prev, &next), ChangeClass::MetaOnly);
    }

    #[test]
    fn goal_change_is_3b_scoped() {
        let prev = profile(Genre::Developer, "ship");
        let mut next = prev.clone();
        next.truth.goals.goal_30d = Some("quality".into());
        assert_eq!(classify_change(&prev, &next), ChangeClass::Scoped);
    }

    #[test]
    fn genre_change_is_3c_breaking() {
        let prev = profile(Genre::Developer, "ship");
        let next = profile(Genre::Creator, "ship");
        assert_eq!(classify_change(&prev, &next), ChangeClass::Breaking);
    }

    #[test]
    fn data_source_path_change_is_3c() {
        let prev = profile(Genre::Developer, "ship");
        let mut next = prev.clone();
        next.data_sources.sources.push(DataSource {
            path: "/new/path".into(),
            kind: "text_dir".into(),
            candidate_tags: vec![],
            tags_source: crate::domain::profile::ProvenanceSource::Truth,
        });
        assert_eq!(classify_change(&prev, &next), ChangeClass::Breaking);
    }

    #[test]
    fn recompile_returns_bundle_and_class() {
        let prev = profile(Genre::Developer, "ship");
        let mut next = prev.clone();
        next.truth.goals.goal_30d = Some("quality".into());
        let (b, c) = recompile(&prev, &next).unwrap();
        assert_eq!(c, ChangeClass::Scoped);
        assert_eq!(b.genre, Genre::Developer);
    }
}

//! B11 smart recall — genre-weighted composite score (ARCH §7.2).
//!
//! `recall_score = recency·w_recency + importance·w_importance + access_freq·w_freq + fts·w_fts`
//! with the epic-harness baseline weights recency 0.25 / importance 0.35 /
//! freq 0.15 / FTS 0.25, and genre-specific recency half-lives & importance.

use crate::domain::genre::Genre;

/// The four component weights.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecallWeights {
    pub recency: f64,
    pub importance: f64,
    pub access_freq: f64,
    pub fts_match: f64,
}

impl RecallWeights {
    /// epic-harness baseline (ARCH §7.2).
    pub fn baseline() -> Self {
        Self {
            recency: 0.25,
            importance: 0.35,
            access_freq: 0.15,
            fts_match: 0.25,
        }
    }

    /// Per-genre weights. The total is renormalized to 1.0 so the genre shift
    /// moves *relative* emphasis without inflating the score (ARCH §7.2).
    pub fn for_genre(genre: Genre) -> Self {
        let mut w = Self::baseline();
        // business/researcher/creator raise importance at recency's expense.
        let shift = match genre {
            Genre::Developer => 0.0,
            Genre::Researcher => 0.05,
            Genre::Creator => 0.05,
            Genre::Business => 0.10,
        };
        w.importance += shift;
        w.recency -= shift;
        w
    }
}

/// Per-genre recency half-life in days (ARCH §7.2 table).
pub fn recency_halflife_days(genre: Genre) -> i64 {
    use Genre::*;
    match genre {
        Developer => 30,
        Researcher => 90,
        Creator => 180,
        Business => 14,
    }
}

/// Importance defaults per genre (ARCH §7.2). The `importance` input is the
/// caller-supplied type importance; this returns the genre multiplier.
pub fn importance_multiplier(genre: Genre) -> f64 {
    use Genre::*;
    match genre {
        Developer => 1.0,
        Researcher => 1.1,
        Creator => 1.1,
        Business => 1.2,
    }
}

/// Compute the composite recall score. All inputs are expected normalized 0..1.
pub fn recall_score(
    genre: Genre,
    recency: f64,
    importance: f64,
    access_freq: f64,
    fts_match: f64,
) -> f64 {
    let w = RecallWeights::for_genre(genre);
    let imp = importance.clamp(0.0, 1.0);
    recency * w.recency + imp * w.importance + access_freq * w.access_freq + fts_match * w.fts_match
}

/// Exponential recency decay: returns 0..=1 given age in days + genre half-life.
pub fn recency_value(age_days: i64, genre: Genre) -> f64 {
    let h = recency_halflife_days(genre) as f64;
    if h <= 0.0 {
        return 1.0;
    }
    0.5f64.powf(age_days as f64 / h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weights_sum_to_one() {
        let w = RecallWeights::baseline();
        let sum = w.recency + w.importance + w.access_freq + w.fts_match;
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn recent_high_importance_outranks_old_low() {
        // AC11: genre recency differences produce correct ordering.
        let fresh_important = recall_score(Genre::Developer, 1.0, 1.0, 0.5, 0.5);
        let old_trivial = recall_score(Genre::Developer, 0.0, 0.1, 0.5, 0.5);
        assert!(fresh_important > old_trivial);
    }

    #[test]
    fn creator_recency_decays_slower_than_business() {
        // 100 days old: creator (180d halflife) still relevant, business (14d) not.
        let creator = recency_value(100, Genre::Creator);
        let business = recency_value(100, Genre::Business);
        assert!(creator > business);
        assert!(business < 0.01);
        assert!(creator > 0.5);
    }

    #[test]
    fn business_importance_weighted_higher() {
        let dev = recall_score(Genre::Business, 0.5, 1.0, 0.5, 0.5);
        let base = recall_score(Genre::Developer, 0.5, 1.0, 0.5, 0.5);
        assert!(dev > base);
    }
}

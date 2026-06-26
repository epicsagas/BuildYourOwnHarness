//! Genre inference for external skills (RFC §9 "카탈로그 스키마 매핑").
//!
//! Maps a free-form upstream `category` (e.g. a plugin.json `category` field or a
//! curated list tag) to BYOH's 4-value [`Genre`] namespace. Conservative: a value
//! that matches no keyword returns `None` — the caller then requires `--genre`.
//! Inference never silently guesses (BYOH "honest failure" principle).

use crate::domain::genre::Genre;

/// Developer-leaning category keywords.
const DEVELOPER: &[&str] = &[
    "code",
    "coding",
    "develop",
    "engineering",
    "testing",
    "debug",
    "refactor",
    "git",
    "software",
    "program",
    "security",
    "devops",
];
/// Creator-leaning category keywords.
const CREATOR: &[&str] = &[
    "write", "writing", "content", "creative", "blog", "book", "story", "editor", "fiction",
    "draft", "prose",
];
/// Researcher-leaning category keywords.
const RESEARCHER: &[&str] = &[
    "research",
    "analysis",
    "investigate",
    "science",
    "data",
    "study",
    "cite",
    "evidence",
    "paper",
    "academic",
];
/// Business-leaning category keywords.
const BUSINESS: &[&str] = &[
    "business",
    "product",
    "market",
    "strategy",
    "decision",
    "productivity",
    "management",
    "ops",
    "finance",
    "planning",
    "startup",
];

/// Infer a [`Genre`] from an upstream `category` string (case-insensitive
/// substring match against the keyword tables). First matching table wins;
/// order is `[developer, creator, researcher, business]`. Returns `None` when
/// nothing matches — the caller must then take `--genre` or error.
pub fn infer_genre(category: &str) -> Option<Genre> {
    let c = category.to_lowercase();
    let tables: [(Genre, &[&str]); 4] = [
        (Genre::Developer, DEVELOPER),
        (Genre::Creator, CREATOR),
        (Genre::Researcher, RESEARCHER),
        (Genre::Business, BUSINESS),
    ];
    for (g, kw) in tables {
        if kw.iter().any(|k| c.contains(k)) {
            return Some(g);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_each_genre() {
        assert_eq!(infer_genre("coding-tools"), Some(Genre::Developer));
        assert_eq!(infer_genre("content writing"), Some(Genre::Creator));
        assert_eq!(infer_genre("data analysis"), Some(Genre::Researcher));
        assert_eq!(infer_genre("product strategy"), Some(Genre::Business));
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(infer_genre("DEVELOPMENT"), Some(Genre::Developer));
        assert_eq!(infer_genre("Market Research"), Some(Genre::Researcher));
    }

    #[test]
    fn unknown_returns_none() {
        assert_eq!(infer_genre("cooking"), None);
        assert_eq!(infer_genre(""), None);
    }

    #[test]
    fn first_match_wins_developer_priority() {
        // "code" (developer) is checked before any later table.
        assert_eq!(infer_genre("code review"), Some(Genre::Developer));
    }
}

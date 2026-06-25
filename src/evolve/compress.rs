//! B13 adaptive compression — budget-driven 4-tier compression (ARCH §8.1).

use crate::domain::genre::Genre;

/// Budget-utilization tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionTier {
    /// < 60% — keep everything.
    StopwordOnly,
    /// < 80% — drop low-importance tokens.
    PruneLowImportance,
    /// < 95% — dedupe + linearize.
    DeduplicateAndLinearize,
    /// ≥ 95% — keep only essentials.
    MaxCompression,
}

impl CompressionTier {
    pub fn from_budget_usage(usage: f64) -> Self {
        if usage < 0.60 {
            CompressionTier::StopwordOnly
        } else if usage < 0.80 {
            CompressionTier::PruneLowImportance
        } else if usage < 0.95 {
            CompressionTier::DeduplicateAndLinearize
        } else {
            CompressionTier::MaxCompression
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CompressionTier::StopwordOnly => "stopword-only",
            CompressionTier::PruneLowImportance => "prune-low-importance",
            CompressionTier::DeduplicateAndLinearize => "dedupe-linearize",
            CompressionTier::MaxCompression => "max-compression",
        }
    }
}

/// Genre-specific token importance weights (ARCH §8.1).
#[derive(Debug, Clone, PartialEq)]
pub struct ImportanceWeights {
    pub code: f64,
    pub comment: f64,
    pub dialogue: f64,
    pub description: f64,
    pub citation: f64,
    pub number: f64,
}

impl ImportanceWeights {
    pub fn for_genre(genre: Genre) -> Self {
        use Genre::*;
        match genre {
            Developer => Self {
                code: 0.9,
                comment: 0.2,
                dialogue: 0.5,
                description: 0.5,
                citation: 0.5,
                number: 0.5,
            },
            Creator => Self {
                code: 0.1,
                comment: 0.1,
                dialogue: 0.95,
                description: 0.6,
                citation: 0.4,
                number: 0.3,
            },
            Researcher => Self {
                code: 0.4,
                comment: 0.2,
                dialogue: 0.4,
                description: 0.4,
                citation: 0.95,
                number: 0.8,
            },
            Business => Self {
                code: 0.3,
                comment: 0.2,
                dialogue: 0.4,
                description: 0.3,
                citation: 0.5,
                number: 0.95,
            },
        }
    }
}

/// A token with a classified kind + genre-weighted importance.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub text: String,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Code,
    Comment,
    Dialogue,
    Description,
    Citation,
    Number,
}

impl Token {
    pub fn importance(&self, weights: &ImportanceWeights) -> f64 {
        use TokenKind::*;
        match self.kind {
            Code => weights.code,
            Comment => weights.comment,
            Dialogue => weights.dialogue,
            Description => weights.description,
            Citation => weights.citation,
            Number => weights.number,
        }
    }
}

/// Compress a token stream under the given budget tier + genre weights.
pub fn compress(tokens: &[Token], tier: CompressionTier, genre: Genre) -> Vec<Token> {
    let weights = ImportanceWeights::for_genre(genre);
    match tier {
        CompressionTier::StopwordOnly => tokens.to_vec(),
        CompressionTier::PruneLowImportance => tokens
            .iter()
            .filter(|t| t.importance(&weights) >= 0.5)
            .cloned()
            .collect(),
        CompressionTier::DeduplicateAndLinearize => {
            let mut seen = std::collections::HashSet::new();
            tokens
                .iter()
                .filter(|t| t.importance(&weights) >= 0.5)
                .filter(|t| seen.insert(t.text.clone()))
                .cloned()
                .collect()
        }
        CompressionTier::MaxCompression => tokens
            .iter()
            .filter(|t| t.importance(&weights) >= 0.9)
            .cloned()
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<Token> {
        vec![
            Token {
                text: "fn main".into(),
                kind: TokenKind::Code,
            },
            Token {
                text: "// todo".into(),
                kind: TokenKind::Comment,
            },
            Token {
                text: "she said".into(),
                kind: TokenKind::Dialogue,
            },
            Token {
                text: "the room".into(),
                kind: TokenKind::Description,
            },
            Token {
                text: "(Smith 2020)".into(),
                kind: TokenKind::Citation,
            },
            Token {
                text: "42".into(),
                kind: TokenKind::Number,
            },
        ]
    }

    #[test]
    fn tiers_from_budget() {
        assert_eq!(
            CompressionTier::from_budget_usage(0.5),
            CompressionTier::StopwordOnly
        );
        assert_eq!(
            CompressionTier::from_budget_usage(0.7),
            CompressionTier::PruneLowImportance
        );
        assert_eq!(
            CompressionTier::from_budget_usage(0.9),
            CompressionTier::DeduplicateAndLinearize
        );
        assert_eq!(
            CompressionTier::from_budget_usage(0.97),
            CompressionTier::MaxCompression
        );
    }

    #[test]
    fn developer_max_compression_keeps_code_only() {
        let out = compress(&sample(), CompressionTier::MaxCompression, Genre::Developer);
        assert!(out.iter().all(|t| t.kind == TokenKind::Code));
        assert!(out.iter().any(|t| t.text == "fn main"));
    }

    #[test]
    fn creator_keeps_dialogue_at_prune() {
        let out = compress(
            &sample(),
            CompressionTier::PruneLowImportance,
            Genre::Creator,
        );
        assert!(out.iter().any(|t| t.kind == TokenKind::Dialogue));
        // comment dropped (importance 0.1 < 0.5)
        assert!(!out.iter().any(|t| t.kind == TokenKind::Comment));
    }

    #[test]
    fn dedupe_drops_duplicates() {
        let mut t = sample();
        t.push(Token {
            text: "fn main".into(),
            kind: TokenKind::Code,
        });
        let out = compress(
            &t,
            CompressionTier::DeduplicateAndLinearize,
            Genre::Developer,
        );
        let count = out.iter().filter(|x| x.text == "fn main").count();
        assert_eq!(count, 1);
    }
}

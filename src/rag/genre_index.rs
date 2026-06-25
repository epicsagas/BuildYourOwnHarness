//! Genre-specific index catalog + BM25 weights (ARCH §8.2).
//!
//! Each genre gets its own vector index (separate `TurbovecIndex`/store) and
//! BM25 field weights. The catalog maps a genre to its index path + weights.

use serde::{Deserialize, Serialize};

use crate::domain::genre::Genre;

/// BM25 field weights per genre (ARCH §8.2 table).
/// We model title/filename/body as integer-ish weights for ranking emphasis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenreWeights {
    pub body_weight: u32,
    pub title_weight: u32,
    pub filename_weight: u32,
    /// CJK tokenizer selector: 0 = ngram, 1 = morph (creator recommended).
    pub cjk_tokenizer: u8,
}

impl GenreWeights {
    /// ARCH §8.2 table values.
    pub fn for_genre(genre: Genre) -> Self {
        use Genre::*;
        match genre {
            Developer => Self {
                body_weight: 10,
                title_weight: 30,
                filename_weight: 20,
                cjk_tokenizer: 0,
            },
            Researcher => Self {
                body_weight: 12,
                title_weight: 25,
                filename_weight: 15,
                cjk_tokenizer: 0,
            },
            Creator => Self {
                body_weight: 10,
                title_weight: 20,
                filename_weight: 10,
                cjk_tokenizer: 1, // morphological recommended
            },
            Business => Self {
                body_weight: 10,
                title_weight: 35,
                filename_weight: 20,
                cjk_tokenizer: 0,
            },
        }
    }
}

/// Convenience accessor (used by `bm25_search`).
pub fn genre_bm25_weights(genre: Genre) -> GenreWeights {
    GenreWeights::for_genre(genre)
}

/// Per-genre index path under a root dir.
pub fn genre_index_path(root: &std::path::Path, genre: Genre) -> std::path::PathBuf {
    root.join("indexes").join(format!("{}.tv", genre.as_str()))
}

/// Catalog of all genre indexes + their weights.
#[derive(Debug, Clone)]
pub struct GenreIndexCatalog {
    pub root: std::path::PathBuf,
    pub weights: Vec<(Genre, GenreWeights)>,
}

impl GenreIndexCatalog {
    pub fn new(root: impl AsRef<std::path::Path>) -> Self {
        let weights = Genre::all()
            .iter()
            .copied()
            .map(|g| (g, GenreWeights::for_genre(g)))
            .collect();
        Self {
            root: root.as_ref().to_path_buf(),
            weights,
        }
    }

    pub fn path_for(&self, genre: Genre) -> std::path::PathBuf {
        genre_index_path(&self.root, genre)
    }

    pub fn weights_for(&self, genre: Genre) -> GenreWeights {
        GenreWeights::for_genre(genre)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_genre_has_weights() {
        for &g in Genre::all() {
            let w = GenreWeights::for_genre(g);
            assert!(w.body_weight > 0);
        }
    }

    #[test]
    fn business_title_weight_higher_than_creator() {
        // ARCH §8.2: business title=3.5 vs creator title=2.0
        assert!(
            GenreWeights::for_genre(Genre::Business).title_weight
                > GenreWeights::for_genre(Genre::Creator).title_weight
        );
    }

    #[test]
    fn creator_recommends_morph_tokenizer() {
        assert_eq!(GenreWeights::for_genre(Genre::Creator).cjk_tokenizer, 1);
    }

    #[test]
    fn catalog_paths_per_genre() {
        let cat = GenreIndexCatalog::new("/tmp/byoh");
        for &g in Genre::all() {
            let p = cat.path_for(g);
            assert!(p.to_string_lossy().contains(g.as_str()));
        }
    }
}

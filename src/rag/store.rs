//! Vector stores — persistence + ANN search.
//!
//! [`InMemoryStore`] is the always-available brute-force cosine backend.
//! [`TurbovecStore`] (behind `native-rag`) wraps `llm_kernel::TurbovecIndex`
//! for quantized ANN + file persistence.

use std::path::Path;

use crate::ports::embedder::Embedding;

/// A stored vector keyed by chunk id, with its payload text for hydration.
#[derive(Debug, Clone)]
pub struct StoredVector {
    pub id: String,
    pub vector: Vec<f32>,
    pub text: String,
}

/// Search result row.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorHit {
    pub id: String,
    pub text: String,
    pub score: f32,
}

/// Vector store port. Implementations: in-memory (default) or TurbovecIndex.
pub trait VectorStore: Send + Sync {
    fn add(&mut self, id: &str, embedding: &Embedding, text: &str) -> crate::domain::Result<()>;

    /// k-NN search; returns up to `k` hits sorted by descending score.
    fn search(&self, query: &Embedding, k: usize) -> crate::domain::Result<Vec<VectorHit>>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Persist to disk (optional; no-op for in-memory).
    fn save(&self, _path: &Path) -> crate::domain::Result<()> {
        Ok(())
    }

    /// Load from disk (optional).
    fn load(&mut self, _path: &Path) -> crate::domain::Result<()> {
        Ok(())
    }

    fn backend(&self) -> &'static str;
}

// ──────────────────────────────────────────────────────────────────────────
// In-memory brute-force cosine — always available
// ──────────────────────────────────────────────────────────────────────────

/// Brute-force cosine similarity store. Sufficient for small/medium corpora
/// and tests; no external dependencies.
#[derive(Debug, Default, Clone)]
pub struct InMemoryStore {
    dim: usize,
    rows: Vec<StoredVector>,
}

impl InMemoryStore {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            rows: Vec::new(),
        }
    }

    /// The `(id, text)` pairs of every stored vector — used to reconstruct the
    /// hybrid-search corpus when loading a persisted index without a sidecar.
    pub fn corpus_pairs(&self) -> Vec<(String, String)> {
        self.rows
            .iter()
            .map(|r| (r.id.clone(), r.text.clone()))
            .collect()
    }
}

impl VectorStore for InMemoryStore {
    fn add(&mut self, id: &str, embedding: &Embedding, text: &str) -> crate::domain::Result<()> {
        if self.dim == 0 {
            self.dim = embedding.vector.len();
        }
        if embedding.vector.len() != self.dim {
            return Err(crate::domain::ByohError::Schema(format!(
                "embedding dim {} != store dim {}",
                embedding.vector.len(),
                self.dim
            )));
        }
        self.rows.push(StoredVector {
            id: id.to_string(),
            vector: embedding.vector.clone(),
            text: text.to_string(),
        });
        Ok(())
    }

    fn search(&self, query: &Embedding, k: usize) -> crate::domain::Result<Vec<VectorHit>> {
        if self.rows.is_empty() {
            return Ok(Vec::new());
        }
        let mut scored: Vec<VectorHit> = self
            .rows
            .iter()
            .map(|r| VectorHit {
                id: r.id.clone(),
                text: r.text.clone(),
                score: cosine(&query.vector, &r.vector),
            })
            .collect();
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(k);
        Ok(scored)
    }

    fn len(&self) -> usize {
        self.rows.len()
    }

    fn save(&self, path: &Path) -> crate::domain::Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let payload: Vec<serde_json::Value> = self
            .rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "text": r.text,
                    "vector": r.vector,
                })
            })
            .collect();
        let body = serde_json::json!({ "dim": self.dim, "rows": payload });
        std::fs::write(path, serde_json::to_vec_pretty(&body)?)?;
        Ok(())
    }

    fn load(&mut self, path: &Path) -> crate::domain::Result<()> {
        let body = std::fs::read_to_string(path)?;
        let v: serde_json::Value = serde_json::from_str(&body)?;
        self.dim = v
            .get("dim")
            .and_then(|d| d.as_u64())
            .map(|d| d as usize)
            .unwrap_or(0);
        self.rows = v
            .get("rows")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|row| {
                        let id = row.get("id")?.as_str()?.to_string();
                        let text = row.get("text")?.as_str()?.to_string();
                        let vector: Vec<f32> = row
                            .get("vector")?
                            .as_array()?
                            .iter()
                            .filter_map(|x| x.as_f64().map(|f| f as f32))
                            .collect();
                        Some(StoredVector { id, vector, text })
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(())
    }

    fn backend(&self) -> &'static str {
        "in-memory-cosine"
    }
}

/// Cosine similarity for two equal-length vectors.
pub(crate) fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
    dot / (na * nb)
}

// ──────────────────────────────────────────────────────────────────────────
// native-rag: TurbovecIndex backend
// ──────────────────────────────────────────────────────────────────────────

#[cfg(feature = "native-rag")]
pub mod turbovec {
    //! Quantized ANN index backed by `llm_kernel::embedding::TurbovecIndex`.

    use std::path::Path;

    // VectorIndex trait is required to call add_with_ids / search / save on the
    // concrete TurbovecIndex.
    use llm_kernel::embedding::VectorIndex;

    use crate::ports::embedder::Embedding;
    use crate::rag::store::{cosine, VectorHit, VectorStore};

    /// Production vector store wrapping llm-kernel's `TurbovecIndex`.
    ///
    /// We keep an in-memory sidecar of chunk ids + text (llm-kernel stores only
    /// the quantized vectors) so search results can hydrate payload text.
    pub struct TurbovecStore {
        index: llm_kernel::embedding::TurbovecIndex,
        dim: usize,
        ids: Vec<String>,
        texts: Vec<String>,
        raw: Vec<Vec<f32>>, // full-precision fallback for cosine re-rank
    }

    impl TurbovecStore {
        pub fn new(dim: usize, bit_width: u8) -> crate::domain::Result<Self> {
            let index = llm_kernel::embedding::TurbovecIndex::new(dim, bit_width)
                .map_err(|e| crate::domain::ByohError::Other(format!("turbovec new: {e}")))?;
            Ok(Self {
                index,
                dim,
                ids: Vec::new(),
                texts: Vec::new(),
                raw: Vec::new(),
            })
        }
    }

    impl VectorStore for TurbovecStore {
        fn add(
            &mut self,
            id: &str,
            embedding: &Embedding,
            text: &str,
        ) -> crate::domain::Result<()> {
            if embedding.vector.len() != self.dim {
                return Err(crate::domain::ByohError::Schema(format!(
                    "embedding dim {} != store dim {}",
                    embedding.vector.len(),
                    self.dim
                )));
            }
            // llm-kernel upsert + id tracking. API: add_with_ids / upsert.
            let local_id = self.ids.len();
            self.index
                .add_with_ids(std::slice::from_ref(&embedding.vector), &[local_id as u64])
                .map_err(|e| crate::domain::ByohError::Other(format!("turbovec add: {e}")))?;
            self.ids.push(id.to_string());
            self.texts.push(text.to_string());
            self.raw.push(embedding.vector.clone());
            Ok(())
        }

        fn search(&self, query: &Embedding, k: usize) -> crate::domain::Result<Vec<VectorHit>> {
            if self.ids.is_empty() {
                return Ok(Vec::new());
            }
            // Use the quantized index for ANN, then re-rank by exact cosine.
            let ann = self
                .index
                .search(&query.vector, k)
                .map_err(|e| crate::domain::ByohError::Other(format!("turbovec search: {e}")))?;
            let mut hits: Vec<VectorHit> = ann
                .into_iter()
                .filter_map(|hit| {
                    let local = hit.id as usize;
                    let raw = self.raw.get(local)?;
                    Some(VectorHit {
                        id: self.ids.get(local)?.clone(),
                        text: self.texts.get(local)?.clone(),
                        score: cosine(&query.vector, raw),
                    })
                })
                .collect();
            hits.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            hits.truncate(k);
            Ok(hits)
        }

        fn len(&self) -> usize {
            self.ids.len()
        }

        fn save(&self, path: &Path) -> crate::domain::Result<()> {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            self.index
                .save(path)
                .map_err(|e| crate::domain::ByohError::Other(format!("turbovec save: {e}")))?;
            // Sidecar for ids/texts.
            let sidecar = path.with_extension("meta.json");
            let payload = serde_json::json!({
                "ids": self.ids,
                "texts": self.texts,
                "dim": self.dim,
            });
            std::fs::write(&sidecar, serde_json::to_vec_pretty(&payload)?)?;
            Ok(())
        }

        fn load(&mut self, path: &Path) -> crate::domain::Result<()> {
            self.index = llm_kernel::embedding::TurbovecIndex::load(path)
                .map_err(|e| crate::domain::ByohError::Other(format!("turbovec load: {e}")))?;
            let sidecar = path.with_extension("meta.json");
            let body = std::fs::read_to_string(&sidecar)?;
            let v: serde_json::Value = serde_json::from_str(&body)?;
            self.ids = v
                .get("ids")
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            self.texts = v
                .get("texts")
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            Ok(())
        }

        fn backend(&self) -> &'static str {
            "turbovec"
        }
    }
}

#[cfg(feature = "native-rag")]
pub use turbovec::TurbovecStore;

#[cfg(test)]
mod tests {
    use super::*;

    fn emb(v: Vec<f32>) -> Embedding {
        Embedding { vector: v }
    }

    #[test]
    fn inmemory_search_ranks_by_cosine() {
        let mut s = InMemoryStore::new(3);
        s.add("a", &emb(vec![1.0, 0.0, 0.0]), "alpha").unwrap();
        s.add("b", &emb(vec![0.0, 1.0, 0.0]), "beta").unwrap();
        s.add("c", &emb(vec![0.9, 0.1, 0.0]), "gamma").unwrap();

        let hits = s.search(&emb(vec![1.0, 0.0, 0.0]), 2).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].id, "a");
        assert!(hits[0].score > 0.99);
    }

    #[test]
    fn inmemory_save_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("idx.json");
        let mut s = InMemoryStore::new(3);
        s.add("a", &emb(vec![1.0, 0.0, 0.0]), "alpha").unwrap();
        s.save(&path).unwrap();

        let mut loaded = InMemoryStore::new(3);
        loaded.load(&path).unwrap();
        assert_eq!(loaded.len(), 1);
        let hits = loaded.search(&emb(vec![1.0, 0.0, 0.0]), 1).unwrap();
        assert_eq!(hits[0].text, "alpha");
    }

    #[test]
    fn dim_mismatch_rejected() {
        let mut s = InMemoryStore::new(3);
        let err = s.add("x", &emb(vec![1.0, 0.0]), "x");
        assert!(err.is_err());
    }

    #[test]
    fn cosine_basic() {
        assert!(cosine(&[1.0, 0.0], &[1.0, 0.0]) > 0.99);
        assert!(cosine(&[1.0, 0.0], &[0.0, 1.0]).abs() < 1e-6);
    }
}

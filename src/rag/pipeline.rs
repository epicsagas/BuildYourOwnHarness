//! RAG pipeline — the top-level orchestration: chunk → embed → index → search.
//!
//! [`build_index`] ingests documents into a genre's vector store; [`IndexHandle`]
//! wraps a built (or loaded) store + the chunk corpus for hybrid fallback.

use std::path::Path;

use crate::domain::genre::Genre;
use crate::ports::embedder::EmbedderProvider;
use crate::rag::chunk::{chunk_document, ChunkOptions};
use crate::rag::genre_index::genre_index_path;
use crate::rag::search::{hybrid_search, HybridHit};
use crate::rag::store::{InMemoryStore, VectorStore};

/// A document to index.
#[derive(Debug, Clone)]
pub struct InputDoc {
    pub id: String,
    pub text: String,
}

/// Stats from a build run.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildReport {
    pub genre: Genre,
    pub docs: usize,
    pub chunks: usize,
    pub dim: usize,
    pub backend: String,
}

/// A built index — owns the store + the (id, text) corpus for BM25/grep fallback.
pub struct IndexHandle<S: VectorStore> {
    pub genre: Genre,
    pub store: S,
    /// (chunk_id, text) corpus mirroring stored vectors, for BM25/grep tiers.
    pub corpus: Vec<(String, String)>,
    pub backend: String,
}

impl<S: VectorStore> IndexHandle<S> {
    /// Hybrid search: vector → BM25 → grep. `query_embedding` optional (None ⇒
    /// skip vector tier, go straight to BM25/grep).
    pub fn search(
        &self,
        embedder: &dyn EmbedderProvider,
        query: &str,
        k: usize,
    ) -> crate::domain::Result<Vec<SearchHit>> {
        let qe = embedder.embed(query)?;
        let hits = hybrid_search(
            Some(&self.store),
            Some(&qe),
            &self.corpus,
            query,
            k,
            self.genre,
        );
        Ok(hits.into_iter().map(Into::into).collect())
    }
}

/// A search hit returned to callers.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHit {
    pub id: String,
    pub text: String,
    pub score: f32,
    pub mode: &'static str,
}

impl From<HybridHit> for SearchHit {
    fn from(h: HybridHit) -> Self {
        Self {
            id: h.id,
            text: h.text,
            score: h.score,
            mode: h.mode.as_str(),
        }
    }
}

/// Build a genre index from documents into an in-memory store (default backend).
pub fn build_index(
    embedder: &dyn EmbedderProvider,
    genre: Genre,
    docs: &[InputDoc],
    opts: &ChunkOptions,
) -> crate::domain::Result<(BuildReport, IndexHandle<InMemoryStore>)> {
    let dim = embedder.dim();
    let mut store = InMemoryStore::new(dim);
    let mut corpus: Vec<(String, String)> = Vec::new();
    let mut total_chunks = 0usize;

    for doc in docs {
        let chunks = chunk_document(&doc.id, &doc.text, opts);
        for chunk in &chunks {
            let embedding = embedder.embed(&chunk.text)?;
            store.add(&chunk.id, &embedding, &chunk.text)?;
            corpus.push((chunk.id.clone(), chunk.text.clone()));
            total_chunks += 1;
        }
    }

    let report = BuildReport {
        genre,
        docs: docs.len(),
        chunks: total_chunks,
        dim,
        backend: store.backend().to_string(),
    };
    let backend = report.backend.clone();
    Ok((
        report,
        IndexHandle {
            genre,
            store,
            corpus,
            backend,
        },
    ))
}

/// Persist an index to disk under `root` keyed by genre.
pub fn save_index<S: VectorStore>(
    handle: &IndexHandle<S>,
    root: &Path,
) -> crate::domain::Result<()> {
    let path = genre_index_path(root, handle.genre);
    handle.store.save(&path)
}

#[cfg(feature = "native-rag")]
pub mod native {
    //! native-rag pipeline: uses TurbovecStore (quantized ANN) + persistence.

    use std::path::Path;

    use crate::domain::genre::Genre;
    use crate::ports::embedder::EmbedderProvider;
    use crate::rag::chunk::ChunkOptions;
    use crate::rag::genre_index::genre_index_path;
    use crate::rag::pipeline::{BuildReport, IndexHandle, InputDoc};
    use crate::rag::store::{TurbovecStore, VectorStore};

    /// Bit width for TurbovecIndex quantization (default 4).
    pub const DEFAULT_BIT_WIDTH: u8 = 4;

    /// Build a genre index backed by TurbovecIndex (quantized ANN).
    pub fn build_index_native(
        embedder: &dyn EmbedderProvider,
        genre: Genre,
        docs: &[InputDoc],
        opts: &ChunkOptions,
        bit_width: u8,
    ) -> crate::domain::Result<(BuildReport, IndexHandle<TurbovecStore>)> {
        let dim = embedder.dim();
        let mut store = TurbovecStore::new(dim, bit_width)?;
        let mut corpus: Vec<(String, String)> = Vec::new();
        let mut total_chunks = 0usize;

        for doc in docs {
            let chunks = crate::rag::chunk::chunk_document(&doc.id, &doc.text, opts);
            for chunk in &chunks {
                let embedding = embedder.embed(&chunk.text)?;
                store.add(&chunk.id, &embedding, &chunk.text)?;
                corpus.push((chunk.id.clone(), chunk.text.clone()));
                total_chunks += 1;
            }
        }

        let report = BuildReport {
            genre,
            docs: docs.len(),
            chunks: total_chunks,
            dim,
            backend: store.backend().to_string(),
        };
        let backend = report.backend.clone();
        Ok((
            report,
            IndexHandle {
                genre,
                store,
                corpus,
                backend,
            },
        ))
    }

    /// Save a native index.
    pub fn save_index_native(
        handle: &IndexHandle<TurbovecStore>,
        root: &Path,
    ) -> crate::domain::Result<()> {
        let path = genre_index_path(root, handle.genre);
        handle.store.save(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::DummyEmbedder;

    #[test]
    fn build_then_search_roundtrip() {
        let emb = DummyEmbedder::new();
        let docs = vec![
            InputDoc {
                id: "d1".into(),
                text: "rust async runtime tokio".into(),
            },
            InputDoc {
                id: "d2".into(),
                text: "python data science numpy".into(),
            },
        ];
        let (report, handle) =
            build_index(&emb, Genre::Developer, &docs, &ChunkOptions::default()).unwrap();
        assert_eq!(report.docs, 2);
        assert!(report.chunks >= 2);

        let hits = handle.search(&emb, "rust", 5).unwrap();
        assert!(!hits.is_empty());
    }

    #[test]
    fn empty_docs_builds_empty_index() {
        let emb = DummyEmbedder::new();
        let (report, _handle) =
            build_index(&emb, Genre::Creator, &[], &ChunkOptions::default()).unwrap();
        assert_eq!(report.chunks, 0);
    }
}

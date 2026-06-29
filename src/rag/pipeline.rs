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

/// Sidecar path holding the `(chunk_id, text)` corpus next to a genre index.
/// The corpus is what powers the BM25/grep fallback tiers after a load — the
/// vector store alone (esp. quantized native stores) may not retain raw text.
pub fn corpus_sidecar_path(root: &Path, genre: Genre) -> std::path::PathBuf {
    root.join("indexes")
        .join(format!("{}.corpus.json", genre.as_str()))
}

/// Persist an index to disk under `root` keyed by genre: the vector store plus
/// a corpus sidecar so [`load_index`] can fully reconstruct the [`IndexHandle`].
pub fn save_index<S: VectorStore>(
    handle: &IndexHandle<S>,
    root: &Path,
) -> crate::domain::Result<()> {
    let path = genre_index_path(root, handle.genre);
    handle.store.save(&path)?;
    save_corpus_sidecar(root, handle.genre, &handle.corpus)
}

/// Write the corpus sidecar (`<genre>.corpus.json`).
pub(crate) fn save_corpus_sidecar(
    root: &Path,
    genre: Genre,
    corpus: &[(String, String)],
) -> crate::domain::Result<()> {
    let path = corpus_sidecar_path(root, genre);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let payload: Vec<serde_json::Value> = corpus
        .iter()
        .map(|(id, text)| serde_json::json!({ "id": id, "text": text }))
        .collect();
    std::fs::write(&path, serde_json::to_vec_pretty(&payload)?)?;
    Ok(())
}

/// Read the corpus sidecar, if present.
pub(crate) fn load_corpus_sidecar(
    root: &Path,
    genre: Genre,
) -> crate::domain::Result<Vec<(String, String)>> {
    let path = corpus_sidecar_path(root, genre);
    let body = std::fs::read_to_string(&path)?;
    let v: serde_json::Value = serde_json::from_str(&body)?;
    Ok(v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|row| {
                    let id = row.get("id")?.as_str()?.to_string();
                    let text = row.get("text")?.as_str()?.to_string();
                    Some((id, text))
                })
                .collect()
        })
        .unwrap_or_default())
}

/// Load a previously-saved in-memory genre index from `root`.
///
/// Returns `Ok(None)` when no index has been persisted for this genre (so the
/// caller can fall back to building ephemerally or to the grep tier). The
/// `(store, corpus)` pair is restored so all three search tiers (vector → BM25
/// → grep) work without re-embedding.
pub fn load_index(
    root: &Path,
    genre: Genre,
) -> crate::domain::Result<Option<IndexHandle<InMemoryStore>>> {
    let store_path = genre_index_path(root, genre);
    if !store_path.exists() {
        return Ok(None);
    }
    let mut store = InMemoryStore::new(0);
    store.load(&store_path)?;
    // Prefer the explicit sidecar; fall back to reconstructing corpus from the
    // store rows (InMemoryStore retains id+text) if the sidecar is absent.
    let corpus = match load_corpus_sidecar(root, genre) {
        Ok(c) if !c.is_empty() => c,
        _ => store.corpus_pairs(),
    };
    let backend = store.backend().to_string();
    Ok(Some(IndexHandle {
        genre,
        store,
        corpus,
        backend,
    }))
}

/// Incrementally (re)build + persist a genre index: re-embed only docs that are
/// new or changed since the last index, drop removed docs, keep unchanged docs
/// as-is. Falls back to a full build when no prior index/manifest exists.
///
/// Returns the build report and the [`IndexDelta`] describing what changed.
/// Saves the store + corpus sidecar + manifest. The result is byte-for-byte
/// equivalent to a full `build_index` for the same doc set (deterministic
/// embedder), but skips re-embedding unchanged docs.
pub fn build_index_incremental(
    embedder: &dyn EmbedderProvider,
    root: &Path,
    genre: Genre,
    docs: &[InputDoc],
    opts: &ChunkOptions,
) -> crate::domain::Result<(BuildReport, crate::rag::manifest::IndexDelta)> {
    use crate::rag::manifest::{diff, IndexManifest};
    use std::collections::BTreeMap;

    let manifest = IndexManifest::load(root, genre)?;
    let delta = diff(manifest.as_ref().unwrap_or(&IndexManifest::default()), docs);

    // No prior index (or manifest) → full build.
    let prior = load_index(root, genre)?;
    let (mut store, mut corpus): (InMemoryStore, Vec<(String, String)>) = match prior {
        Some(h) if manifest.is_some() => (h.store, h.corpus),
        _ => (InMemoryStore::new(embedder.dim()), Vec::new()),
    };

    let to_refresh = delta.to_embed(); // added ∪ changed
    let drop_ids: Vec<&String> = delta.removed.iter().chain(delta.changed.iter()).collect();

    // Drop removed + changed docs from store + corpus.
    for id in &drop_ids {
        store.remove_doc(id);
    }
    let dropped: std::collections::BTreeSet<String> =
        drop_ids.iter().map(|s| (*s).clone()).collect();
    // Keep chunks whose doc-id (prefix before "::") is not in the dropped set.
    corpus.retain(|(cid, _)| {
        let doc_id = cid.split("::").next().unwrap_or(cid.as_str());
        !dropped.contains(doc_id)
    });

    // Embed added + changed docs.
    let refresh_set: std::collections::BTreeSet<&str> =
        to_refresh.iter().map(|s| s.as_str()).collect();
    let mut chunk_counts: BTreeMap<String, usize> = BTreeMap::new();
    // seed counts for unchanged docs from the prior manifest
    if let Some(m) = &manifest {
        for (id, e) in &m.entries {
            if delta.unchanged.contains(id) {
                chunk_counts.insert(id.clone(), e.chunks);
            }
        }
    }
    for doc in docs {
        if !refresh_set.contains(doc.id.as_str()) {
            continue;
        }
        let chunks = chunk_document(&doc.id, &doc.text, opts);
        for chunk in &chunks {
            let embedding = embedder.embed(&chunk.text)?;
            store.add(&chunk.id, &embedding, &chunk.text)?;
            corpus.push((chunk.id.clone(), chunk.text.clone()));
        }
        chunk_counts.insert(doc.id.clone(), chunks.len());
    }

    let handle = IndexHandle {
        genre,
        store,
        corpus,
        backend: embedder.backend().to_string(),
    };
    save_index(&handle, root)?;
    IndexManifest::from_docs(docs, &chunk_counts).save(root, genre)?;

    let report = BuildReport {
        genre,
        docs: docs.len(),
        chunks: handle.store.len(),
        dim: embedder.dim(),
        backend: handle.backend.clone(),
    };
    Ok((report, delta))
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

    /// Save a native index (vector store + corpus sidecar).
    pub fn save_index_native(
        handle: &IndexHandle<TurbovecStore>,
        root: &Path,
    ) -> crate::domain::Result<()> {
        let path = genre_index_path(root, handle.genre);
        handle.store.save(&path)?;
        crate::rag::pipeline::save_corpus_sidecar(root, handle.genre, &handle.corpus)
    }

    /// Load a persisted native index, or `Ok(None)` if none exists for `genre`.
    /// `dim`/`bit_width` seed the store before `load` repopulates it.
    pub fn load_index_native(
        root: &Path,
        genre: Genre,
        dim: usize,
        bit_width: u8,
    ) -> crate::domain::Result<Option<IndexHandle<TurbovecStore>>> {
        let store_path = genre_index_path(root, genre);
        if !store_path.exists() {
            return Ok(None);
        }
        let mut store = TurbovecStore::new(dim, bit_width)?;
        store.load(&store_path)?;
        let corpus = crate::rag::pipeline::load_corpus_sidecar(root, genre).unwrap_or_default();
        let backend = store.backend().to_string();
        Ok(Some(IndexHandle {
            genre,
            store,
            corpus,
            backend,
        }))
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

    fn sample_docs() -> Vec<InputDoc> {
        vec![
            InputDoc {
                id: "d1".into(),
                text: "rust async runtime tokio".into(),
            },
            InputDoc {
                id: "d2".into(),
                text: "python data science pandas".into(),
            },
        ]
    }

    #[test]
    fn save_writes_corpus_sidecar() {
        let emb = DummyEmbedder::new();
        let dir = tempfile::tempdir().unwrap();
        let (_r, handle) = build_index(
            &emb,
            Genre::Developer,
            &sample_docs(),
            &ChunkOptions::default(),
        )
        .unwrap();
        save_index(&handle, dir.path()).unwrap();
        let sidecar = corpus_sidecar_path(dir.path(), Genre::Developer);
        assert!(sidecar.exists(), "corpus sidecar must be written");
        let body = std::fs::read_to_string(&sidecar).unwrap();
        assert!(body.contains("\"id\""), "sidecar holds corpus rows");
    }

    #[test]
    fn load_index_none_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        let loaded = load_index(dir.path(), Genre::Developer).unwrap();
        assert!(loaded.is_none(), "no persisted index → None");
    }

    #[test]
    fn persist_roundtrip_matches_fresh_build() {
        let emb = DummyEmbedder::new();
        let dir = tempfile::tempdir().unwrap();
        let docs = sample_docs();

        // build → save → load
        let (_r, built) =
            build_index(&emb, Genre::Developer, &docs, &ChunkOptions::default()).unwrap();
        save_index(&built, dir.path()).unwrap();
        let loaded = load_index(dir.path(), Genre::Developer)
            .unwrap()
            .expect("index should load");

        // corpus + store survived
        assert_eq!(loaded.corpus.len(), built.corpus.len());
        assert_eq!(loaded.store.len(), built.store.len());

        // search on loaded == search on freshly built (DummyEmbedder is deterministic)
        let q = "rust tokio";
        let fresh = built.search(&emb, q, 3).unwrap();
        let from_disk = loaded.search(&emb, q, 3).unwrap();
        let fresh_ids: Vec<&str> = fresh.iter().map(|h| h.id.as_str()).collect();
        let disk_ids: Vec<&str> = from_disk.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(
            fresh_ids, disk_ids,
            "loaded index must return identical hits"
        );
    }

    // ── incremental reindex ──────────────────────────────────────────────

    use crate::ports::embedder::Embedding;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// DummyEmbedder wrapper that counts `embed` calls (to prove unchanged docs
    /// are not re-embedded). `AtomicUsize` keeps it `Sync` without `unsafe`.
    struct CountingEmbedder {
        inner: DummyEmbedder,
        calls: AtomicUsize,
    }
    impl CountingEmbedder {
        fn new() -> Self {
            Self {
                inner: DummyEmbedder::new(),
                calls: AtomicUsize::new(0),
            }
        }
        fn count(&self) -> usize {
            self.calls.load(Ordering::Relaxed)
        }
    }
    impl EmbedderProvider for CountingEmbedder {
        fn embed(&self, text: &str) -> crate::domain::Result<Embedding> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            self.inner.embed(text)
        }
        fn dim(&self) -> usize {
            self.inner.dim()
        }
        fn backend(&self) -> &'static str {
            self.inner.backend()
        }
    }

    fn idocs(pairs: &[(&str, &str)]) -> Vec<InputDoc> {
        pairs
            .iter()
            .map(|(id, t)| InputDoc {
                id: (*id).into(),
                text: (*t).into(),
            })
            .collect()
    }

    #[test]
    fn incremental_full_build_when_no_manifest() {
        let emb = DummyEmbedder::new();
        let dir = tempfile::tempdir().unwrap();
        let docs = idocs(&[("a", "alpha text"), ("b", "beta text")]);
        let (report, delta) = build_index_incremental(
            &emb,
            dir.path(),
            Genre::Developer,
            &docs,
            &ChunkOptions::default(),
        )
        .unwrap();
        assert_eq!(delta.added.len(), 2, "first run = all added");
        assert!(delta.unchanged.is_empty());
        assert!(report.chunks >= 2);
    }

    #[test]
    fn incremental_skips_unchanged_docs() {
        let dir = tempfile::tempdir().unwrap();
        let docs = idocs(&[("a", "alpha"), ("b", "beta"), ("c", "gamma")]);

        // initial full index
        let emb0 = DummyEmbedder::new();
        build_index_incremental(
            &emb0,
            dir.path(),
            Genre::Developer,
            &docs,
            &ChunkOptions::default(),
        )
        .unwrap();

        // change only "b"; a,c unchanged
        let docs2 = idocs(&[("a", "alpha"), ("b", "beta CHANGED"), ("c", "gamma")]);
        let counter = CountingEmbedder::new();
        let (_r, delta) = build_index_incremental(
            &counter,
            dir.path(),
            Genre::Developer,
            &docs2,
            &ChunkOptions::default(),
        )
        .unwrap();
        assert_eq!(delta.changed, vec!["b"]);
        assert_eq!(delta.unchanged, vec!["a", "c"]);
        // only "b" (1 chunk) re-embedded → exactly 1 embed call (a,c skipped).
        assert_eq!(counter.count(), 1, "only changed doc re-embedded");
    }

    #[test]
    fn incremental_handles_removed_doc() {
        let dir = tempfile::tempdir().unwrap();
        let emb = DummyEmbedder::new();
        build_index_incremental(
            &emb,
            dir.path(),
            Genre::Developer,
            &idocs(&[("a", "alpha"), ("b", "beta")]),
            &ChunkOptions::default(),
        )
        .unwrap();
        // drop "b"
        let (_r, delta) = build_index_incremental(
            &emb,
            dir.path(),
            Genre::Developer,
            &idocs(&[("a", "alpha")]),
            &ChunkOptions::default(),
        )
        .unwrap();
        assert_eq!(delta.removed, vec!["b"]);
        let loaded = load_index(dir.path(), Genre::Developer).unwrap().unwrap();
        assert!(
            loaded
                .corpus
                .iter()
                .all(|(id, _)| !id.starts_with("b::") && id != "b"),
            "removed doc's chunks must be gone"
        );
    }

    #[test]
    fn incremental_equivalent_to_full_rebuild() {
        let docs = idocs(&[("a", "alpha one"), ("b", "beta two"), ("c", "gamma three")]);
        let docs2 = idocs(&[
            ("a", "alpha one"),
            ("b", "beta TWO changed"),
            ("d", "delta four"),
        ]);

        // incremental path: build docs, then reindex docs2
        let dir_inc = tempfile::tempdir().unwrap();
        let emb = DummyEmbedder::new();
        build_index_incremental(
            &emb,
            dir_inc.path(),
            Genre::Developer,
            &docs,
            &ChunkOptions::default(),
        )
        .unwrap();
        build_index_incremental(
            &emb,
            dir_inc.path(),
            Genre::Developer,
            &docs2,
            &ChunkOptions::default(),
        )
        .unwrap();
        let inc = load_index(dir_inc.path(), Genre::Developer)
            .unwrap()
            .unwrap();

        // full path: build docs2 from scratch
        let dir_full = tempfile::tempdir().unwrap();
        let (_r, full) =
            build_index(&emb, Genre::Developer, &docs2, &ChunkOptions::default()).unwrap();
        save_index(&full, dir_full.path()).unwrap();
        let full = load_index(dir_full.path(), Genre::Developer)
            .unwrap()
            .unwrap();

        // same corpus content (order-independent)
        let mut inc_ids: Vec<&str> = inc.corpus.iter().map(|(id, _)| id.as_str()).collect();
        let mut full_ids: Vec<&str> = full.corpus.iter().map(|(id, _)| id.as_str()).collect();
        inc_ids.sort();
        full_ids.sort();
        assert_eq!(
            inc_ids, full_ids,
            "incremental corpus == full-rebuild corpus"
        );
        assert_eq!(inc.store.len(), full.store.len(), "same vector count");
    }
}

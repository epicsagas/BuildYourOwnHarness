//! BYOH self-contained RAG subsystem.
//!
//! `chunk → embed → index → search` pipeline that runs inside the BYOH process,
//! with no external execution-layer tool (alcove/Episteme) required.
//!
//! Two vector-store backends:
//! - [`store::InMemoryStore`] — always available, brute-force cosine. Powers
//!   tests and the default build.
//! - [`store::TurbovecStore`] — behind `native-rag`, wraps `llm_kernel`'s
//!   quantized ANN index for production-scale corpora + persistence.
//!
//! Hybrid search (`search::hybrid`) falls back vector → bm25 → grep, matching
//! ARCH §8.2 and the existing `ProfileSource` scan contract.

pub mod chunk;
pub mod genre_index;
pub mod manifest;
pub mod pipeline;
pub mod search;
pub mod store;

pub use chunk::{Chunk, ChunkOptions, chunk_document};
pub use genre_index::{GenreIndexCatalog, GenreWeights, genre_bm25_weights};
pub use manifest::{IndexDelta, IndexManifest, index_status};
pub use pipeline::{
    BuildReport, IndexHandle, InputDoc, SearchHit, build_index, build_index_incremental,
    load_index, save_index,
};
pub use search::{SearchMode, bm25_search, grep_search, hybrid_search};
#[cfg(feature = "native-rag")]
pub use store::TurbovecStore;
pub use store::{InMemoryStore, VectorStore};

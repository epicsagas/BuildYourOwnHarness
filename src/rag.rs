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
pub mod pipeline;
pub mod search;
pub mod store;

pub use chunk::{chunk_document, Chunk, ChunkOptions};
pub use genre_index::{genre_bm25_weights, GenreIndexCatalog, GenreWeights};
pub use pipeline::{build_index, save_index, BuildReport, IndexHandle, InputDoc, SearchHit};
pub use search::{bm25_search, grep_search, hybrid_search, SearchMode};
#[cfg(feature = "native-rag")]
pub use store::TurbovecStore;
pub use store::{InMemoryStore, VectorStore};

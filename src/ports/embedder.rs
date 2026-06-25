//! Embedder port — abstracts text→vector embedding so the RAG subsystem is
//! backend-agnostic and testable without a model download.
//!
//! The trait is pure (no llm-kernel dependency) so default builds compile.
//! The `DummyEmbedder` (deterministic) is always available; `FastembedEmbedder`
//! and `OpenAIEmbedder` live behind the `native-rag` cargo feature.

/// A single embedding vector.
#[derive(Debug, Clone, PartialEq)]
pub struct Embedding {
    pub vector: Vec<f32>,
}

/// Embeds text into a fixed-dimension vector space.
pub trait EmbedderProvider: Send + Sync {
    /// Embed a single text. Dimension is consistent across calls.
    fn embed(&self, text: &str) -> crate::domain::Result<Embedding>;

    /// Vector dimensionality.
    fn dim(&self) -> usize;

    /// Backend name (for logging / index metadata).
    fn backend(&self) -> &'static str;
}

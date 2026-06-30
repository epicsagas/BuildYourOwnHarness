//! Embedder adapters.
//!
//! - [`DummyEmbedder`] — always available, deterministic hashing-based vectors.
//!   Used by tests and the default (model-free) build.
//! - [`FastembedEmbedder`] / [`OpenAIEmbedder`] — behind the `native-rag` cargo
//!   feature, wrapping `llm_kernel::embedding` backends.

use crate::ports::embedder::{EmbedderProvider, Embedding};

/// Deterministic, model-free embedder. Vectors are derived from a stable hash
/// of the text — **not** semantically meaningful, but reproducible and zero-cost.
/// Used so the RAG pipeline can be built and tested without downloading a model.
#[derive(Debug, Clone)]
pub struct DummyEmbedder {
    pub dim: usize,
}

impl DummyEmbedder {
    pub const DEFAULT_DIM: usize = 64;

    pub fn new() -> Self {
        Self {
            dim: Self::DEFAULT_DIM,
        }
    }

    pub fn with_dim(dim: usize) -> Self {
        Self { dim: dim.max(8) }
    }
}

impl Default for DummyEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbedderProvider for DummyEmbedder {
    fn embed(&self, text: &str) -> crate::domain::Result<Embedding> {
        // Deterministic pseudo-embedding: spread bytes of a stable hash across dims.
        let mut vector = vec![0.0f32; self.dim];
        let bytes = text.as_bytes();
        if bytes.is_empty() {
            return Ok(Embedding { vector });
        }
        let mut acc: u64 = 0xcbf29ce484222325; // FNV offset
        for (i, b) in bytes.iter().enumerate() {
            acc ^= *b as u64;
            acc = acc.wrapping_mul(0x100000001b3);
            let slot = (i + (acc as usize)) % self.dim;
            vector[slot] += ((acc >> 8) as f32 / u32::MAX as f32) - 0.5;
        }
        // L2 normalize so cosine similarity is well-defined.
        let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-9);
        for v in &mut vector {
            *v /= norm;
        }
        Ok(Embedding { vector })
    }

    fn dim(&self) -> usize {
        self.dim
    }

    fn backend(&self) -> &'static str {
        "dummy"
    }
}

// ──────────────────────────────────────────────────────────────────────────
// native-rag feature: real embedders backed by llm-kernel
// ──────────────────────────────────────────────────────────────────────────

#[cfg(feature = "native-rag")]
pub mod native {
    //! Real embedding backends — only compiled with `--features native-rag`.

    // llm-kernel's EmbeddingProvider trait is distinct from BYOH's EmbedderProvider;
    // alias it to call its methods on the inner provider.
    use llm_kernel::embedding::EmbeddingProvider as LkEmbeddingProvider;

    use crate::ports::embedder::{EmbedderProvider, Embedding};

    /// Local multilingual-e5-small embedder via fastembed (no API key).
    /// First call downloads the model (~120MB) into the fastembed cache.
    pub struct FastembedEmbedder {
        inner: llm_kernel::embedding::FastembedProvider,
        dim_cache: usize,
    }

    impl FastembedEmbedder {
        pub fn new() -> crate::domain::Result<Self> {
            let inner = llm_kernel::embedding::FastembedProvider::new(
                llm_kernel::embedding::EmbeddingModel::MultilingualE5Small,
                None,
            )
            .map_err(|e| crate::domain::ByohError::Other(format!("fastembed init: {e}")))?;
            let dim = inner.dim();
            Ok(Self {
                inner,
                dim_cache: dim,
            })
        }
    }

    impl EmbedderProvider for FastembedEmbedder {
        fn embed(&self, text: &str) -> crate::domain::Result<Embedding> {
            let res = LkEmbeddingProvider::embed(&self.inner, text)
                .map_err(|e| crate::domain::ByohError::Other(format!("fastembed embed: {e}")))?;
            Ok(Embedding { vector: res.vector })
        }

        fn dim(&self) -> usize {
            self.dim_cache
        }

        fn backend(&self) -> &'static str {
            "fastembed:multilingual-e5-small"
        }
    }

    /// OpenAI text-embedding embedder. Requires `OPENAI_API_KEY` at runtime;
    /// only constructed when the user explicitly opts in via the `rag-openai`
    /// feature + config.
    #[cfg(feature = "rag-openai")]
    pub struct OpenAIEmbedder {
        inner: llm_kernel::embedding::OpenAIEmbeddingClient,
        dim_cache: usize,
    }

    #[cfg(feature = "rag-openai")]
    impl OpenAIEmbedder {
        pub fn new(model: &str, api_key: &str) -> crate::domain::Result<Self> {
            // llm-kernel 0.10: new_with_model(api_key, model, dim) — infallible.
            let dim = if model.contains("3-large") {
                3072
            } else {
                1536
            };
            let inner =
                llm_kernel::embedding::OpenAIEmbeddingClient::new_with_model(api_key, model, dim);
            Ok(Self {
                inner,
                dim_cache: dim,
            })
        }
    }

    #[cfg(feature = "rag-openai")]
    impl EmbedderProvider for OpenAIEmbedder {
        fn embed(&self, text: &str) -> crate::domain::Result<Embedding> {
            let res = LkEmbeddingProvider::embed(&self.inner, text)
                .map_err(|e| crate::domain::ByohError::Other(format!("openai embed: {e}")))?;
            Ok(Embedding { vector: res.vector })
        }
        fn dim(&self) -> usize {
            self.dim_cache
        }
        fn backend(&self) -> &'static str {
            "openai:text-embedding-3"
        }
    }
}

#[cfg(feature = "native-rag")]
pub use native::FastembedEmbedder;
#[cfg(feature = "rag-openai")]
pub use native::OpenAIEmbedder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_is_deterministic_and_normalized() {
        let e = DummyEmbedder::new();
        let a = e.embed("hello world").unwrap();
        let b = e.embed("hello world").unwrap();
        assert_eq!(a, b, "deterministic");
        let norm: f32 = a.vector.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-3, "L2 normalized: {norm}");
    }

    #[test]
    fn dummy_dim_respected() {
        let e = DummyEmbedder::with_dim(32);
        assert_eq!(e.dim(), 32);
        assert_eq!(e.embed("x").unwrap().vector.len(), 32);
    }

    #[test]
    fn dummy_backend_name() {
        assert_eq!(DummyEmbedder::new().backend(), "dummy");
    }

    #[test]
    fn different_texts_different_vectors() {
        let e = DummyEmbedder::new();
        let a = e.embed("rust").unwrap();
        let b = e.embed("python").unwrap();
        assert_ne!(a.vector, b.vector);
    }
}

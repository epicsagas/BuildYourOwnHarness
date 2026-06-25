//! Chunking — split a document into indexable pieces.
//!
//! With `native-rag`, delegates to `llm_kernel::tokens::chunk_text` (CJK/Hangul
//! sentence-boundary aware). Without it, a simple whitespace/length splitter
//! keeps the pipeline functional for tests and the default build.

use serde::{Deserialize, Serialize};

/// Chunking parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkOptions {
    /// Target maximum tokens per chunk.
    pub max_tokens: usize,
    /// Overlap in tokens between adjacent chunks.
    pub overlap: usize,
}

impl Default for ChunkOptions {
    fn default() -> Self {
        Self {
            max_tokens: 256,
            overlap: 32,
        }
    }
}

impl ChunkOptions {
    pub fn new(max_tokens: usize, overlap: usize) -> Self {
        Self {
            max_tokens: max_tokens.max(16),
            overlap: overlap.min(max_tokens.saturating_sub(1)),
        }
    }
}

/// An indexable chunk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    /// Stable id within a document (e.g. "<doc_id>::<n>").
    pub id: String,
    /// Source document id.
    pub doc_id: String,
    /// 0-based ordinal within the document.
    pub ord: usize,
    pub text: String,
}

/// Split a document into chunks. `doc_id` seeds chunk ids.
pub fn chunk_document(doc_id: &str, text: &str, opts: &ChunkOptions) -> Vec<Chunk> {
    let pieces = chunk_text_inner(text, opts);
    pieces
        .into_iter()
        .enumerate()
        .map(|(i, t)| Chunk {
            id: format!("{doc_id}::{i}"),
            doc_id: doc_id.to_string(),
            ord: i,
            text: t,
        })
        .collect()
}

#[cfg(feature = "native-rag")]
fn chunk_text_inner(text: &str, opts: &ChunkOptions) -> Vec<String> {
    let lk_opts = llm_kernel::tokens::ChunkOptions::new(opts.max_tokens, opts.overlap);
    llm_kernel::tokens::chunk_text(text, &lk_opts)
}

#[cfg(not(feature = "native-rag"))]
fn chunk_text_inner(text: &str, opts: &ChunkOptions) -> Vec<String> {
    // Simple fallback: tokenize on whitespace, group by max_tokens with overlap.
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut start = 0;
    while start < words.len() {
        let end = (start + opts.max_tokens).min(words.len());
        let piece = words[start..end].join(" ");
        if !piece.is_empty() {
            out.push(piece);
        }
        if end >= words.len() {
            break;
        }
        start = end.saturating_sub(opts.overlap);
        if start == 0 || start == end {
            start = end;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_have_ids_and_order() {
        // Large input with small max_tokens guarantees multiple chunks under both
        // the fallback splitter and llm-kernel's tokenizer.
        let text = (0..2000)
            .map(|i| format!("word{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let chunks = chunk_document("doc1", &text, &ChunkOptions::new(32, 4));
        assert!(!chunks.is_empty(), "should produce at least one chunk");
        for (i, c) in chunks.iter().enumerate() {
            assert_eq!(c.ord, i);
            assert!(c.id.starts_with("doc1::"));
        }
    }

    #[test]
    fn empty_text_no_chunks() {
        assert!(chunk_document("d", "", &ChunkOptions::default()).is_empty());
    }

    #[test]
    fn options_clamp() {
        let o = ChunkOptions::new(1, 100);
        assert!(o.overlap < o.max_tokens);
    }
}

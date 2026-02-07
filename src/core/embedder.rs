//! Embedding generation for semantic search
//!
//! Uses fastembed for local embedding generation with the all-MiniLM-L6-v2 model.

use std::sync::Mutex;

use crate::error::{AppError, Result};

/// Chunk of text with metadata for embedding
#[derive(Debug, Clone)]
pub struct TextChunk {
    /// The text content to embed
    pub text: String,
    /// Start offset in original content (characters)
    pub start_offset: usize,
    /// End offset in original content (characters)
    pub end_offset: usize,
}

/// Embedding result for a chunk
#[derive(Debug, Clone)]
pub struct ChunkEmbedding {
    /// The chunk that was embedded
    pub chunk: TextChunk,
    /// The embedding vector (384 dimensions for `MiniLM`)
    pub embedding: Vec<f32>,
}

/// Embedding generator using fastembed
pub struct Embedder {
    model: Mutex<fastembed::TextEmbedding>,
}

impl Embedder {
    /// Create a new embedder with the specified model
    pub fn new(model_name: &str) -> Result<Self> {
        let model_type = Self::parse_model_name(model_name)?;

        let options = fastembed::TextInitOptions::new(model_type);

        let model = fastembed::TextEmbedding::try_new(options)
            .map_err(|e| AppError::Other(format!("Failed to load embedding model: {e}")))?;

        Ok(Self {
            model: Mutex::new(model),
        })
    }

    /// Parse model name string to fastembed model type
    fn parse_model_name(name: &str) -> Result<fastembed::EmbeddingModel> {
        match name.to_lowercase().as_str() {
            "all-minilm-l6-v2" | "minilm" => Ok(fastembed::EmbeddingModel::AllMiniLML6V2),
            "bge-small-en-v1.5" | "bge-small" => Ok(fastembed::EmbeddingModel::BGESmallENV15),
            "bge-base-en-v1.5" | "bge-base" => Ok(fastembed::EmbeddingModel::BGEBaseENV15),
            _ => Err(AppError::Config(format!(
                "Unknown embedding model: {name}. Supported: all-MiniLM-L6-v2, bge-small-en-v1.5, bge-base-en-v1.5"
            ))),
        }
    }

    /// Get the embedding dimension for the loaded model
    #[must_use]
    #[allow(dead_code)]
    #[allow(clippy::unused_self)]
    pub fn dimension(&self) -> usize {
        384 // all-MiniLM-L6-v2 dimension
    }

    /// Split text into chunks with overlap
    ///
    /// Uses ~512 token approximation (chars/4) with 50 token overlap
    pub fn chunk_text(content: &str, max_tokens: usize, overlap_tokens: usize) -> Vec<TextChunk> {
        // Approximate tokens as chars/4 (rough estimate)
        let max_chars = max_tokens * 4;
        let overlap_chars = overlap_tokens * 4;

        if content.len() <= max_chars {
            return vec![TextChunk {
                text: content.to_string(),
                start_offset: 0,
                end_offset: content.len(),
            }];
        }

        let mut chunks = Vec::new();
        let mut start = 0;

        while start < content.len() {
            let end = (start + max_chars).min(content.len());

            // Try to break at word boundary
            let actual_end = if end < content.len() {
                content[start..end]
                    .rfind(char::is_whitespace)
                    .map_or(end, |pos| start + pos)
            } else {
                end
            };

            let chunk_text = content[start..actual_end].trim();
            if !chunk_text.is_empty() {
                chunks.push(TextChunk {
                    text: chunk_text.to_string(),
                    start_offset: start,
                    end_offset: actual_end,
                });
            }

            // Move forward with overlap
            start = if actual_end >= content.len() {
                actual_end
            } else {
                (actual_end - overlap_chars).max(start + 1)
            };

            // Avoid infinite loop
            if start >= actual_end && actual_end < content.len() {
                start = actual_end;
            }
        }

        chunks
    }

    /// Generate embeddings for text chunks
    pub fn embed_chunks(&self, chunks: &[TextChunk]) -> Result<Vec<ChunkEmbedding>> {
        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        let texts: Vec<&str> = chunks.iter().map(|c| c.text.as_str()).collect();

        let mut model = self
            .model
            .lock()
            .map_err(|e| AppError::Other(format!("Failed to lock model: {e}")))?;

        let embeddings = model
            .embed(texts, None)
            .map_err(|e| AppError::Other(format!("Failed to generate embeddings: {e}")))?;

        let results = chunks
            .iter()
            .cloned()
            .zip(embeddings)
            .map(|(chunk, embedding)| ChunkEmbedding { chunk, embedding })
            .collect();

        Ok(results)
    }

    /// Generate embedding for a single query string
    pub fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
        let mut model = self
            .model
            .lock()
            .map_err(|e| AppError::Other(format!("Failed to lock model: {e}")))?;

        let embeddings = model
            .embed(vec![query], None)
            .map_err(|e| AppError::Other(format!("Failed to generate query embedding: {e}")))?;

        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Other("No embedding generated".into()))
    }

    /// Generate embeddings for file content
    pub fn embed_content(&self, content: &str) -> Result<Vec<ChunkEmbedding>> {
        let chunks = Self::chunk_text(content, 512, 50);
        self.embed_chunks(&chunks)
    }
}

/// Calculate cosine similarity between two vectors
#[must_use]
#[allow(dead_code)]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text_small() {
        let content = "Hello world";
        let chunks = Embedder::chunk_text(content, 512, 50);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, "Hello world");
    }

    #[test]
    fn test_chunk_text_large() {
        let content = "word ".repeat(1000);
        let chunks = Embedder::chunk_text(&content, 100, 10);
        assert!(chunks.len() > 1);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &c).abs() < 0.001);
    }
}

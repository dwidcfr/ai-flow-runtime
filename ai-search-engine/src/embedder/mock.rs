use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::embedder::Embedder;
use crate::error::{Result, SearchError};

const DEFAULT_DIMENSIONS: usize = 128;

pub struct MockEmbedder {
    dimensions: usize,
}

impl Default for MockEmbedder {
    fn default() -> Self {
        Self::new(DEFAULT_DIMENSIONS)
    }
}

impl MockEmbedder {
    pub fn new(dimensions: usize) -> Self {
        Self { dimensions }
    }

    fn hash_token(token: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        hasher.finish()
    }

    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty())
            .map(|t| t.to_string())
            .collect()
    }

    fn normalize(vector: &mut [f32]) {
        let norm: f32 = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in vector.iter_mut() {
                *value /= norm;
            }
        }
    }
}

impl Embedder for MockEmbedder {
    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        if self.dimensions == 0 {
            return Err(SearchError::EmbeddingError {
                message: "dimensions must be greater than zero".to_string(),
            });
        }

        let mut vector = vec![0.0_f32; self.dimensions];
        for token in Self::tokenize(text) {
            let hash = Self::hash_token(&token);
            let bucket = (hash as usize) % self.dimensions;
            let sign = if hash % 2 == 0 { 1.0 } else { -1.0 };
            vector[bucket] += sign;
        }

        Self::normalize(&mut vector);
        Ok(vector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_is_deterministic() {
        let embedder = MockEmbedder::default();
        let first = embedder.embed("Click payment methods").unwrap();
        let second = embedder.embed("Click payment methods").unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn different_texts_produce_different_vectors() {
        let embedder = MockEmbedder::default();
        let payment = embedder.embed("Click Payme payment").unwrap();
        let greeting = embedder.embed("Добрый день привет").unwrap();
        assert_ne!(payment, greeting);
    }

    #[test]
    fn vector_is_normalized() {
        let embedder = MockEmbedder::default();
        let vector = embedder.embed("some searchable text").unwrap();
        let norm: f32 = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5 || norm == 0.0);
    }

    #[test]
    fn similar_texts_have_higher_cosine_similarity() {
        let embedder = MockEmbedder::default();
        let base = embedder.embed("Click Payme payment methods").unwrap();
        let similar = embedder.embed("Click payment Payme").unwrap();
        let unrelated = embedder.embed("greetings hello world").unwrap();

        let sim_score = cosine_similarity(&base, &similar);
        let unrelated_score = cosine_similarity(&base, &unrelated);
        assert!(sim_score > unrelated_score);
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }
}

mod mock;

pub use mock::MockEmbedder;

use crate::error::Result;

pub trait Embedder: Send + Sync {
    fn dimensions(&self) -> usize;
    fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

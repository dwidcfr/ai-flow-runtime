mod memory;

pub use memory::InMemoryIndex;

use crate::error::Result;
use crate::types::{ScoredCandidate, SearchDocument};

pub trait Index: Send + Sync {
    fn add(&mut self, doc: SearchDocument, embedding: Vec<f32>) -> Result<()>;
    fn remove(&mut self, document_id: &str) -> Result<()>;
    fn update(&mut self, doc: SearchDocument, embedding: Vec<f32>) -> Result<()>;
    fn clear_module(&mut self, module_id: &str) -> Result<()>;
    fn clear(&mut self) -> Result<()>;
    fn search(
        &self,
        query_embedding: &[f32],
        module_filter: Option<&[String]>,
        limit: usize,
    ) -> Result<Vec<ScoredCandidate>>;
    fn len(&self) -> usize;
}

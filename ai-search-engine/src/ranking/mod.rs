mod simple;

pub use simple::SimpleRanker;

use crate::types::{ScoredCandidate, SearchResult};

pub trait Ranker: Send + Sync {
    fn rank(
        &self,
        query: &str,
        candidates: Vec<ScoredCandidate>,
        top_k: usize,
    ) -> Vec<SearchResult>;
}

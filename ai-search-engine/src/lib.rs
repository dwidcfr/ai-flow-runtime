pub mod embedder;
pub mod error;
pub mod index;
pub mod providers;
pub mod ranking;
pub mod search;
pub mod types;

pub use embedder::{Embedder, MockEmbedder};
pub use error::{Result, SearchError};
pub use index::{Index, InMemoryIndex};
pub use providers::SearchProvider;
pub use ranking::{Ranker, SimpleRanker};
pub use search::SearchEngine;
pub use types::{ScoredCandidate, SearchDocument, SearchResult};

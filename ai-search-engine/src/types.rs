use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchDocument {
    pub document_id: String,
    pub module_id: String,
    pub section_id: Option<String>,
    pub title: String,
    pub content: String,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub document_id: String,
    pub module_id: String,
    pub section_id: Option<String>,
    pub score: f32,
    pub title: String,
    pub snippet: String,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoredCandidate {
    pub document: SearchDocument,
    pub score: f32,
}

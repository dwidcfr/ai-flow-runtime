use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouterHistoryRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouterHistoryEntry {
    pub role: RouterHistoryRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouterSearchCandidate {
    pub document_id: String,
    pub module_id: String,
    pub section_id: Option<String>,
    pub score: f32,
    pub title: String,
    pub snippet: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouterContext {
    pub user_message: String,
    pub current_node: String,
    pub available_actions: Vec<String>,
    pub history: Vec<RouterHistoryEntry>,
    pub client_data: HashMap<String, Value>,
    pub conversation_context: HashMap<String, String>,
    pub search_candidates: Vec<RouterSearchCandidate>,
    #[serde(default)]
    pub prompt_set_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RouterDecision {
    Flow {
        action: String,
        confidence: f32,
    },
    Module {
        module_id: String,
        section_id: Option<String>,
        confidence: f32,
    },
    Clarify {
        reason: String,
    },
    EndConversation {
        reason: String,
    },
    Error {
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedDecision {
    Flow {
        action: String,
        confidence: f32,
    },
    Module {
        module_id: String,
        section_id: Option<String>,
        confidence: f32,
    },
    Clarify {
        reason: String,
    },
    End {
        reason: String,
    },
    Error {
        code: String,
        message: String,
    },
}

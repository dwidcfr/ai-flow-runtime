use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::identity::AgentIdentity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseHistoryRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseHistoryEntry {
    pub role: ResponseHistoryRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseDecisionKind {
    Flow {
        action: String,
        current_node: String,
        node_task: Option<String>,
        node_payload: Value,
    },
    Module {
        module_id: String,
        section_id: Option<String>,
        title: String,
        content: String,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseContext {
    pub user_message: String,
    pub decision: ResponseDecisionKind,
    pub client_data: HashMap<String, Value>,
    pub conversation_context: HashMap<String, String>,
    pub identity: AgentIdentity,
    pub history: Vec<ResponseHistoryEntry>,
    #[serde(default)]
    pub prompt_set_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub text: String,
    pub finish_conversation: bool,
}

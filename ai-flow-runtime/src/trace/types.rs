use std::collections::HashMap;

use ai_response_engine::AgentIdentity;
use ai_router_engine::RouterDecision;
use ai_search_engine::SearchResult;
use serde::{Deserialize, Serialize};

use crate::state::RuntimeState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineStep {
    pub step: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowTraceSnapshot {
    pub flow_id: Option<String>,
    pub previous_node: Option<String>,
    pub current_node: Option<String>,
    pub paused_node: Option<String>,
    pub available_transitions: Vec<String>,
    pub node_payload: serde_json::Value,
    pub is_end: bool,
    pub runtime_state: RuntimeState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleTraceSnapshot {
    pub active_module: Option<String>,
    pub conversation_entries: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTraceSnapshot {
    pub prompt_set: String,
    pub identity: AgentIdentity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTrace {
    pub search_results: Vec<SearchResult>,
    pub search_ms: u64,
    pub router_prompt: String,
    pub router_raw_json: String,
    pub router_decision: RouterDecision,
    pub router_ms: u64,
    pub response_prompt: String,
    pub response_raw_json: String,
    pub response_ms: u64,
    pub total_ms: u64,
    pub timeline: Vec<TimelineStep>,
    pub flow: FlowTraceSnapshot,
    pub modules: ModuleTraceSnapshot,
    pub prompts: PromptTraceSnapshot,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HandleMessageOptions {
    pub collect_trace: bool,
}

#[derive(Debug, Clone)]
pub struct HandleMessageResult {
    pub decision: RouterDecision,
    pub response: ai_response_engine::Response,
    pub trace: Option<MessageTrace>,
}

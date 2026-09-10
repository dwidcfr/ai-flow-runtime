use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use ai_flow_runtime::MessageTrace;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SearchCandidateDto {
    pub module_id: String,
    pub section_id: Option<String>,
    pub score: f32,
    pub title: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TimelineStepDto {
    pub step: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FlowDebugDto {
    pub flow_id: Option<String>,
    pub previous_node: Option<String>,
    pub current_node: Option<String>,
    pub paused_node: Option<String>,
    pub available_transitions: Vec<String>,
    pub is_end: bool,
    pub runtime_state: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ModuleDebugDto {
    pub active_module: Option<String>,
    pub conversation_entries: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PromptDebugDto {
    pub prompt_set: String,
    pub identity_name: String,
    pub identity_role: String,
    pub router_prompt: String,
    pub response_prompt: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RuntimeDebugDto {
    pub search_ms: u64,
    pub router_ms: u64,
    pub response_ms: u64,
    pub total_ms: u64,
    pub timeline: Vec<TimelineStepDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct JsonDebugDto {
    pub router_raw_json: String,
    pub response_raw_json: String,
    #[schema(value_type = Object)]
    pub router_decision: Value,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MessageDebugDto {
    pub search_candidates: Vec<SearchCandidateDto>,
    pub flow: FlowDebugDto,
    pub modules: ModuleDebugDto,
    pub prompts: PromptDebugDto,
    pub runtime: RuntimeDebugDto,
    pub json: JsonDebugDto,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct MessageQuery {
    #[serde(default)]
    pub debug: bool,
}

pub fn message_debug_from_trace(trace: &MessageTrace) -> MessageDebugDto {
    MessageDebugDto {
        search_candidates: trace
            .search_results
            .iter()
            .map(|r| SearchCandidateDto {
                module_id: r.module_id.clone(),
                section_id: r.section_id.clone(),
                score: r.score,
                title: r.title.clone(),
                snippet: r.snippet.clone(),
            })
            .collect(),
        flow: FlowDebugDto {
            flow_id: trace.flow.flow_id.clone(),
            previous_node: trace.flow.previous_node.clone(),
            current_node: trace.flow.current_node.clone(),
            paused_node: trace.flow.paused_node.clone(),
            available_transitions: trace.flow.available_transitions.clone(),
            is_end: trace.flow.is_end,
            runtime_state: format!("{:?}", trace.flow.runtime_state),
        },
        modules: ModuleDebugDto {
            active_module: trace.modules.active_module.clone(),
            conversation_entries: trace.modules.conversation_entries.clone(),
        },
        prompts: PromptDebugDto {
            prompt_set: trace.prompts.prompt_set.clone(),
            identity_name: trace.prompts.identity.name.clone(),
            identity_role: trace.prompts.identity.role.clone(),
            router_prompt: trace.router_prompt.clone(),
            response_prompt: trace.response_prompt.clone(),
        },
        runtime: RuntimeDebugDto {
            search_ms: trace.search_ms,
            router_ms: trace.router_ms,
            response_ms: trace.response_ms,
            total_ms: trace.total_ms,
            timeline: trace
                .timeline
                .iter()
                .map(|s| TimelineStepDto {
                    step: s.step.clone(),
                    duration_ms: s.duration_ms,
                })
                .collect(),
        },
        json: JsonDebugDto {
            router_raw_json: trace.router_raw_json.clone(),
            response_raw_json: trace.response_raw_json.clone(),
            router_decision: serde_json::to_value(&trace.router_decision).unwrap_or(Value::Null),
        },
    }
}

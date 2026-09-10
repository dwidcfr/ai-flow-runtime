use ai_response_engine::ResponseDecisionKind;
use ai_router_engine::RouterDecision;
use serde_json::Value;

use crate::error::{Result, RuntimeError};
use crate::modules::ModuleContent;
use crate::runtime::Runtime;

pub fn map_router_decision(
    runtime: &Runtime,
    session_id: &str,
    decision: &RouterDecision,
) -> Result<ResponseDecisionKind> {
    match decision {
        RouterDecision::Flow { action, .. } => {
            let session = runtime.get_session(session_id)?;
            let current_node = session
                .current_node
                .clone()
                .ok_or_else(|| RuntimeError::NodeNotFound {
                    flow_id: session.flow_id.clone().unwrap_or_default(),
                    node_id: "none".to_string(),
                })?;
            let payload = runtime.get_current_node_payload(session_id)?;
            let node_task = payload
                .get("task")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Ok(ResponseDecisionKind::Flow {
                action: action.clone(),
                current_node,
                node_task,
                node_payload: payload,
            })
        }
        RouterDecision::Module {
            module_id,
            section_id,
            ..
        } => {
            let content = runtime.get_module_content(
                session_id,
                module_id,
                section_id.as_deref(),
            )?;
            let (title, body) = module_content_to_parts(&content);
            Ok(ResponseDecisionKind::Module {
                module_id: module_id.clone(),
                section_id: section_id.clone(),
                title,
                content: body,
            })
        }
        RouterDecision::Clarify { reason } => Ok(ResponseDecisionKind::Clarify {
            reason: reason.clone(),
        }),
        RouterDecision::EndConversation { reason } => Ok(ResponseDecisionKind::EndConversation {
            reason: reason.clone(),
        }),
        RouterDecision::Error { code, message } => Ok(ResponseDecisionKind::Error {
            code: code.clone(),
            message: message.clone(),
        }),
    }
}

fn module_content_to_parts(content: &ModuleContent) -> (String, String) {
    if let Some(map) = content.data.as_object() {
        if let (Some(description), Some(body)) = (
            map.get("description").and_then(|v| v.as_str()),
            map.get("content").and_then(|v| v.as_str()),
        ) {
            return (description.to_string(), body.to_string());
        }

        if map.len() == 1 {
            let (key, value) = map.iter().next().unwrap();
            let body = value.as_str().unwrap_or_default().to_string();
            return (key.clone(), body);
        }
    }

    (
        "Module".to_string(),
        content.data.to_string(),
    )
}

pub fn node_task_from_payload(payload: &Value) -> Option<String> {
    payload.get("task").and_then(|v| v.as_str()).map(str::to_string)
}

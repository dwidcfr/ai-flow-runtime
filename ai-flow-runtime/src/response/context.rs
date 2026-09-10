use std::collections::HashMap;

use ai_response_engine::{
    ResponseContext, ResponseDecisionKind, ResponseHistoryEntry, ResponseHistoryRole,
};
use ai_router_engine::RouterDecision;

use crate::error::Result;
use crate::modules::CONVERSATION_MODULE_ID;
use crate::response::mapper::map_router_decision;
use crate::runtime::Runtime;
use crate::session::HistoryRole;

impl Runtime {
    pub fn build_response_context(
        &self,
        session_id: &str,
        user_message: &str,
        decision: &RouterDecision,
    ) -> Result<ResponseContext> {
        let session = self.get_session(session_id)?;

        let history = session
            .history
            .iter()
            .map(|entry| ResponseHistoryEntry {
                role: match entry.role {
                    HistoryRole::User => ResponseHistoryRole::User,
                    HistoryRole::Assistant => ResponseHistoryRole::Assistant,
                    HistoryRole::System => ResponseHistoryRole::System,
                },
                content: entry.content.clone(),
            })
            .collect();

        let conversation_context = self
            .get_module_content(session_id, CONVERSATION_MODULE_ID, None)
            .map(|content| conversation_map_from_value(&content.data))
            .unwrap_or_default();

        let prompt_set_id = self.session_prompt_set_id(&session)?;
        let identity = self
            .prompt_registry()
            .read()
            .map_err(|e| crate::error::RuntimeError::Configuration {
                message: format!("prompt registry lock poisoned: {e}"),
            })?
            .identity(&prompt_set_id)
            .map_err(|e| crate::error::RuntimeError::Configuration {
                message: format!("failed to load identity for prompt set {prompt_set_id}: {e}"),
            })?;

        Ok(ResponseContext {
            user_message: user_message.to_string(),
            decision: map_router_decision(self, session_id, decision)?,
            client_data: session.client_data.clone(),
            conversation_context,
            identity,
            history,
            prompt_set_id,
        })
    }

    pub fn build_opening_response_context(&self, session_id: &str) -> Result<ResponseContext> {
        let session = self.get_session(session_id)?;
        let current_node = session
            .current_node
            .clone()
            .ok_or_else(|| crate::error::RuntimeError::NodeNotFound {
                flow_id: session.flow_id.clone().unwrap_or_default(),
                node_id: "none".to_string(),
            })?;
        let payload = self.get_current_node_payload(session_id)?;
        let node_task = payload
            .get("task")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let prompt_set_id = self.session_prompt_set_id(&session)?;
        let identity = self
            .prompt_registry()
            .read()
            .map_err(|e| crate::error::RuntimeError::Configuration {
                message: format!("prompt registry lock poisoned: {e}"),
            })?
            .identity(&prompt_set_id)
            .map_err(|e| crate::error::RuntimeError::Configuration {
                message: format!("failed to load identity for prompt set {prompt_set_id}: {e}"),
            })?;

        Ok(ResponseContext {
            user_message: String::new(),
            decision: ResponseDecisionKind::Flow {
                action: "opening".to_string(),
                current_node,
                node_task,
                node_payload: payload,
            },
            client_data: session.client_data.clone(),
            conversation_context: HashMap::new(),
            identity,
            history: Vec::new(),
            prompt_set_id,
        })
    }
}

fn conversation_map_from_value(value: &serde_json::Value) -> HashMap<String, String> {
    value
        .as_object()
        .map(|map| {
            map.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

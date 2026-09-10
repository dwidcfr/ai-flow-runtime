use std::collections::HashMap;

use ai_router_engine::{
    RouterContext, RouterHistoryEntry, RouterHistoryRole, RouterSearchCandidate,
};

use crate::error::{Result, RuntimeError};
use crate::modules::CONVERSATION_MODULE_ID;
use crate::runtime::Runtime;
use crate::session::HistoryRole;

impl Runtime {
    pub fn build_router_context(
        &self,
        session_id: &str,
        user_message: &str,
    ) -> Result<RouterContext> {
        let session = self.get_session(session_id)?;
        ensure_active_for_route(session.runtime_state)?;

        let current_node = session
            .current_node
            .clone()
            .ok_or_else(|| RuntimeError::NodeNotFound {
                flow_id: session.flow_id.clone().unwrap_or_default(),
                node_id: "none".to_string(),
            })?;

        let available_actions = self.available_transitions(session_id)?;
        let search_results = self.search_session(session_id, user_message, None, 5)?;

        let history = session
            .history
            .iter()
            .map(|entry| RouterHistoryEntry {
                role: match entry.role {
                    HistoryRole::User => RouterHistoryRole::User,
                    HistoryRole::Assistant => RouterHistoryRole::Assistant,
                    HistoryRole::System => RouterHistoryRole::System,
                },
                content: entry.content.clone(),
            })
            .collect();

        let conversation_context = self
            .get_module_content(session_id, CONVERSATION_MODULE_ID, None)
            .map(|content| conversation_map_from_value(&content.data))
            .unwrap_or_default();

        let search_candidates = search_results
            .into_iter()
            .map(|result| RouterSearchCandidate {
                document_id: result.document_id,
                module_id: result.module_id,
                section_id: result.section_id,
                score: result.score,
                title: result.title,
                snippet: result.snippet,
            })
            .collect();

        Ok(RouterContext {
            user_message: user_message.to_string(),
            current_node,
            available_actions,
            history,
            client_data: session.client_data.clone(),
            conversation_context,
            search_candidates,
            prompt_set_id: self.session_prompt_set_id(session)?,
        })
    }

    pub(crate) fn session_prompt_set_id(&self, session: &crate::session::Session) -> Result<String> {
        if let Some(value) = session.get_metadata("prompt_set_id").and_then(|v| v.as_str()) {
            return Ok(value.to_string());
        }
        let registry_arc = self.prompt_registry();
        let registry = registry_arc.read().map_err(|e| {
            crate::error::RuntimeError::Configuration {
                message: format!("prompt registry lock poisoned: {e}"),
            }
        })?;
        Ok(registry.default_set_id().to_string())
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

fn ensure_active_for_route(state: crate::state::RuntimeState) -> Result<()> {
    if state == crate::state::RuntimeState::Finished {
        return Err(RuntimeError::SessionFinished);
    }
    Ok(())
}

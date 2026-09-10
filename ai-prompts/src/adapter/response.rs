use serde_json::json;

use ai_response_engine::{PromptBuilder, ResponseContext, ResponseHistoryEntry};
use std::sync::{Arc, RwLock};

use crate::registry::PromptRegistry;
use crate::renderer::render_template;
use crate::types::RenderVars;

pub struct RegistryResponsePromptBuilder {
    registry: Arc<RwLock<PromptRegistry>>,
}

impl RegistryResponsePromptBuilder {
    pub fn new(registry: Arc<RwLock<PromptRegistry>>) -> Self {
        Self { registry }
    }
}

impl PromptBuilder for RegistryResponsePromptBuilder {
    fn build(&self, context: &ResponseContext) -> String {
        self.build_inner(context)
            .unwrap_or_else(|e| format!("Prompt build error: {e}"))
    }
}

impl RegistryResponsePromptBuilder {
    fn build_inner(&self, context: &ResponseContext) -> Result<String, String> {
        let set_id = if context.prompt_set_id.is_empty() {
            let registry = self.registry.read().map_err(|e| e.to_string())?;
            registry.default_set_id().to_string()
        } else {
            context.prompt_set_id.clone()
        };

        let registry = self.registry.read().map_err(|e| e.to_string())?;
        if !registry.has_set(&set_id) {
            return Err(format!("prompt set not found: {set_id}"));
        }
        let template = registry
            .response_template(&set_id)
            .map_err(|e| e.to_string())?;

        let context_json = build_response_context_json(context);
        let mut vars = RenderVars::default();
        vars.insert("context_json", context_json);
        vars.insert("identity.name", context.identity.name.clone());
        vars.insert(
            "flow.node",
            match &context.decision {
                ai_response_engine::ResponseDecisionKind::Flow { current_node, .. } => {
                    current_node.clone()
                }
                _ => String::new(),
            },
        );
        vars.insert(
            "module.content",
            match &context.decision {
                ai_response_engine::ResponseDecisionKind::Module { content, .. } => {
                    content.clone()
                }
                _ => String::new(),
            },
        );
        vars.insert("history", history_json(&context.history));

        Ok(render_template(template, &vars))
    }
}

fn build_response_context_json(context: &ResponseContext) -> String {
    const MAX_HISTORY_ENTRIES: usize = 10;

    let history: Vec<_> = context
        .history
        .iter()
        .rev()
        .take(MAX_HISTORY_ENTRIES)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|entry| {
            json!({
                "role": format!("{:?}", entry.role),
                "content": entry.content,
            })
        })
        .collect();

    let decision = decision_json(context);

    let payload = json!({
        "identity": {
            "name": context.identity.name,
            "bank_name": context.identity.bank_name,
            "role": context.identity.role,
            "communication_style": context.identity.communication_style,
            "tone": context.identity.tone,
            "language": context.identity.language,
            "constraints": context.identity.constraints,
            "conversation_rules": context.identity.conversation_rules,
            "additional_instructions": context.identity.additional_instructions,
        },
        "user_message": context.user_message,
        "decision": decision,
        "client_data": context.client_data,
        "conversation_context": context.conversation_context,
        "history": history,
    });

    serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
}

fn decision_json(context: &ResponseContext) -> serde_json::Value {
    match &context.decision {
        ai_response_engine::ResponseDecisionKind::Flow {
            action,
            current_node,
            node_task,
            node_payload,
        } => json!({
            "type": "flow",
            "action": action,
            "current_node": current_node,
            "node_task": node_task,
            "node_payload": node_payload,
        }),
        ai_response_engine::ResponseDecisionKind::Module {
            module_id,
            section_id,
            title,
            content,
        } => json!({
            "type": "module",
            "module_id": module_id,
            "section_id": section_id,
            "title": title,
            "content": content,
        }),
        ai_response_engine::ResponseDecisionKind::Clarify { reason } => json!({
            "type": "clarify",
            "reason": reason,
        }),
        ai_response_engine::ResponseDecisionKind::EndConversation { reason } => json!({
            "type": "end",
            "reason": reason,
        }),
        ai_response_engine::ResponseDecisionKind::Error { code, message } => json!({
            "type": "error",
            "code": code,
            "message": message,
        }),
    }
}

fn history_json(history: &[ResponseHistoryEntry]) -> String {
    let entries: Vec<_> = history
        .iter()
        .map(|entry| {
            json!({
                "role": format!("{:?}", entry.role),
                "content": entry.content,
            })
        })
        .collect();
    serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string())
}

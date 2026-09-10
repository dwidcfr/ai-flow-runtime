use serde_json::json;

use crate::prompt::PromptBuilder;
use crate::types::ResponseContext;

const MAX_HISTORY_ENTRIES: usize = 10;

pub struct DefaultPromptBuilder;

impl Default for DefaultPromptBuilder {
    fn default() -> Self {
        Self
    }
}

impl DefaultPromptBuilder {
    pub fn new() -> Self {
        Self
    }

    fn decision_json(context: &ResponseContext) -> serde_json::Value {
        match &context.decision {
            crate::types::ResponseDecisionKind::Flow {
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
            crate::types::ResponseDecisionKind::Module {
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
            crate::types::ResponseDecisionKind::Clarify { reason } => json!({
                "type": "clarify",
                "reason": reason,
            }),
            crate::types::ResponseDecisionKind::EndConversation { reason } => json!({
                "type": "end",
                "reason": reason,
            }),
            crate::types::ResponseDecisionKind::Error { code, message } => json!({
                "type": "error",
                "code": code,
                "message": message,
            }),
        }
    }
}

impl PromptBuilder for DefaultPromptBuilder {
    fn build(&self, context: &ResponseContext) -> String {
        let payload = build_context_payload(context);
        format!("Context:\n{payload}")
    }
}

fn build_context_payload(context: &ResponseContext) -> String {
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
        "decision": DefaultPromptBuilder::decision_json(context),
        "client_data": context.client_data,
        "conversation_context": context.conversation_context,
        "history": history,
    });

    serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::identity::AgentIdentity;
    use crate::types::{ResponseDecisionKind, ResponseHistoryEntry, ResponseHistoryRole};

    fn sample_context() -> ResponseContext {
        ResponseContext {
            user_message: "да, это я".to_string(),
            decision: ResponseDecisionKind::Flow {
                action: "confirm".to_string(),
                current_node: "payment".to_string(),
                node_task: Some("Payment Reminder".to_string()),
                node_payload: json!({ "task": "Payment Reminder" }),
            },
            client_data: HashMap::from([(
                "client_name".to_string(),
                json!("Sodiq"),
            )]),
            conversation_context: HashMap::new(),
            identity: AgentIdentity::default_payment_agent(),
            history: vec![ResponseHistoryEntry {
                role: ResponseHistoryRole::User,
                content: "да".to_string(),
            }],
            prompt_set_id: String::new(),
        }
    }

    #[test]
    fn prompt_contains_identity_and_decision() {
        let prompt = DefaultPromptBuilder::new().build(&sample_context());
        assert!(prompt.contains("Alex"));
        assert!(prompt.contains("Acme Insurance"));
        assert!(prompt.contains("Payment Reminder"));
        assert!(prompt.contains("client_name"));
        assert!(prompt.starts_with("Context:\n"));
    }

    #[test]
    fn prompt_context_json_is_valid() {
        let prompt = DefaultPromptBuilder::new().build(&sample_context());
        let start = prompt.find("Context:\n").unwrap() + "Context:\n".len();
        let parsed: serde_json::Value = serde_json::from_str(&prompt[start..]).unwrap();
        assert_eq!(parsed["user_message"], "да, это я");
    }
}

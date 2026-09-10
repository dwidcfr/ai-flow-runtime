use serde_json::json;

use crate::prompt::PromptBuilder;
use crate::types::RouterContext;

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
}

impl PromptBuilder for DefaultPromptBuilder {
    fn build(&self, context: &RouterContext, normalized_message: &str) -> String {
        let payload = build_context_payload(context, normalized_message);
        format!("Context:\n{payload}")
    }
}

fn build_context_payload(context: &RouterContext, normalized_message: &str) -> String {
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

    let search_candidates: Vec<_> = context
        .search_candidates
        .iter()
        .map(|candidate| {
            json!({
                "document_id": candidate.document_id,
                "module_id": candidate.module_id,
                "section_id": candidate.section_id,
                "score": candidate.score,
                "title": candidate.title,
                "snippet": candidate.snippet,
            })
        })
        .collect();

    let payload = json!({
        "user_message": normalized_message,
        "current_node": context.current_node,
        "available_actions": context.available_actions,
        "client_data": context.client_data,
        "conversation_context": context.conversation_context,
        "search_candidates": search_candidates,
        "history": history,
    });

    serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::types::{RouterHistoryEntry, RouterHistoryRole, RouterSearchCandidate};

    fn sample_context() -> RouterContext {
        RouterContext {
            user_message: "да, это я".to_string(),
            current_node: "greeting".to_string(),
            available_actions: vec!["confirm".to_string(), "wrong_person".to_string()],
            history: vec![RouterHistoryEntry {
                role: RouterHistoryRole::User,
                content: "привет".to_string(),
            }],
            client_data: HashMap::from([(
                "client_name".to_string(),
                json!("Sodiq"),
            )]),
            conversation_context: HashMap::new(),
            search_candidates: vec![RouterSearchCandidate {
                document_id: "company::payment_methods".to_string(),
                module_id: "company".to_string(),
                section_id: Some("payment_methods".to_string()),
                score: 0.5,
                title: "Способы оплаты".to_string(),
                snippet: "Click Payme".to_string(),
            }],
            prompt_set_id: String::new(),
        }
    }

    #[test]
    fn prompt_contains_context_sections() {
        let builder = DefaultPromptBuilder::new();
        let prompt = builder.build(&sample_context(), "да, это я");

        assert!(prompt.contains("greeting"));
        assert!(prompt.contains("confirm"));
        assert!(prompt.contains("client_name"));
        assert!(prompt.contains("search_candidates"));
        assert!(prompt.starts_with("Context:\n"));
    }

    #[test]
    fn prompt_json_is_valid() {
        let builder = DefaultPromptBuilder::new();
        let prompt = builder.build(&sample_context(), "test");
        let context_start = prompt.find("Context:\n").unwrap() + "Context:\n".len();
        let json_part = &prompt[context_start..];
        let parsed: serde_json::Value = serde_json::from_str(json_part).unwrap();
        assert_eq!(parsed["current_node"], "greeting");
    }
}

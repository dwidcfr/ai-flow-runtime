use serde_json::Value;

use crate::error::{Result, RouterError};
use crate::llm::RouterLLM;

pub struct MockRouterLLM;

impl Default for MockRouterLLM {
    fn default() -> Self {
        Self
    }
}

impl MockRouterLLM {
    pub fn new() -> Self {
        Self
    }

    fn extract_context(prompt: &str) -> Option<Value> {
        let marker = "Context:\n";
        let start = prompt.find(marker)? + marker.len();
        serde_json::from_str(&prompt[start..]).ok()
    }

    fn normalized_message(context: &Value) -> String {
        context["user_message"]
            .as_str()
            .unwrap_or_default()
            .to_lowercase()
    }

    fn available_actions(context: &Value) -> Vec<String> {
        context["available_actions"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn has_candidate(context: &Value, module_id: &str, section_id: &str) -> bool {
        context["search_candidates"]
            .as_array()
            .map(|arr| {
                arr.iter().any(|c| {
                    c["module_id"].as_str() == Some(module_id)
                        && c["section_id"].as_str() == Some(section_id)
                })
            })
            .unwrap_or(false)
    }

    fn decide(context: &Value, message: &str) -> String {
        let actions = Self::available_actions(context);
        let current_node = context["current_node"].as_str().unwrap_or_default();

        if message.contains("до свидания") || message.contains("пока") {
            return r#"{"type":"end","reason":"user goodbye"}"#.to_string();
        }

        if current_node == "greeting" && actions.iter().any(|a| a == "confirm") {
            let is_confirm = message.contains("это я")
                || message.contains("подтверждаю")
                || (message.contains("да") && message.contains("я"));
            if is_confirm {
                return r#"{"type":"flow","action":"confirm","confidence":0.9}"#.to_string();
            }
            if message.contains("не он")
                || message.contains("не она")
                || message.contains("wrong")
                || message.contains("ошиблись")
            {
                return r#"{"type":"flow","action":"wrong_person","confidence":0.9}"#.to_string();
            }
            let is_pickup = message.contains("ало")
                || message.contains("алло")
                || message.contains("привет")
                || message.contains("слушаю")
                || message.contains("здравств")
                || message.contains("добрый")
                || message == "да"
                || message == "я";
            if is_pickup {
                return r#"{"type":"clarify","reason":"awaiting_identity_confirmation"}"#.to_string();
            }
        }

        if (message.contains("да") || message.contains("подтверждаю") || message.contains("это я"))
            && actions.iter().any(|a| a == "confirm")
        {
            return r#"{"type":"flow","action":"confirm","confidence":0.9}"#.to_string();
        }

        if (message.contains("оплатил") || message.contains("paid"))
            && actions.iter().any(|a| a == "paid")
        {
            return r#"{"type":"flow","action":"paid","confidence":0.9}"#.to_string();
        }

        if (message.contains("click") || message.contains("оплат"))
            && Self::has_candidate(context, "company", "payment_methods")
        {
            return r#"{"type":"module","module_id":"company","section_id":"payment_methods","confidence":0.85}"#
                .to_string();
        }

        r#"{"type":"clarify","reason":"ambiguous intent"}"#.to_string()
    }
}

impl RouterLLM for MockRouterLLM {
    fn complete(&self, prompt: &str) -> Result<String> {
        let context = Self::extract_context(prompt).ok_or_else(|| RouterError::LlmError {
            message: "failed to extract context from prompt".to_string(),
        })?;
        let message = Self::normalized_message(&context);
        Ok(Self::decide(&context, &message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::{DefaultPromptBuilder, PromptBuilder};
    use crate::types::{
        RouterContext, RouterHistoryEntry, RouterHistoryRole, RouterSearchCandidate,
    };
    use std::collections::HashMap;

    fn build_prompt(message: &str, actions: Vec<&str>, with_search: bool) -> String {
        let mut candidates = Vec::new();
        if with_search {
            candidates.push(RouterSearchCandidate {
                document_id: "company::payment_methods".to_string(),
                module_id: "company".to_string(),
                section_id: Some("payment_methods".to_string()),
                score: 0.5,
                title: "Pay".to_string(),
                snippet: "Click".to_string(),
            });
        }

        let context = RouterContext {
            user_message: message.to_string(),
            current_node: "greeting".to_string(),
            available_actions: actions.into_iter().map(|a| a.to_string()).collect(),
            history: vec![RouterHistoryEntry {
                role: RouterHistoryRole::User,
                content: message.to_string(),
            }],
            client_data: HashMap::new(),
            conversation_context: HashMap::new(),
            search_candidates: candidates,
            prompt_set_id: String::new(),
        };

        DefaultPromptBuilder::new().build(&context, message)
    }

    #[test]
    fn mock_returns_flow_confirm() {
        let llm = MockRouterLLM::new();
        let prompt = build_prompt("да, это я", vec!["confirm", "wrong_person"], false);
        let raw = llm.complete(&prompt).unwrap();
        assert!(raw.contains(r#""type":"flow"#));
        assert!(raw.contains(r#""action":"confirm"#));
    }

    #[test]
    fn mock_returns_module_for_payment_query() {
        let llm = MockRouterLLM::new();
        let prompt = build_prompt("Click оплата", vec!["paid"], true);
        let raw = llm.complete(&prompt).unwrap();
        assert!(raw.contains(r#""type":"module"#));
        assert!(raw.contains(r#""module_id":"company"#));
    }

    #[test]
    fn mock_returns_clarify_for_ambiguous() {
        let llm = MockRouterLLM::new();
        let prompt = build_prompt("хмм", vec!["confirm"], false);
        let raw = llm.complete(&prompt).unwrap();
        assert!(raw.contains(r#""type":"clarify"#));
    }

    #[test]
    fn mock_returns_end_for_goodbye() {
        let llm = MockRouterLLM::new();
        let prompt = build_prompt("до свидания", vec!["confirm"], false);
        let raw = llm.complete(&prompt).unwrap();
        assert!(raw.contains(r#""type":"end"#));
    }
}

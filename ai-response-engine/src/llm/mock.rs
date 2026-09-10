use serde_json::Value;

use crate::error::{Result, ResponseError};
use crate::llm::ResponseLLM;

pub struct MockResponseLLM;

impl Default for MockResponseLLM {
    fn default() -> Self {
        Self
    }
}

impl MockResponseLLM {
    pub fn new() -> Self {
        Self
    }

    fn extract_context(prompt: &str) -> Option<Value> {
        let marker = "Context:\n";
        let start = prompt.find(marker)? + marker.len();
        serde_json::from_str(&prompt[start..]).ok()
    }

    fn client_name(context: &Value) -> String {
        context["client_data"]["client_name"]
            .as_str()
            .unwrap_or("клиент")
            .to_string()
    }

    fn client_amount(context: &Value) -> String {
        context["client_data"]["amount"]
            .as_str()
            .unwrap_or("задолженность")
            .to_string()
    }

    fn merchant_name(context: &Value) -> String {
        context["client_data"]["merchant_name"]
            .as_str()
            .unwrap_or("компании")
            .to_string()
    }

    fn operator_name(context: &Value) -> String {
        context["identity"]["name"]
            .as_str()
            .unwrap_or("оператор")
            .to_string()
    }

    fn bank_name(context: &Value) -> String {
        context["identity"]["bank_name"]
            .as_str()
            .unwrap_or("банка")
            .to_string()
    }

    fn product_name(context: &Value) -> Option<String> {
        context["client_data"]["product_name"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    }

    fn greeting_identity_text(context: &Value) -> String {
        let client = Self::client_name(context);
        let operator = Self::operator_name(context);
        let bank = Self::bank_name(context);
        format!(
            "Добрый день! Меня зовут {operator}, звоню от банка {bank}. Вы {client}?"
        )
    }

    fn debt_disclosure_text(context: &Value) -> String {
        let amount = Self::client_amount(context);
        let merchant = Self::merchant_name(context);
        let product_part = Self::product_name(context)
            .map(|p| format!(" ({p})"))
            .unwrap_or_default();
        format!(
            "Спасибо за подтверждение. Напоминаю, что у вас задолженность {amount} сум за покупку в {merchant}{product_part}. Готовы ли вы произвести оплату?"
        )
    }

    fn greeting_opening_text(context: &Value) -> String {
        Self::greeting_identity_text(context)
    }

    fn decide(context: &Value) -> String {
        let decision_type = context["decision"]["type"].as_str().unwrap_or("error");

        match decision_type {
            "flow" => {
                let action = context["decision"]["action"]
                    .as_str()
                    .unwrap_or_default();
                let task = context["decision"]["node_task"]
                    .as_str()
                    .or_else(|| context["decision"]["node_payload"]["task"].as_str())
                    .unwrap_or_default();
                let amount = Self::client_amount(context);

                let text = if action == "confirm" {
                    Self::debt_disclosure_text(context)
                } else {
                    match task {
                        "Greeting" if action == "opening" => Self::greeting_opening_text(context),
                        "Greeting" => Self::greeting_identity_text(context),
                        "Payment Reminder" => format!(
                            "Напоминаю, что у вас имеется задолженность в размере {amount}. Готовы ли вы произвести оплату?"
                        ),
                        "Wrong Person" => {
                            "Прошу прощения. Не могли бы вы подтвердить, что я обращаюсь к нужному человеку?"
                                .to_string()
                        }
                        _ => format!("Продолжаем диалог на шаге: {task}."),
                    }
                };

                Self::json_response(&text, false)
            }
            "module" => {
                let content = context["decision"]["content"]
                    .as_str()
                    .unwrap_or_default();
                let text = if content.contains("Click") {
                    "Оплату можно произвести через Click, Payme или Paynet. Реквизиты Click указаны в информации компании."
                        .to_string()
                } else {
                    format!("Вот информация по вашему запросу: {content}")
                };
                Self::json_response(&text, false)
            }
            "clarify" => {
                let reason = context["decision"]["reason"]
                    .as_str()
                    .unwrap_or("ambiguous intent");
                let text = if reason == "awaiting_identity_confirmation" {
                    Self::greeting_identity_text(context)
                } else {
                    format!(
                        "Извините, я не совсем поняла ваш ответ ({reason}). Не могли бы вы уточнить?"
                    )
                };
                Self::json_response(&text, false)
            }
            "end" => Self::json_response(
                "Спасибо за разговор. Всего доброго!",
                true,
            ),
            _ => Self::json_response(
                "Произошла ошибка при обработке запроса. Пожалуйста, повторите.",
                false,
            ),
        }
    }

    fn json_response(text: &str, finish: bool) -> String {
        serde_json::json!({
            "text": text,
            "finish_conversation": finish,
        })
        .to_string()
    }
}

impl ResponseLLM for MockResponseLLM {
    fn complete(&self, prompt: &str) -> Result<String> {
        let context = Self::extract_context(prompt).ok_or_else(|| ResponseError::LlmError {
            message: "failed to extract context from prompt".to_string(),
        })?;
        Ok(Self::decide(&context))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::identity::AgentIdentity;
    use crate::prompt::{DefaultPromptBuilder, PromptBuilder};
    use crate::types::{
        ResponseContext, ResponseDecisionKind, ResponseHistoryEntry, ResponseHistoryRole,
    };
    use std::collections::HashMap;

    fn build_prompt(decision: ResponseDecisionKind) -> String {
        let context = ResponseContext {
            user_message: "test".to_string(),
            decision,
            client_data: HashMap::from([
                ("client_name".to_string(), json!("Sodiq")),
                ("amount".to_string(), json!("1 500 000")),
                ("merchant_name".to_string(), json!("Media Park")),
                ("product_name".to_string(), json!("Samsung Galaxy A54")),
            ]),
            conversation_context: HashMap::new(),
            identity: AgentIdentity::default_payment_agent(),
            history: vec![ResponseHistoryEntry {
                role: ResponseHistoryRole::User,
                content: "test".to_string(),
            }],
            prompt_set_id: String::new(),
        };
        DefaultPromptBuilder::new().build(&context)
    }

    #[test]
    fn mock_flow_greeting_opening_hides_debt_details() {
        let llm = MockResponseLLM::new();
        let prompt = build_prompt(ResponseDecisionKind::Flow {
            action: "opening".to_string(),
            current_node: "greeting".to_string(),
            node_task: Some("Greeting".to_string()),
            node_payload: json!({ "task": "Greeting" }),
        });
        let raw = llm.complete(&prompt).unwrap();
        assert!(raw.contains("Sodiq"));
        assert!(raw.contains("Acme Insurance"));
        assert!(!raw.contains("1 500 000"));
        assert!(!raw.contains("Media Park"));
    }

    #[test]
    fn mock_flow_greeting_confirm_reveals_debt_details() {
        let llm = MockResponseLLM::new();
        let prompt = build_prompt(ResponseDecisionKind::Flow {
            action: "confirm".to_string(),
            current_node: "payment".to_string(),
            node_task: Some("Payment Reminder".to_string()),
            node_payload: json!({ "task": "Payment Reminder" }),
        });
        let raw = llm.complete(&prompt).unwrap();
        assert!(raw.contains("1 500 000"));
        assert!(raw.contains("Media Park"));
    }

    #[test]
    fn mock_flow_greeting() {
        let llm = MockResponseLLM::new();
        let prompt = build_prompt(ResponseDecisionKind::Flow {
            action: "confirm".to_string(),
            current_node: "payment".to_string(),
            node_task: Some("Payment Reminder".to_string()),
            node_payload: json!({ "task": "Payment Reminder" }),
        });
        let raw = llm.complete(&prompt).unwrap();
        assert!(raw.contains("Sodiq") || raw.contains("1 500 000"));
        assert!(raw.contains("Media Park"));
    }

    #[test]
    fn mock_flow_payment_reminder() {
        let llm = MockResponseLLM::new();
        let prompt = build_prompt(ResponseDecisionKind::Flow {
            action: "question".to_string(),
            current_node: "payment".to_string(),
            node_task: Some("Payment Reminder".to_string()),
            node_payload: json!({ "task": "Payment Reminder" }),
        });
        let raw = llm.complete(&prompt).unwrap();
        assert!(raw.contains("1 500 000"));
    }

    #[test]
    fn mock_module_payment_methods() {
        let llm = MockResponseLLM::new();
        let prompt = build_prompt(ResponseDecisionKind::Module {
            module_id: "company".to_string(),
            section_id: Some("payment_methods".to_string()),
            title: "Способы оплаты".to_string(),
            content: "Click Payme Paynet".to_string(),
        });
        let raw = llm.complete(&prompt).unwrap();
        assert!(raw.contains("Click"));
    }

    #[test]
    fn mock_clarify() {
        let llm = MockResponseLLM::new();
        let prompt = build_prompt(ResponseDecisionKind::Clarify {
            reason: "ambiguous".to_string(),
        });
        let raw = llm.complete(&prompt).unwrap();
        assert!(raw.contains("уточнить"));
    }

    #[test]
    fn mock_end_conversation() {
        let llm = MockResponseLLM::new();
        let prompt = build_prompt(ResponseDecisionKind::EndConversation {
            reason: "goodbye".to_string(),
        });
        let raw = llm.complete(&prompt).unwrap();
        assert!(raw.contains(r#""finish_conversation":true"#));
    }
}

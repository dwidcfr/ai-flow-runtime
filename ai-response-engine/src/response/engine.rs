use crate::error::ResponseError;
use crate::llm::ResponseLLM;
use crate::parser::{JsonResponseParser, ResponseParser};
use crate::prompt::{DefaultPromptBuilder, PromptBuilder};
use crate::types::{Response, ResponseContext};

const FALLBACK_TEXT: &str = "Извините, не удалось сформировать ответ. Пожалуйста, повторите.";

pub struct ResponseEngine {
    prompt_builder: Box<dyn PromptBuilder>,
    llm: Box<dyn ResponseLLM>,
    parser: Box<dyn ResponseParser>,
}

impl ResponseEngine {
    pub fn new(llm: Box<dyn ResponseLLM>) -> Self {
        Self::with_prompt_builder(llm, Box::new(DefaultPromptBuilder::new()))
    }

    pub fn with_prompt_builder(
        llm: Box<dyn ResponseLLM>,
        prompt_builder: Box<dyn PromptBuilder>,
    ) -> Self {
        Self {
            prompt_builder,
            llm,
            parser: Box::new(JsonResponseParser::new()),
        }
    }

    pub fn generate(&self, context: ResponseContext) -> crate::error::Result<Response> {
        let prompt = self.prompt_builder.build(&context);

        let raw = match self.llm.complete(&prompt) {
            Ok(raw) => raw,
            Err(ResponseError::LlmError { message }) => {
                return Ok(fallback_response(&message));
            }
            Err(err) => return Err(err),
        };

        match self.parser.parse(&raw) {
            Ok(response) => Ok(response),
            Err(ResponseError::ParseError { message }) => Ok(fallback_response(&message)),
            Err(err) => Err(err),
        }
    }
}

fn fallback_response(detail: &str) -> Response {
    Response {
        text: format!("{FALLBACK_TEXT} ({detail})"),
        finish_conversation: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::AgentIdentity;
    use crate::llm::MockResponseLLM;
    use crate::types::{ResponseDecisionKind, ResponseHistoryEntry, ResponseHistoryRole};
    use std::collections::HashMap;

    fn flow_context(task: &str) -> ResponseContext {
        ResponseContext {
            user_message: "да".to_string(),
            decision: ResponseDecisionKind::Flow {
                action: "confirm".to_string(),
                current_node: "greeting".to_string(),
                node_task: Some(task.to_string()),
                node_payload: serde_json::json!({ "task": task }),
            },
            client_data: HashMap::from([("client_name".to_string(), serde_json::json!("Sodiq"))]),
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
    fn generates_flow_greeting_response() {
        let engine = ResponseEngine::new(Box::new(MockResponseLLM::new()));
        let mut context = flow_context("Greeting");
        if let ResponseDecisionKind::Flow { action, .. } = &mut context.decision {
            *action = "opening".to_string();
        }
        let response = engine.generate(context).unwrap();
        assert!(response.text.contains("Sodiq"));
        assert!(!response.text.contains("задолженность"));
        assert!(!response.finish_conversation);
    }

    #[test]
    fn generates_clarify_response() {
        let engine = ResponseEngine::new(Box::new(MockResponseLLM::new()));
        let context = ResponseContext {
            user_message: "хм".to_string(),
            decision: ResponseDecisionKind::Clarify {
                reason: "ambiguous".to_string(),
            },
            client_data: HashMap::new(),
            conversation_context: HashMap::new(),
            identity: AgentIdentity::default_payment_agent(),
            history: Vec::new(),
            prompt_set_id: String::new(),
        };
        let response = engine.generate(context).unwrap();
        assert!(response.text.contains("уточнить"));
    }

    #[test]
    fn generates_end_conversation_response() {
        let engine = ResponseEngine::new(Box::new(MockResponseLLM::new()));
        let context = ResponseContext {
            user_message: "пока".to_string(),
            decision: ResponseDecisionKind::EndConversation {
                reason: "goodbye".to_string(),
            },
            client_data: HashMap::new(),
            conversation_context: HashMap::new(),
            identity: AgentIdentity::default_payment_agent(),
            history: Vec::new(),
            prompt_set_id: String::new(),
        };
        let response = engine.generate(context).unwrap();
        assert!(response.finish_conversation);
    }
}

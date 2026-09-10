use crate::decision::{DecisionParser, JsonDecisionParser};
use crate::llm::RouterLLM;
use crate::normalizer::{DefaultNormalizer, Normalizer};
use crate::policy::{DefaultPolicyEngine, PolicyEngine};
use crate::prompt::{DefaultPromptBuilder, PromptBuilder};
use crate::types::{RouterContext, RouterDecision};

pub struct RouterEngine {
    normalizer: Box<dyn Normalizer>,
    prompt_builder: Box<dyn PromptBuilder>,
    llm: Box<dyn RouterLLM>,
    parser: Box<dyn DecisionParser>,
    policy: Box<dyn PolicyEngine>,
}

impl RouterEngine {
    pub fn new(llm: Box<dyn RouterLLM>) -> Self {
        Self::with_prompt_builder(llm, Box::new(DefaultPromptBuilder::new()))
    }

    pub fn with_prompt_builder(
        llm: Box<dyn RouterLLM>,
        prompt_builder: Box<dyn PromptBuilder>,
    ) -> Self {
        Self {
            normalizer: Box::new(DefaultNormalizer::new()),
            prompt_builder,
            llm,
            parser: Box::new(JsonDecisionParser::new()),
            policy: Box::new(DefaultPolicyEngine::default()),
        }
    }

    pub fn with_policy(mut self, policy: Box<dyn PolicyEngine>) -> Self {
        self.policy = policy;
        self
    }

    pub fn route(&self, context: RouterContext) -> RouterDecision {
        let normalized = self.normalizer.normalize(&context.user_message);
        let prompt = self.prompt_builder.build(&context, &normalized);

        let raw = match self.llm.complete(&prompt) {
            Ok(raw) => raw,
            Err(err) => {
                return RouterDecision::Error {
                    code: "llm_error".to_string(),
                    message: err.to_string(),
                };
            }
        };

        let parsed = match self.parser.parse(&raw) {
            Ok(parsed) => parsed,
            Err(err) => {
                return RouterDecision::Error {
                    code: "parse_error".to_string(),
                    message: err.to_string(),
                };
            }
        };

        self.policy.apply(parsed, &context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::MockRouterLLM;
    use crate::types::{RouterHistoryEntry, RouterHistoryRole, RouterSearchCandidate};
    use std::collections::HashMap;

    fn greeting_context(message: &str, with_search: bool) -> RouterContext {
        let mut candidates = Vec::new();
        if with_search {
            candidates.push(RouterSearchCandidate {
                document_id: "company::payment_methods".to_string(),
                module_id: "company".to_string(),
                section_id: Some("payment_methods".to_string()),
                score: 0.5,
                title: "Способы оплаты".to_string(),
                snippet: "Click Payme".to_string(),
            });
        }

        RouterContext {
            user_message: message.to_string(),
            current_node: "greeting".to_string(),
            available_actions: vec!["confirm".to_string(), "wrong_person".to_string()],
            history: vec![RouterHistoryEntry {
                role: RouterHistoryRole::User,
                content: message.to_string(),
            }],
            client_data: HashMap::new(),
            conversation_context: HashMap::new(),
            search_candidates: candidates,
            prompt_set_id: String::new(),
        }
    }

    #[test]
    fn pipeline_returns_flow_decision() {
        let engine = RouterEngine::new(Box::new(MockRouterLLM::new()));
        let decision = engine.route(greeting_context("да, это я", false));
        assert!(matches!(decision, RouterDecision::Flow { .. }));
    }

    #[test]
    fn pipeline_returns_clarify_for_ambiguous() {
        let engine = RouterEngine::new(Box::new(MockRouterLLM::new()));
        let decision = engine.route(greeting_context("хмм", false));
        assert!(matches!(decision, RouterDecision::Clarify { .. }));
    }

    #[test]
    fn pipeline_returns_module_decision() {
        let engine = RouterEngine::new(Box::new(MockRouterLLM::new()));
        let mut context = greeting_context("Click оплата", true);
        context.available_actions = vec!["paid".to_string()];
        context.current_node = "payment".to_string();
        let decision = engine.route(context);
        assert!(matches!(decision, RouterDecision::Module { .. }));
    }

    #[test]
    fn pipeline_returns_end_conversation() {
        let engine = RouterEngine::new(Box::new(MockRouterLLM::new()));
        let decision = engine.route(greeting_context("до свидания", false));
        assert!(matches!(decision, RouterDecision::EndConversation { .. }));
    }
}

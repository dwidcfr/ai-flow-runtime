use crate::policy::{PolicyConfig, PolicyEngine};
use crate::types::{ParsedDecision, RouterContext, RouterDecision};

pub struct DefaultPolicyEngine {
    config: PolicyConfig,
}

impl Default for DefaultPolicyEngine {
    fn default() -> Self {
        Self::new(PolicyConfig::default())
    }
}

impl DefaultPolicyEngine {
    pub fn new(config: PolicyConfig) -> Self {
        Self { config }
    }

    fn find_matching_flow_action(context: &RouterContext, message: &str) -> Option<String> {
        let lower = message.to_lowercase();
        if (lower.contains("да") || lower.contains("подтверждаю") || lower.contains("это я"))
            && context.available_actions.iter().any(|a| a == "confirm")
        {
            return Some("confirm".to_string());
        }
        if (lower.contains("оплатил") || lower.contains("paid"))
            && context.available_actions.iter().any(|a| a == "paid")
        {
            return Some("paid".to_string());
        }
        None
    }

    fn candidate_score(
        context: &RouterContext,
        module_id: &str,
        section_id: Option<&str>,
    ) -> Option<f32> {
        context
            .search_candidates
            .iter()
            .find(|c| {
                c.module_id == module_id
                    && (c.section_id.as_deref() == section_id
                        || (section_id.is_none() && c.section_id.is_none()))
            })
            .map(|c| c.score)
    }
}

impl PolicyEngine for DefaultPolicyEngine {
    fn apply(&self, parsed: ParsedDecision, context: &RouterContext) -> RouterDecision {
        match parsed {
            ParsedDecision::Flow { action, confidence } => {
                if !context.available_actions.iter().any(|a| a == &action) {
                    return RouterDecision::Error {
                        code: "invalid_flow_action".to_string(),
                        message: format!("action '{action}' is not available at current node"),
                    };
                }
                if confidence < self.config.min_flow_confidence {
                    return RouterDecision::Clarify {
                        reason: "low flow confidence".to_string(),
                    };
                }
                RouterDecision::Flow { action, confidence }
            }
            ParsedDecision::Module {
                module_id,
                section_id,
                confidence,
            } => {
                if context.search_candidates.is_empty() {
                    return RouterDecision::Clarify {
                        reason: "no search candidates available".to_string(),
                    };
                }

                if self.config.prefer_flow_on_match {
                    if let Some(action) =
                        Self::find_matching_flow_action(context, &context.user_message)
                    {
                        return RouterDecision::Flow {
                            action,
                            confidence: self.config.min_flow_confidence,
                        };
                    }
                }

                let score = Self::candidate_score(context, &module_id, section_id.as_deref());
                match score {
                    Some(s) if s >= self.config.min_module_score && confidence >= self.config.min_flow_confidence => {
                        RouterDecision::Module {
                            module_id,
                            section_id,
                            confidence,
                        }
                    }
                    _ => RouterDecision::Clarify {
                        reason: "low relevance".to_string(),
                    },
                }
            }
            ParsedDecision::Clarify { reason } => RouterDecision::Clarify { reason },
            ParsedDecision::End { reason } => RouterDecision::EndConversation { reason },
            ParsedDecision::Error { code, message } => RouterDecision::Error { code, message },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::types::RouterSearchCandidate;

    fn base_context(actions: Vec<&str>, candidates: Vec<RouterSearchCandidate>) -> RouterContext {
        RouterContext {
            user_message: "test".to_string(),
            current_node: "greeting".to_string(),
            available_actions: actions.into_iter().map(|a| a.to_string()).collect(),
            history: Vec::new(),
            client_data: HashMap::new(),
            conversation_context: HashMap::new(),
            search_candidates: candidates,
            prompt_set_id: String::new(),
        }
    }

    #[test]
    fn rejects_invalid_flow_action() {
        let engine = DefaultPolicyEngine::default();
        let context = base_context(vec!["confirm"], Vec::new());
        let decision = engine.apply(
            ParsedDecision::Flow {
                action: "unknown".to_string(),
                confidence: 0.9,
            },
            &context,
        );
        assert!(matches!(decision, RouterDecision::Error { .. }));
    }

    #[test]
    fn low_score_module_becomes_clarify() {
        let engine = DefaultPolicyEngine::default();
        let context = base_context(
            vec!["paid"],
            vec![RouterSearchCandidate {
                document_id: "company::payment_methods".to_string(),
                module_id: "company".to_string(),
                section_id: Some("payment_methods".to_string()),
                score: 0.01,
                title: "t".to_string(),
                snippet: "s".to_string(),
            }],
        );
        let decision = engine.apply(
            ParsedDecision::Module {
                module_id: "company".to_string(),
                section_id: Some("payment_methods".to_string()),
                confidence: 0.9,
            },
            &context,
        );
        assert!(matches!(decision, RouterDecision::Clarify { .. }));
    }

    #[test]
    fn module_without_search_candidates_becomes_clarify() {
        let engine = DefaultPolicyEngine::default();
        let context = base_context(vec!["confirm"], Vec::new());
        let decision = engine.apply(
            ParsedDecision::Module {
                module_id: "company".to_string(),
                section_id: Some("payment_methods".to_string()),
                confidence: 0.9,
            },
            &context,
        );
        assert!(matches!(decision, RouterDecision::Clarify { .. }));
    }

    #[test]
    fn valid_module_passes() {
        let engine = DefaultPolicyEngine::default();
        let mut context = base_context(
            vec!["paid"],
            vec![RouterSearchCandidate {
                document_id: "company::payment_methods".to_string(),
                module_id: "company".to_string(),
                section_id: Some("payment_methods".to_string()),
                score: 0.5,
                title: "t".to_string(),
                snippet: "s".to_string(),
            }],
        );
        context.user_message = "xyz unrelated".to_string();
        let decision = engine.apply(
            ParsedDecision::Module {
                module_id: "company".to_string(),
                section_id: Some("payment_methods".to_string()),
                confidence: 0.85,
            },
            &context,
        );
        assert!(matches!(decision, RouterDecision::Module { .. }));
    }

    #[test]
    fn prefer_flow_when_message_matches() {
        let engine = DefaultPolicyEngine::default();
        let mut context = base_context(
            vec!["confirm"],
            vec![RouterSearchCandidate {
                document_id: "company::payment_methods".to_string(),
                module_id: "company".to_string(),
                section_id: Some("payment_methods".to_string()),
                score: 0.9,
                title: "t".to_string(),
                snippet: "s".to_string(),
            }],
        );
        context.user_message = "да, это я".to_string();
        let decision = engine.apply(
            ParsedDecision::Module {
                module_id: "company".to_string(),
                section_id: Some("payment_methods".to_string()),
                confidence: 0.9,
            },
            &context,
        );
        assert!(matches!(decision, RouterDecision::Flow { .. }));
    }
}

mod engine;

pub use engine::DefaultPolicyEngine;

use crate::types::{ParsedDecision, RouterContext, RouterDecision};

#[derive(Debug, Clone)]
pub struct PolicyConfig {
    pub min_module_score: f32,
    pub min_flow_confidence: f32,
    pub prefer_flow_on_match: bool,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            min_module_score: 0.15,
            min_flow_confidence: 0.5,
            prefer_flow_on_match: true,
        }
    }
}

pub trait PolicyEngine: Send + Sync {
    fn apply(&self, parsed: ParsedDecision, context: &RouterContext) -> RouterDecision;
}

use std::sync::{Arc, Mutex};

use ai_response_engine::{PromptBuilder as ResponsePromptBuilder, ResponseContext};
use ai_router_engine::{PromptBuilder as RouterPromptBuilder, RouterContext};

use super::capture::LlmCapture;

pub struct TracingRouterPromptBuilder {
    inner: Box<dyn RouterPromptBuilder>,
    capture: Arc<Mutex<LlmCapture>>,
}

impl TracingRouterPromptBuilder {
    pub fn new(inner: Box<dyn RouterPromptBuilder>, capture: Arc<Mutex<LlmCapture>>) -> Self {
        Self { inner, capture }
    }
}

impl RouterPromptBuilder for TracingRouterPromptBuilder {
    fn build(&self, context: &RouterContext, normalized_message: &str) -> String {
        let prompt = self.inner.build(context, normalized_message);
        if let Ok(mut guard) = self.capture.lock() {
            guard.router_prompt = prompt.clone();
        }
        prompt
    }
}

pub struct TracingResponsePromptBuilder {
    inner: Box<dyn ResponsePromptBuilder>,
    capture: Arc<Mutex<LlmCapture>>,
}

impl TracingResponsePromptBuilder {
    pub fn new(inner: Box<dyn ResponsePromptBuilder>, capture: Arc<Mutex<LlmCapture>>) -> Self {
        Self { inner, capture }
    }
}

impl ResponsePromptBuilder for TracingResponsePromptBuilder {
    fn build(&self, context: &ResponseContext) -> String {
        let prompt = self.inner.build(context);
        if let Ok(mut guard) = self.capture.lock() {
            guard.response_prompt = prompt.clone();
        }
        prompt
    }
}

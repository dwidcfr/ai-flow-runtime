use std::sync::{Arc, Mutex};

use ai_response_engine::ResponseLLM;
use ai_router_engine::RouterLLM;

use super::capture::LlmCapture;

pub struct TracingRouterLLM {
    inner: Box<dyn RouterLLM>,
    capture: Arc<Mutex<LlmCapture>>,
}

impl TracingRouterLLM {
    pub fn new(inner: Box<dyn RouterLLM>, capture: Arc<Mutex<LlmCapture>>) -> Self {
        Self { inner, capture }
    }
}

impl RouterLLM for TracingRouterLLM {
    fn complete(&self, prompt: &str) -> ai_router_engine::error::Result<String> {
        let result = self.inner.complete(prompt);
        if let Ok(mut guard) = self.capture.lock() {
            guard.router_raw = result.as_ref().cloned().unwrap_or_default();
        }
        result
    }
}

pub struct TracingResponseLLM {
    inner: Box<dyn ResponseLLM>,
    capture: Arc<Mutex<LlmCapture>>,
}

impl TracingResponseLLM {
    pub fn new(inner: Box<dyn ResponseLLM>, capture: Arc<Mutex<LlmCapture>>) -> Self {
        Self { inner, capture }
    }
}

impl ResponseLLM for TracingResponseLLM {
    fn complete(&self, prompt: &str) -> ai_response_engine::error::Result<String> {
        let result = self.inner.complete(prompt);
        if let Ok(mut guard) = self.capture.lock() {
            guard.response_raw = result.as_ref().cloned().unwrap_or_default();
        }
        result
    }
}

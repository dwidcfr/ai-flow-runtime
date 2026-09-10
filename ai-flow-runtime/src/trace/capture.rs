use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct LlmCapture {
    pub router_prompt: String,
    pub router_raw: String,
    pub response_prompt: String,
    pub response_raw: String,
}

#[derive(Debug, Clone)]
pub struct TraceCapture {
    inner: Arc<Mutex<LlmCapture>>,
}

impl TraceCapture {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(LlmCapture::default())),
        }
    }

    pub fn shared(&self) -> Arc<Mutex<LlmCapture>> {
        Arc::clone(&self.inner)
    }

    pub fn reset(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            *guard = LlmCapture::default();
        }
    }

    pub fn snapshot(&self) -> LlmCapture {
        self.inner
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }
}

impl Default for TraceCapture {
    fn default() -> Self {
        Self::new()
    }
}

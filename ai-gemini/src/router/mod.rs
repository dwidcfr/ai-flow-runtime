use std::sync::Arc;

use ai_router_engine::{Result as RouterResult, RouterError, RouterLLM};

use crate::client::{new_request_id, GeminiClient};
use crate::config::GeminiConfig;
use crate::errors::GeminiError;
use crate::types::router_response_schema;

pub struct GeminiRouterLLM {
    client: Arc<GeminiClient>,
    model: String,
}

impl GeminiRouterLLM {
    pub fn new(config: &GeminiConfig) -> Result<Self, GeminiError> {
        let client = GeminiClient::from_config(config)?;
        Ok(Self {
            client,
            model: config.model_router.clone(),
        })
    }

    fn map_error(err: GeminiError) -> RouterError {
        RouterError::LlmError {
            message: err.to_string(),
        }
    }
}

impl RouterLLM for GeminiRouterLLM {
    fn complete(&self, prompt: &str) -> RouterResult<String> {
        let request_id = new_request_id();
        let schema = router_response_schema();
        self.client
            .generate_json(prompt, &self.model, schema, &request_id)
            .map_err(Self::map_error)
    }
}

use std::sync::Arc;

use ai_response_engine::{Result as ResponseResult, ResponseError, ResponseLLM};

use crate::client::{new_request_id, GeminiClient};
use crate::config::GeminiConfig;
use crate::errors::GeminiError;
use crate::types::response_output_schema;

pub struct GeminiResponseLLM {
    client: Arc<GeminiClient>,
    model: String,
}

impl GeminiResponseLLM {
    pub fn new(config: &GeminiConfig) -> Result<Self, GeminiError> {
        let client = GeminiClient::from_config(config)?;
        Ok(Self {
            client,
            model: config.model_response.clone(),
        })
    }

    fn map_error(err: GeminiError) -> ResponseError {
        ResponseError::LlmError {
            message: err.to_string(),
        }
    }
}

impl ResponseLLM for GeminiResponseLLM {
    fn complete(&self, prompt: &str) -> ResponseResult<String> {
        let request_id = new_request_id();
        let schema = response_output_schema();
        self.client
            .generate_json(prompt, &self.model, schema, &request_id)
            .map_err(Self::map_error)
    }
}

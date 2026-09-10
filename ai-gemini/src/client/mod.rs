use std::sync::Arc;
use std::time::Instant;

use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde_json::Value;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::GeminiConfig;
use crate::errors::{GeminiError, Result};
use crate::retry::RetryPolicy;
use crate::types::{
    sanitize_json_text, ApiErrorBody, Content, GenerateContentRequest, GenerateContentResponse,
    GenerationConfig, Part,
};

pub struct GeminiClient {
    http: Client,
    config: GeminiConfig,
    retry: RetryPolicy,
}

impl GeminiClient {
    pub fn new(config: GeminiConfig) -> Result<Self> {
        if config.api_key.trim().is_empty() {
            return Err(GeminiError::Configuration(
                "api_key must not be empty".to_string(),
            ));
        }

        let http = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| GeminiError::Configuration(format!("failed to build HTTP client: {e}")))?;

        let retry = RetryPolicy::new(config.retry.clone());

        Ok(Self {
            http,
            config,
            retry,
        })
    }

    pub fn from_config(config: &GeminiConfig) -> Result<Arc<Self>> {
        Ok(Arc::new(Self::new(config.clone())?))
    }

    pub fn generate_json(
        &self,
        prompt: &str,
        model: &str,
        schema: Value,
        request_id: &str,
    ) -> Result<String> {
        self.retry
            .execute(|| self.generate_json_once(prompt, model, schema.clone(), request_id))
    }

    fn generate_json_once(
        &self,
        prompt: &str,
        model: &str,
        schema: Value,
        request_id: &str,
    ) -> Result<String> {
        let started = Instant::now();

        if self.config.logging.log_prompts {
            debug!(request_id, model, prompt, "Gemini request prompt");
        } else {
            debug!(
                request_id,
                model,
                prompt_len = prompt.len(),
                "Gemini request"
            );
        }

        let url = format!(
            "{}/models/{model}:generateContent",
            self.config.api_base.trim_end_matches('/')
        );

        let body = GenerateContentRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: GenerationConfig {
                temperature: self.config.temperature,
                max_output_tokens: self.config.max_output_tokens,
                response_mime_type: "application/json".to_string(),
                response_schema: schema,
            },
        };

        let response = self
            .http
            .post(&url)
            .query(&[("key", self.config.api_key.as_str())])
            .json(&body)
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    GeminiError::Timeout
                } else {
                    GeminiError::Network(e.to_string())
                }
            })?;

        let status = response.status();
        let duration_ms = started.elapsed().as_millis();

        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            error!(request_id, model, duration_ms, "Gemini unauthorized");
            return Err(GeminiError::Unauthorized);
        }

        if status == StatusCode::TOO_MANY_REQUESTS {
            warn!(request_id, model, duration_ms, "Gemini rate limited");
            return Err(GeminiError::RateLimited);
        }

        let payload: GenerateContentResponse = response.json().map_err(|e| {
            GeminiError::Network(format!("failed to read response body: {e}"))
        })?;

        if let Some(api_error) = payload.error {
            let message = api_error_message(&api_error);
            let code = api_error.code.unwrap_or(status.as_u16() as i32) as u16;
            error!(
                request_id,
                model,
                duration_ms,
                status = code,
                error = %message,
                "Gemini API error"
            );
            return Err(GeminiError::from_status(code, message));
        }

        if !status.is_success() {
            let message = format!("unexpected HTTP status {status}");
            error!(request_id, model, duration_ms, status = %status, "Gemini HTTP error");
            return Err(GeminiError::from_status(status.as_u16(), message));
        }

        let raw_text = extract_response_text(&payload)?;
        let sanitized = sanitize_json_text(&raw_text);

        serde_json::from_str::<Value>(&sanitized).map_err(|e| {
            error!(
                request_id,
                model,
                duration_ms,
                error = %e,
                "Gemini invalid JSON"
            );
            GeminiError::InvalidJson(e.to_string())
        })?;

        info!(
            request_id,
            model,
            duration_ms,
            response_len = sanitized.len(),
            "Gemini request succeeded"
        );

        Ok(sanitized)
    }
}

fn api_error_message(error: &ApiErrorBody) -> String {
    error
        .message
        .clone()
        .or_else(|| error.status.clone())
        .unwrap_or_else(|| "unknown API error".to_string())
}

fn extract_response_text(response: &GenerateContentResponse) -> Result<String> {
    if let Some(feedback) = &response.prompt_feedback {
        if let Some(reason) = &feedback.block_reason {
            return Err(GeminiError::Api {
                status: 400,
                message: format!("prompt blocked: {reason}"),
            });
        }
    }

    let candidates = response
        .candidates
        .as_ref()
        .filter(|c| !c.is_empty())
        .ok_or(GeminiError::EmptyResponse)?;

    let first = &candidates[0];
    if let Some(reason) = &first.finish_reason {
        if reason == "SAFETY" || reason == "RECITATION" {
            return Err(GeminiError::Api {
                status: 400,
                message: format!("generation blocked: {reason}"),
            });
        }
    }

    let text = first
        .content
        .as_ref()
        .and_then(|c| c.parts.as_ref())
        .and_then(|parts| parts.first())
        .and_then(|p| p.text.as_ref())
        .filter(|t| !t.trim().is_empty())
        .ok_or(GeminiError::EmptyResponse)?;

    Ok(text.clone())
}

pub fn new_request_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::response_output_schema;

    fn test_config() -> GeminiConfig {
        GeminiConfig::new("test-api-key")
    }

    #[test]
    fn client_rejects_empty_api_key() {
        let mut config = test_config();
        config.api_key = String::new();
        assert!(matches!(
            GeminiClient::new(config),
            Err(GeminiError::Configuration(_))
        ));
    }

    #[test]
    fn extract_response_text_from_valid_payload() {
        let payload = GenerateContentResponse {
            candidates: Some(vec![crate::types::Candidate {
                content: Some(crate::types::ContentResponse {
                    parts: Some(vec![crate::types::PartResponse {
                        text: Some(r#"{"text":"hi"}"#.to_string()),
                    }]),
                }),
                finish_reason: Some("STOP".to_string()),
            }]),
            prompt_feedback: None,
            error: None,
        };

        let text = extract_response_text(&payload).unwrap();
        assert_eq!(text, r#"{"text":"hi"}"#);
    }

    #[test]
    fn extract_response_text_empty_candidates() {
        let payload = GenerateContentResponse {
            candidates: None,
            prompt_feedback: None,
            error: None,
        };
        assert!(matches!(
            extract_response_text(&payload),
            Err(GeminiError::EmptyResponse)
        ));
    }

    #[test]
    fn error_from_status_maps_correctly() {
        assert!(matches!(
            GeminiError::from_status(401, "bad key".into()),
            GeminiError::Unauthorized
        ));
        assert!(matches!(
            GeminiError::from_status(429, "slow down".into()),
            GeminiError::RateLimited
        ));
        assert!(matches!(
            GeminiError::from_status(503, "unavailable".into()),
            GeminiError::Api { status: 503, .. }
        ));
    }

    #[test]
    fn response_schema_is_object() {
        let schema = response_output_schema();
        assert_eq!(schema["required"][0], "text");
    }

    #[test]
    fn error_is_retryable() {
        assert!(GeminiError::Network("x".into()).is_retryable());
        assert!(GeminiError::Timeout.is_retryable());
        assert!(GeminiError::RateLimited.is_retryable());
        assert!(GeminiError::Api {
            status: 503,
            message: "down".into()
        }
        .is_retryable());
        assert!(!GeminiError::Unauthorized.is_retryable());
        assert!(!GeminiError::InvalidJson("bad".into()).is_retryable());
    }
}

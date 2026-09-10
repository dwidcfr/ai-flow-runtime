use serde::Deserialize;

use crate::error::{Result, ResponseError};
use crate::types::Response;

pub trait ResponseParser: Send + Sync {
    fn parse(&self, raw: &str) -> Result<Response>;
}

#[derive(Debug, Deserialize)]
struct RawResponse {
    text: Option<String>,
    finish_conversation: Option<bool>,
}

pub struct JsonResponseParser;

impl Default for JsonResponseParser {
    fn default() -> Self {
        Self
    }
}

impl JsonResponseParser {
    pub fn new() -> Self {
        Self
    }
}

impl ResponseParser for JsonResponseParser {
    fn parse(&self, raw: &str) -> Result<Response> {
        let parsed: RawResponse =
            serde_json::from_str(raw.trim()).map_err(|e| ResponseError::ParseError {
                message: format!("invalid JSON: {e}"),
            })?;

        let text = parsed
            .text
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ResponseError::ParseError {
                message: "missing field: text".to_string(),
            })?;

        Ok(Response {
            text,
            finish_conversation: parsed.finish_conversation.unwrap_or(false),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_response() {
        let parser = JsonResponseParser::new();
        let response = parser
            .parse(r#"{"text":"Привет","finish_conversation":false}"#)
            .unwrap();
        assert_eq!(response.text, "Привет");
        assert!(!response.finish_conversation);
    }

    #[test]
    fn invalid_json_fails() {
        let parser = JsonResponseParser::new();
        assert!(parser.parse("not json").is_err());
    }

    #[test]
    fn missing_text_fails() {
        let parser = JsonResponseParser::new();
        assert!(parser.parse(r#"{"finish_conversation":false}"#).is_err());
    }
}

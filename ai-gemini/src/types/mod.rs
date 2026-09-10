use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";

/// JSON schema for router structured output (matches JsonDecisionParser).
pub fn router_response_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "type": {
                "type": "string",
                "enum": ["flow", "module", "clarify", "end", "error"]
            },
            "action": { "type": "string" },
            "module_id": { "type": "string" },
            "section_id": { "type": "string" },
            "confidence": { "type": "number" },
            "reason": { "type": "string" },
            "code": { "type": "string" },
            "message": { "type": "string" }
        },
        "required": ["type"]
    })
}

/// JSON schema for response structured output (matches JsonResponseParser).
pub fn response_output_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "text": { "type": "string" },
            "finish_conversation": { "type": "boolean" }
        },
        "required": ["text", "finish_conversation"]
    })
}

#[derive(Debug, Serialize)]
pub struct GenerateContentRequest {
    pub contents: Vec<Content>,
    #[serde(rename = "generationConfig")]
    pub generation_config: GenerationConfig,
}

#[derive(Debug, Serialize)]
pub struct Content {
    pub parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
pub struct Part {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct GenerationConfig {
    pub temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    pub max_output_tokens: u32,
    #[serde(rename = "responseMimeType")]
    pub response_mime_type: String,
    #[serde(rename = "responseSchema")]
    pub response_schema: Value,
}

#[derive(Debug, Deserialize)]
pub struct GenerateContentResponse {
    pub candidates: Option<Vec<Candidate>>,
    #[serde(rename = "promptFeedback")]
    pub prompt_feedback: Option<PromptFeedback>,
    pub error: Option<ApiErrorBody>,
}

#[derive(Debug, Deserialize)]
pub struct Candidate {
    pub content: Option<ContentResponse>,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContentResponse {
    pub parts: Option<Vec<PartResponse>>,
}

#[derive(Debug, Deserialize)]
pub struct PartResponse {
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PromptFeedback {
    #[serde(rename = "blockReason")]
    pub block_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorBody {
    pub code: Option<i32>,
    pub message: Option<String>,
    pub status: Option<String>,
}

/// Strip markdown JSON fences and return clean JSON string.
pub fn sanitize_json_text(raw: &str) -> String {
    let trimmed = raw.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }

    let without_open = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed);
    let inner = without_open.trim();
    if let Some(end) = inner.rfind("```") {
        inner[..end].trim().to_string()
    } else {
        inner.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_plain_json() {
        let raw = r#"{"type":"flow","action":"confirm"}"#;
        assert_eq!(sanitize_json_text(raw), raw);
    }

    #[test]
    fn sanitize_fenced_json() {
        let raw = "```json\n{\"text\":\"hi\",\"finish_conversation\":false}\n```";
        assert_eq!(
            sanitize_json_text(raw),
            r#"{"text":"hi","finish_conversation":false}"#
        );
    }

    #[test]
    fn router_schema_is_valid_json() {
        let schema = router_response_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["type"]["enum"].is_array());
    }
}

use serde::Deserialize;

use crate::decision::DecisionParser;
use crate::error::{Result, RouterError};
use crate::types::ParsedDecision;

#[derive(Debug, Deserialize)]
struct RawDecision {
    #[serde(rename = "type")]
    decision_type: String,
    action: Option<String>,
    module_id: Option<String>,
    section_id: Option<String>,
    confidence: Option<f32>,
    reason: Option<String>,
    code: Option<String>,
    message: Option<String>,
}

pub struct JsonDecisionParser;

impl Default for JsonDecisionParser {
    fn default() -> Self {
        Self
    }
}

impl JsonDecisionParser {
    pub fn new() -> Self {
        Self
    }
}

impl DecisionParser for JsonDecisionParser {
    fn parse(&self, raw: &str) -> Result<ParsedDecision> {
        let trimmed = raw.trim();
        let parsed: RawDecision = serde_json::from_str(trimmed).map_err(|e| RouterError::ParseError {
            message: format!("invalid JSON: {e}"),
        })?;

        match parsed.decision_type.as_str() {
            "flow" => {
                let action = parsed
                    .action
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| RouterError::ParseError {
                        message: "missing field: action".to_string(),
                    })?;
                Ok(ParsedDecision::Flow {
                    action,
                    confidence: parsed.confidence.unwrap_or(0.0),
                })
            }
            "module" => {
                let module_id = parsed
                    .module_id
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| RouterError::ParseError {
                        message: "missing field: module_id".to_string(),
                    })?;
                Ok(ParsedDecision::Module {
                    module_id,
                    section_id: parsed.section_id,
                    confidence: parsed.confidence.unwrap_or(0.0),
                })
            }
            "clarify" => Ok(ParsedDecision::Clarify {
                reason: parsed
                    .reason
                    .unwrap_or_else(|| "clarification needed".to_string()),
            }),
            "end" => Ok(ParsedDecision::End {
                reason: parsed
                    .reason
                    .unwrap_or_else(|| "conversation ended".to_string()),
            }),
            "error" => Ok(ParsedDecision::Error {
                code: parsed
                    .code
                    .unwrap_or_else(|| "unknown_error".to_string()),
                message: parsed
                    .message
                    .unwrap_or_else(|| "unknown error".to_string()),
            }),
            other => Err(RouterError::ParseError {
                message: format!("unknown decision type: {other}"),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flow_decision() {
        let parser = JsonDecisionParser::new();
        let parsed = parser
            .parse(r#"{"type":"flow","action":"confirm","confidence":0.9}"#)
            .unwrap();
        assert_eq!(
            parsed,
            ParsedDecision::Flow {
                action: "confirm".to_string(),
                confidence: 0.9
            }
        );
    }

    #[test]
    fn parses_module_decision() {
        let parser = JsonDecisionParser::new();
        let parsed = parser
            .parse(
                r#"{"type":"module","module_id":"company","section_id":"payment_methods","confidence":0.8}"#,
            )
            .unwrap();
        assert!(matches!(parsed, ParsedDecision::Module { .. }));
    }

    #[test]
    fn invalid_json_fails() {
        let parser = JsonDecisionParser::new();
        assert!(parser.parse("not json").is_err());
    }

    #[test]
    fn missing_action_fails() {
        let parser = JsonDecisionParser::new();
        assert!(parser.parse(r#"{"type":"flow"}"#).is_err());
    }

    #[test]
    fn unknown_type_fails() {
        let parser = JsonDecisionParser::new();
        assert!(parser.parse(r#"{"type":"unknown"}"#).is_err());
    }
}

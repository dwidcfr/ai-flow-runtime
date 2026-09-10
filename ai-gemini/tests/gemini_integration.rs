use std::time::Duration;

use ai_router_engine::{DecisionParser, JsonDecisionParser, RouterLLM};
use ai_response_engine::{JsonResponseParser, ResponseLLM, ResponseParser};
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

use ai_gemini::{
    GeminiClient, GeminiConfig, GeminiError, GeminiResponseLLM, GeminiRouterLLM,
};
use ai_gemini::config::RetryConfig;
use ai_gemini::types::response_output_schema;

fn require_api_key() -> Option<String> {
    std::env::var("GEMINI_API_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty())
}

fn mock_config(api_base: &str) -> GeminiConfig {
    let mut config = GeminiConfig::new("test-key");
    config.api_base = api_base.to_string();
    config.model_router = "gemini-test".to_string();
    config.model_response = "gemini-test".to_string();
    config.retry = RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(20),
    };
    config
}

#[test]
fn router_llm_returns_parseable_flow_json_with_live_api() {
    let Some(api_key) = require_api_key() else {
        eprintln!("Skipping live test: GEMINI_API_KEY not set");
        return;
    };

    let config = GeminiConfig::new(api_key);
    let llm = GeminiRouterLLM::new(&config).expect("create router llm");

    let prompt = r#"You are a routing decision engine. Analyze the context and respond with strict JSON only.
Allowed response types:
{"type":"flow","action":"<action>","confidence":0.0-1.0}

Context:
{"user_message":"да, это я","current_node":"greeting","available_actions":["confirm","wrong_person"],"client_data":{},"conversation_context":{},"search_candidates":[],"history":[]}"#;

    let raw = llm.complete(prompt).expect("router complete");
    let parser = JsonDecisionParser::new();
    let parsed = parser.parse(&raw).expect("parse router json");
    assert!(matches!(parsed, ai_router_engine::ParsedDecision::Flow { .. }));
}

#[test]
fn response_llm_returns_parseable_json_with_live_api() {
    let Some(api_key) = require_api_key() else {
        eprintln!("Skipping live test: GEMINI_API_KEY not set");
        return;
    };

    let config = GeminiConfig::new(api_key);
    let llm = GeminiResponseLLM::new(&config).expect("create response llm");

    let prompt = r#"You are a response generation engine. Generate a natural user-facing reply.
Respond with strict JSON only:
{"text":"<reply>","finish_conversation":false}

Context:
{"identity":{"name":"Алиса","role":"agent","communication_style":"polite","language":"ru","constraints":[],"conversation_rules":[],"additional_instructions":[]},"user_message":"да","decision":{"type":"flow","action":"confirm","current_node":"greeting","node_task":"Greeting","node_payload":{"task":"Greeting"}},"client_data":{"client_name":"Sodiq"},"conversation_context":{},"history":[]}"#;

    let raw = llm.complete(prompt).expect("response complete");
    let parser = JsonResponseParser::new();
    let response = parser.parse(&raw).expect("parse response json");
    assert!(!response.text.is_empty());
}

fn setup_mock_server(
    responses: Vec<(u16, serde_json::Value)>,
) -> (tokio::runtime::Runtime, MockServer) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    let server = rt.block_on(async {
        let server = MockServer::start().await;
        for (status, body) in responses {
            Mock::given(method("POST"))
                .and(path_regex(r"/models/.+:generateContent"))
                .respond_with(ResponseTemplate::new(status).set_body_json(body))
                .mount(&server)
                .await;
        }
        server
    });

    (rt, server)
}

#[test]
fn client_maps_unauthorized_from_mock_server() {
    let (_rt, server) = setup_mock_server(vec![(
        401,
        serde_json::json!({
            "error": { "code": 401, "message": "API key not valid" }
        }),
    )]);

    let config = mock_config(&server.uri());
    let client = GeminiClient::new(config).expect("client");
    let result = client.generate_json(
        "test prompt",
        "gemini-test",
        response_output_schema(),
        "req-1",
    );

    assert!(matches!(result, Err(GeminiError::Unauthorized)));
}

#[test]
fn client_retries_rate_limited_then_succeeds() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    let server = rt.block_on(async {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path_regex(r"/models/.+:generateContent"))
            .respond_with(ResponseTemplate::new(429))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path_regex(r"/models/.+:generateContent"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "candidates": [{
                    "content": {
                        "parts": [{ "text": "{\"text\":\"ok\",\"finish_conversation\":false}" }]
                    },
                    "finishReason": "STOP"
                }]
            })))
            .mount(&server)
            .await;

        server
    });

    let config = mock_config(&server.uri());
    let client = GeminiClient::new(config).expect("client");
    let result = client.generate_json(
        "test prompt",
        "gemini-test",
        response_output_schema(),
        "req-2",
    );

    assert_eq!(
        result.expect("success"),
        r#"{"text":"ok","finish_conversation":false}"#
    );
}

#[test]
fn client_rejects_invalid_json_from_model() {
    let (_rt, server) = setup_mock_server(vec![(
        200,
        serde_json::json!({
            "candidates": [{
                "content": { "parts": [{ "text": "not-json" }] },
                "finishReason": "STOP"
            }]
        }),
    )]);

    let config = mock_config(&server.uri());
    let client = GeminiClient::new(config).expect("client");
    let result = client.generate_json(
        "test prompt",
        "gemini-test",
        response_output_schema(),
        "req-3",
    );

    assert!(matches!(result, Err(GeminiError::InvalidJson(_))));
}

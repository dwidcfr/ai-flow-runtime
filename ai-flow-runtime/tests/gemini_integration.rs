#![cfg(feature = "gemini")]

use std::path::PathBuf;

use ai_flow_runtime::{
    flow::provider::YamlFlowProvider, RouterDecision, Runtime, COMPANY_MODULE_ID,
};

fn data_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn require_api_key() -> Option<()> {
    match std::env::var("GEMINI_API_KEY") {
        Ok(key) if !key.trim().is_empty() => Some(()),
        _ => {
            eprintln!("Skipping gemini e2e: GEMINI_API_KEY not set");
            None
        }
    }
}

fn setup_gemini_runtime() -> (Runtime, String) {
    let mut rt = Runtime::with_gemini_from_env(YamlFlowProvider::new()).expect("gemini runtime");
    let flow_id = rt
        .load_flow(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
        .unwrap();
    let session_id = rt
        .create_session(
            &flow_id,
            data_path("data/clients/sample.json").to_str().unwrap(),
        )
        .unwrap();
    rt.load_session_module(
        &session_id,
        COMPANY_MODULE_ID,
        data_path("data/company/acme.yaml").to_str().unwrap(),
    )
    .unwrap();
    (rt, session_id)
}

#[test]
fn gemini_full_cycle_handle_user_message() {
    let Some(()) = require_api_key() else {
        return;
    };

    let (mut rt, session_id) = setup_gemini_runtime();

    let (decision, response) = rt
        .handle_user_message(&session_id, "да, это я")
        .expect("handle user message");

    assert!(
        matches!(decision, RouterDecision::Flow { .. }
            | RouterDecision::Clarify { .. }
            | RouterDecision::Module { .. }),
        "unexpected decision: {decision:?}"
    );
    assert!(!response.text.is_empty());

    let session = rt.get_session(&session_id).unwrap();
    assert!(session.history.iter().any(|e| e.content.contains("да, это я")));
    assert!(session
        .history
        .iter()
        .any(|e| e.content == response.text));
}

#[test]
fn gemini_runtime_rejects_missing_api_key() {
    let original = std::env::var("GEMINI_API_KEY").ok();
    std::env::remove_var("GEMINI_API_KEY");

    let result = Runtime::with_gemini_from_env(YamlFlowProvider::new());
    assert!(result.is_err());

    if let Some(value) = original {
        std::env::set_var("GEMINI_API_KEY", value);
    }
}

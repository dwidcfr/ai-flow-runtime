use std::path::PathBuf;

use ai_flow_runtime::{
    HistoryRole, RouterDecision, Runtime, RuntimeState, COMPANY_MODULE_ID,
};

fn data_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn setup_session() -> (Runtime, String) {
    let mut rt = Runtime::new();
    let flow_id = rt
        .load_flow(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
        .unwrap();
    let session_id = rt
        .create_session(
            &flow_id,
            data_path("data/clients/sample.json").to_str().unwrap(),
        )
        .unwrap();
    (rt, session_id)
}

#[test]
fn generate_opening_message_on_session_start() {
    let (mut rt, session_id) = setup_session();

    let response = rt.generate_opening_message(&session_id).unwrap();
    assert!(!response.text.is_empty());
    assert!(response.text.contains("Sodiq"));
    assert!(
        response.text.contains("Acme Insurance") || response.text.contains("Alex")
    );
    assert!(!response.text.contains("1 500 000"));
    assert!(!response.text.contains("Media Park"));
    assert!(!response.text.to_lowercase().contains("чем могу помочь"));

    let history = &rt.get_session(&session_id).unwrap().history;
    assert!(history.iter().any(|e| matches!(e.role, HistoryRole::Assistant)));
}

#[test]
fn pickup_phrase_at_greeting_asks_identity() {
    let (mut rt, session_id) = setup_session();
    rt.generate_opening_message(&session_id).unwrap();

    let (decision, response) = rt.handle_user_message(&session_id, "ало").unwrap();
    assert!(matches!(decision, RouterDecision::Clarify { .. }));
    assert!(response.text.contains("Sodiq") || response.text.contains("Вы"));
}

#[test]
fn handle_user_message_returns_greeting_response() {
    let (mut rt, session_id) = setup_session();

    let (decision, response) = rt
        .handle_user_message(&session_id, "да, это я")
        .unwrap();

    assert!(matches!(decision, RouterDecision::Flow { .. }));
    assert!(!response.text.is_empty());
    assert!(!response.finish_conversation);
    assert!(
        response.text.contains("задолженность")
            || response.text.contains("Sodiq")
            || response.text.contains("Media Park")
    );

    let history = &rt.get_session(&session_id).unwrap().history;
    assert!(history.iter().any(|entry| {
        matches!(entry.role, HistoryRole::Assistant) && entry.content == response.text
    }));
}

#[test]
fn module_response_mentions_payment_methods() {
    let (mut rt, session_id) = setup_session();

    rt.load_session_module(
        &session_id,
        COMPANY_MODULE_ID,
        data_path("data/company/acme.yaml").to_str().unwrap(),
    )
    .unwrap();
    rt.goto_transition(&session_id, "confirm").unwrap();

    let (decision, response) = rt
        .handle_user_message(&session_id, "Click оплата")
        .unwrap();

    assert!(matches!(decision, RouterDecision::Module { .. }));
    assert!(response.text.contains("Click") || response.text.contains("Payme"));
}

#[test]
fn clarify_response_recorded_without_state_change() {
    let (mut rt, session_id) = setup_session();

    let node_before = rt.get_session(&session_id).unwrap().current_node.clone();
    let (decision, response) = rt.handle_user_message(&session_id, "хмм").unwrap();

    assert!(matches!(decision, RouterDecision::Clarify { .. }));
    assert!(response.text.contains("уточнить") || response.text.contains("поняла"));
    assert_eq!(
        rt.get_session(&session_id).unwrap().current_node,
        node_before
    );
}

#[test]
fn goodbye_returns_farewell_and_finishes_session() {
    let (mut rt, session_id) = setup_session();

    let (decision, response) = rt
        .handle_user_message(&session_id, "до свидания")
        .unwrap();

    assert!(matches!(decision, RouterDecision::EndConversation { .. }));
    assert!(response.finish_conversation);
    assert_eq!(
        rt.get_session(&session_id).unwrap().runtime_state,
        RuntimeState::Finished
    );
}

#[test]
fn generate_response_does_not_mutate_session() {
    let (rt, session_id) = setup_session();

    let decision = rt.route_user_message(&session_id, "да, это я").unwrap();
    let node_before = rt.get_session(&session_id).unwrap().current_node.clone();

    let _response = rt
        .generate_response(&session_id, "да, это я", &decision)
        .unwrap();

    assert_eq!(
        rt.get_session(&session_id).unwrap().current_node,
        node_before
    );
}

#[test]
fn record_assistant_message_writes_assistant_role() {
    let (mut rt, session_id) = setup_session();

    rt.record_assistant_message(&session_id, "Тестовый ответ")
        .unwrap();

    let history = &rt.get_session(&session_id).unwrap().history;
    assert!(history.iter().any(|entry| {
        entry.content == "Тестовый ответ" && matches!(entry.role, HistoryRole::Assistant)
    }));
}

#[test]
fn get_current_node_payload_returns_task() {
    let (rt, session_id) = setup_session();

    let payload = rt.get_current_node_payload(&session_id).unwrap();
    assert_eq!(payload["task"], "Greeting");
}

#[test]
fn error_decision_generates_fallback_response() {
    let (rt, session_id) = setup_session();

    let decision = RouterDecision::Error {
        code: "test".to_string(),
        message: "test error".to_string(),
    };

    let response = rt
        .generate_response(&session_id, "test", &decision)
        .unwrap();

    assert!(!response.text.is_empty());
}

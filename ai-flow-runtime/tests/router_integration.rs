use std::path::PathBuf;

use ai_flow_runtime::{
    RouterDecision, Runtime, RuntimeState, COMPANY_MODULE_ID,
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
fn route_confirm_at_greeting_returns_flow_decision() {
    let (rt, session_id) = setup_session();

    let decision = rt
        .route_user_message(&session_id, "да, это я")
        .unwrap();

    assert!(matches!(
        decision,
        RouterDecision::Flow {
            action,
            ..
        } if action == "confirm"
    ));
}

#[test]
fn handle_user_message_transitions_to_payment() {
    let (mut rt, session_id) = setup_session();

    let (decision, _response) = rt
        .handle_user_message(&session_id, "да, это я")
        .unwrap();

    assert!(matches!(decision, RouterDecision::Flow { .. }));
    assert_eq!(
        rt.get_session(&session_id).unwrap().current_node.as_deref(),
        Some("payment")
    );
}

#[test]
fn route_user_message_does_not_mutate_session_state() {
    let (rt, session_id) = setup_session();

    let before = rt.get_session(&session_id).unwrap().current_node.clone();
    let _ = rt.route_user_message(&session_id, "да, это я").unwrap();
    let after = rt.get_session(&session_id).unwrap().current_node.clone();

    assert_eq!(before, after);
    assert_eq!(after.as_deref(), Some("greeting"));
}

#[test]
fn handle_user_message_opens_module_for_payment_query() {
    let (mut rt, session_id) = setup_session();

    rt.load_session_module(
        &session_id,
        COMPANY_MODULE_ID,
        data_path("data/company/acme.yaml").to_str().unwrap(),
    )
    .unwrap();
    rt.goto_transition(&session_id, "confirm").unwrap();

    let (decision, _response) = rt
        .handle_user_message(&session_id, "Click оплата")
        .unwrap();

    assert!(matches!(decision, RouterDecision::Module { .. }));
    assert_eq!(
        rt.get_session(&session_id).unwrap().runtime_state,
        RuntimeState::Module
    );
}

#[test]
fn ambiguous_message_returns_clarify() {
    let (rt, session_id) = setup_session();

    let decision = rt.route_user_message(&session_id, "хмм").unwrap();
    assert!(matches!(decision, RouterDecision::Clarify { .. }));
}

#[test]
fn goodbye_ends_conversation() {
    let (mut rt, session_id) = setup_session();

    let (decision, _response) = rt
        .handle_user_message(&session_id, "до свидания")
        .unwrap();

    assert!(matches!(decision, RouterDecision::EndConversation { .. }));
    assert_eq!(
        rt.get_session(&session_id).unwrap().runtime_state,
        RuntimeState::Finished
    );
}

#[test]
fn module_without_search_candidates_becomes_clarify() {
    let (rt, session_id) = setup_session();

    let decision = rt.route_user_message(&session_id, "Click оплата").unwrap();
    assert!(matches!(decision, RouterDecision::Clarify { .. }));
}

#[test]
fn handle_user_message_records_user_message_in_history() {
    let (mut rt, session_id) = setup_session();

    rt.handle_user_message(&session_id, "да, это я").unwrap();

    let history = &rt.get_session(&session_id).unwrap().history;
    assert!(history.iter().any(|entry| {
        entry.content == "да, это я"
            && matches!(entry.role, ai_flow_runtime::HistoryRole::User)
    }));
}

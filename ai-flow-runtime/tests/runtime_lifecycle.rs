use std::path::PathBuf;

use ai_flow_runtime::{
    Runtime, RuntimeError, RuntimeState, COMPANY_MODULE_ID, CONVERSATION_MODULE_ID,
};

fn data_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

#[test]
fn runtime_lifecycle_scenario() {
    let mut rt = Runtime::new();
    let flow_id = rt
        .load_flow(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
        .expect("load flow");
    let sid = rt
        .create_session(
            &flow_id,
            data_path("data/clients/sample.json").to_str().unwrap(),
        )
        .expect("create session");

    assert_eq!(rt.get_session(&sid).unwrap().runtime_state, RuntimeState::Flow);
    assert_eq!(
        rt.get_session(&sid).unwrap().current_node.as_deref(),
        Some("greeting")
    );

    let transitions = rt.available_transitions(&sid).unwrap();
    assert!(transitions.contains(&"confirm".to_string()));

    rt.load_session_module(
        &sid,
        COMPANY_MODULE_ID,
        data_path("data/company/acme.yaml").to_str().unwrap(),
    )
    .unwrap();

    rt.goto_transition(&sid, "confirm").expect("transition to payment");
    assert_eq!(
        rt.get_session(&sid).unwrap().current_node.as_deref(),
        Some("payment")
    );

    rt.pause_flow(&sid).expect("pause flow");
    rt.open_module(&sid, "company.payment_methods")
        .expect("open module");

    assert_eq!(
        rt.get_session(&sid).unwrap().runtime_state,
        RuntimeState::Module
    );
    assert_eq!(
        rt.get_session(&sid).unwrap().paused_node.as_deref(),
        Some("payment")
    );

    rt.close_module(&sid).expect("close module auto-resume");

    assert_eq!(rt.get_session(&sid).unwrap().runtime_state, RuntimeState::Flow);
    assert_eq!(
        rt.get_session(&sid).unwrap().current_node.as_deref(),
        Some("payment")
    );
    assert!(rt.get_session(&sid).unwrap().paused_node.is_none());

    rt.goto_transition(&sid, "paid").expect("transition to end");
    assert_eq!(
        rt.get_session(&sid).unwrap().runtime_state,
        RuntimeState::Finished
    );
}

#[test]
fn open_module_without_pause_is_rejected() {
    let mut rt = Runtime::new();
    let flow_id = rt
        .load_flow(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
        .unwrap();
    let sid = rt
        .create_session(
            &flow_id,
            data_path("data/clients/sample.json").to_str().unwrap(),
        )
        .unwrap();

    let err = rt
        .open_module(&sid, "company.payment_methods")
        .unwrap_err();
    assert!(matches!(err, RuntimeError::PauseRequiredBeforeModule));
}

#[test]
fn invalid_transition_is_rejected() {
    let mut rt = Runtime::new();
    let flow_id = rt
        .load_flow(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
        .unwrap();
    let sid = rt
        .create_session(
            &flow_id,
            data_path("data/clients/sample.json").to_str().unwrap(),
        )
        .unwrap();

    let err = rt.goto_transition(&sid, "nonexistent").unwrap_err();
    assert!(matches!(err, RuntimeError::TransitionNotFound { .. }));
}

#[test]
fn close_module_when_not_open_is_rejected() {
    let mut rt = Runtime::new();
    let flow_id = rt
        .load_flow(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
        .unwrap();
    let sid = rt
        .create_session(
            &flow_id,
            data_path("data/clients/sample.json").to_str().unwrap(),
        )
        .unwrap();

    let err = rt.close_module(&sid).unwrap_err();
    assert!(matches!(err, RuntimeError::InvalidStateTransition { .. }));
}

#[test]
fn resume_flow_from_paused_without_module() {
    let mut rt = Runtime::new();
    let flow_id = rt
        .load_flow(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
        .unwrap();
    let sid = rt
        .create_session(
            &flow_id,
            data_path("data/clients/sample.json").to_str().unwrap(),
        )
        .unwrap();

    rt.goto_transition(&sid, "confirm").unwrap();
    rt.pause_flow(&sid).unwrap();
    rt.resume_flow(&sid).unwrap();

    let session = rt.get_session(&sid).unwrap();
    assert_eq!(session.runtime_state, RuntimeState::Flow);
    assert_eq!(session.current_node.as_deref(), Some("payment"));
    assert!(session.paused_node.is_none());
}

#[test]
fn destroy_session_removes_session() {
    let mut rt = Runtime::new();
    let flow_id = rt
        .load_flow(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
        .unwrap();
    let sid = rt
        .create_session(
            &flow_id,
            data_path("data/clients/sample.json").to_str().unwrap(),
        )
        .unwrap();

    rt.destroy_session(&sid).unwrap();
    assert!(rt.get_session(&sid).is_err());
}

#[test]
fn conversation_cleared_after_finish() {
    let mut rt = Runtime::new();
    let flow_id = rt
        .load_flow(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
        .unwrap();
    let sid = rt
        .create_session(
            &flow_id,
            data_path("data/clients/sample.json").to_str().unwrap(),
        )
        .unwrap();

    rt.set_conversation_entry(&sid, "payment_promise", "вечером")
        .unwrap();
    let content = rt
        .get_module_content(&sid, CONVERSATION_MODULE_ID, Some("payment_promise"))
        .unwrap();
    assert_eq!(content.data["payment_promise"], "вечером");

    rt.finish_session(&sid).unwrap();
    let err = rt
        .get_module_content(&sid, CONVERSATION_MODULE_ID, None)
        .unwrap_err();
    assert!(matches!(err, RuntimeError::ModuleInstanceNotLoaded(_)));
}

#[test]
fn get_company_module_content() {
    let mut rt = Runtime::new();
    let flow_id = rt
        .load_flow(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
        .unwrap();
    let sid = rt
        .create_session(
            &flow_id,
            data_path("data/clients/sample.json").to_str().unwrap(),
        )
        .unwrap();

    rt.load_session_module(
        &sid,
        COMPANY_MODULE_ID,
        data_path("data/company/acme.yaml").to_str().unwrap(),
    )
    .unwrap();

    let content = rt
        .get_module_content(&sid, COMPANY_MODULE_ID, Some("payment_methods"))
        .unwrap();
    assert!(content.data["content"]
        .as_str()
        .unwrap()
        .contains("Click"));
}

#[test]
fn list_modules_includes_builtin_four() {
    let rt = Runtime::new();
    let modules = rt.list_modules();
    assert_eq!(modules.len(), 4);
    let ids: Vec<_> = modules.iter().map(|m| m.id.as_str()).collect();
    assert!(ids.contains(&"client"));
    assert!(ids.contains(&"company"));
    assert!(ids.contains(&"conversation"));
    assert!(ids.contains(&"smalltalk"));
}

#[test]
fn smalltalk_module_load() {
    let mut rt = Runtime::new();
    let flow_id = rt
        .load_flow(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
        .unwrap();
    let sid = rt
        .create_session(
            &flow_id,
            data_path("data/clients/sample.json").to_str().unwrap(),
        )
        .unwrap();

    rt.load_session_module(
        &sid,
        "smalltalk",
        data_path("data/modules/smalltalk.yaml").to_str().unwrap(),
    )
    .unwrap();

    let content = rt
        .get_module_content(&sid, "smalltalk", Some("greetings"))
        .unwrap();
    assert!(content.data["content"]
        .as_str()
        .unwrap()
        .contains("Добрый день"));
}

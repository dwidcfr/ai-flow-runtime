use std::path::PathBuf;

use ai_flow_runtime::Runtime;

fn data_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn setup_demo_session() -> (Runtime, String) {
    let mut runtime = Runtime::new();
    let flow_id = runtime
        .load_flow(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
        .expect("load flow");
    let session_id = runtime
        .create_session_with_prompt_set(
            &flow_id,
            data_path("data/clients/sample.json").to_str().unwrap(),
            "demo",
        )
        .expect("create session");
    (runtime, session_id)
}

#[test]
fn create_session_with_prompt_set_stores_metadata() {
    let (runtime, session_id) = setup_demo_session();
    let session = runtime.get_session(&session_id).expect("get session");
    assert_eq!(
        session
            .get_metadata("prompt_set_id")
            .and_then(|v| v.as_str()),
        Some("demo")
    );
}

#[test]
fn response_context_uses_demo_identity() {
    let (runtime, session_id) = setup_demo_session();

    let router_context = runtime
        .build_router_context(&session_id, "да")
        .expect("router context");
    let decision = runtime
        .route_user_message(&session_id, "да")
        .expect("route");
    let response_context = runtime
        .build_response_context(&session_id, "да", &decision)
        .expect("response context");

    assert_eq!(response_context.identity.name, "Alex");
    assert_eq!(router_context.prompt_set_id, "demo");
}

#[test]
fn reload_prompts_succeeds() {
    let runtime = Runtime::new();
    runtime.reload_prompts().expect("reload prompts");
}

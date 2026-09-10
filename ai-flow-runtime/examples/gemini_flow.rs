use std::path::PathBuf;

use ai_flow_runtime::{
    flow::provider::YamlFlowProvider, Response, RouterDecision, Runtime, RuntimeState,
    COMPANY_MODULE_ID, SMALLTALK_MODULE_ID,
};

fn data_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn main() {
    // Загружает .env из корня проекта (если есть)
    let _ = dotenvy::dotenv();

    let mut runtime = Runtime::with_gemini_from_env(YamlFlowProvider::new())
        .expect("initialize runtime with Gemini (set GEMINI_API_KEY)");

    println!("Registered modules: {:?}", runtime.list_modules());

    let flow_id = runtime
        .load_flow(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
        .expect("load flow");

    let session_id = runtime
        .create_session(
            &flow_id,
            data_path("data/clients/sample.json").to_str().unwrap(),
        )
        .expect("create session");

    runtime
        .load_session_module(
            &session_id,
            COMPANY_MODULE_ID,
            data_path("data/company/acme.yaml").to_str().unwrap(),
        )
        .expect("load company module");

    runtime
        .load_session_module(
            &session_id,
            SMALLTALK_MODULE_ID,
            data_path("data/modules/smalltalk.yaml").to_str().unwrap(),
        )
        .expect("load smalltalk module");

    runtime
        .set_conversation_entry(&session_id, "preferred_payment", "Click")
        .expect("set conversation entry");

    println!("\n=== Gemini session created: {session_id} ===");
    print_session(&runtime, &session_id);

    println!("\n=== Gemini: handle_user_message('да, это я') ===");
    let (router_decision, response) = runtime
        .handle_user_message(&session_id, "да, это я")
        .expect("handle user message");
    print_router_decision(&router_decision);
    print_response(&response);
    print_session(&runtime, &session_id);

    println!("\n=== Gemini: handle_user_message('Как оплатить через Click?') ===");
    let (router_decision, response) = runtime
        .handle_user_message(&session_id, "Как оплатить через Click?")
        .expect("handle payment question");
    print_router_decision(&router_decision);
    print_response(&response);

    println!("\n=== History ===");
    for entry in &runtime.get_session(&session_id).unwrap().history {
        println!(
            "[{}] {:?} {}: {}",
            entry.timestamp.format("%H:%M:%S"),
            entry.role,
            entry.event.as_deref().unwrap_or("message"),
            entry.content
        );
    }
}

fn print_response(response: &Response) {
    println!("  response: {}", response.text);
    println!("  finish:   {}", response.finish_conversation);
}

fn print_router_decision(decision: &RouterDecision) {
    match decision {
        RouterDecision::Flow { action, confidence } => {
            println!("  decision: Flow action={action} confidence={confidence:.2}");
        }
        RouterDecision::Module {
            module_id,
            section_id,
            confidence,
        } => {
            println!(
                "  decision: Module {module_id}.{} confidence={confidence:.2}",
                section_id.as_deref().unwrap_or("-")
            );
        }
        RouterDecision::Clarify { reason } => {
            println!("  decision: Clarify reason={reason}");
        }
        RouterDecision::EndConversation { reason } => {
            println!("  decision: EndConversation reason={reason}");
        }
        RouterDecision::Error { code, message } => {
            println!("  decision: Error {code}: {message}");
        }
    }
}

fn print_session(runtime: &Runtime, session_id: &str) {
    let session = runtime.get_session(session_id).expect("get session");
    println!("  state:      {:?}", session.runtime_state);
    println!("  flow:       {:?}", session.flow_id);
    println!("  node:       {:?}", session.current_node);
    println!("  paused:     {:?}", session.paused_node);
    println!("  module:     {:?}", session.active_module);

    if session.runtime_state == RuntimeState::Finished {
        println!("  (session is finished)");
    }
}

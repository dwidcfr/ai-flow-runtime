use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use ai_prompts::{
    bundled_prompts_root, PromptRegistry, RegistryResponsePromptBuilder, RegistryRouterPromptBuilder,
};
use ai_response_engine::{
    PromptBuilder as ResponsePromptBuilder, ResponseContext, ResponseDecisionKind,
    ResponseHistoryEntry, ResponseHistoryRole,
};
use ai_router_engine::{
    PromptBuilder as RouterPromptBuilder, RouterContext, RouterHistoryEntry, RouterHistoryRole,
};

fn shared_registry() -> Arc<RwLock<PromptRegistry>> {
    Arc::new(RwLock::new(
        PromptRegistry::load(bundled_prompts_root()).expect("registry loads"),
    ))
}

fn sample_router_context(prompt_set_id: &str) -> RouterContext {
    RouterContext {
        user_message: "да".to_string(),
        current_node: "greeting".to_string(),
        available_actions: vec!["confirm".to_string()],
        history: vec![RouterHistoryEntry {
            role: RouterHistoryRole::User,
            content: "да".to_string(),
        }],
        client_data: HashMap::from([(
            "client_name".to_string(),
            serde_json::json!("Sodiq"),
        )]),
        conversation_context: HashMap::new(),
        search_candidates: vec![],
        prompt_set_id: prompt_set_id.to_string(),
    }
}

fn sample_response_context(prompt_set_id: &str, identity_name: &str) -> ResponseContext {
    ResponseContext {
        user_message: "да".to_string(),
        decision: ResponseDecisionKind::Flow {
            action: "confirm".to_string(),
            current_node: "greeting".to_string(),
            node_task: Some("Greeting".to_string()),
            node_payload: serde_json::json!({ "task": "Greeting" }),
        },
        client_data: HashMap::from([(
            "client_name".to_string(),
            serde_json::json!("Sodiq"),
        )]),
        conversation_context: HashMap::new(),
        identity: ai_response_engine::AgentIdentity {
            name: identity_name.to_string(),
            bank_name: String::new(),
            role: "test".to_string(),
            language: "ru".to_string(),
            communication_style: String::new(),
            tone: String::new(),
            constraints: vec![],
            conversation_rules: vec![],
            additional_instructions: String::new(),
        },
        history: vec![ResponseHistoryEntry {
            role: ResponseHistoryRole::User,
            content: "да".to_string(),
        }],
        prompt_set_id: prompt_set_id.to_string(),
    }
}

#[test]
fn router_builder_includes_context_json_block() {
    let builder = RegistryRouterPromptBuilder::new(shared_registry());
    let prompt = builder.build(&sample_router_context("demo"), "да");
    assert!(prompt.contains("Context:\n"));
    assert!(prompt.contains("\"current_node\":\"greeting\""));
    assert!(prompt.contains("routing decision engine"));
}

#[test]
fn response_builder_includes_identity_from_set() {
    let registry = shared_registry();
    let builder = RegistryResponsePromptBuilder::new(Arc::clone(&registry));

    let demo_prompt = builder.build(&sample_response_context("demo", "Alex"));
    assert!(demo_prompt.contains("Alex"));
    assert!(demo_prompt.contains("Context:\n"));
}

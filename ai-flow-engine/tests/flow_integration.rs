use std::path::PathBuf;

use ai_flow_engine::{FlowError, FlowLoader};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("ai-flow-runtime/data/flows/payment_flow.yaml")
}

#[test]
fn load_production_flow_fixture() {
    let flow = FlowLoader::load(fixture_path()).expect("load fixture");
    assert_eq!(flow.id(), "payment_flow");
    assert_eq!(flow.initial_node(), "greeting");
    assert_eq!(flow.meta().name.as_deref(), Some("Payment Reminder"));
    assert_eq!(flow.meta().version, Some(1));
}

#[test]
fn greeting_has_transitions() {
    let flow = FlowLoader::load(fixture_path()).unwrap();
    let actions = flow.available_transitions("greeting").unwrap();
    assert!(actions.contains(&"confirm"));
    assert!(actions.contains(&"wrong_person"));
}

#[test]
fn full_navigation_path() {
    let flow = FlowLoader::load(fixture_path()).unwrap();

    let t1 = flow.transition("greeting", "confirm").unwrap();
    assert_eq!(t1.to, "payment");

    let t2 = flow.transition("payment", "paid").unwrap();
    assert_eq!(t2.to, "end");
    assert!(t2.target_is_end);
}

#[test]
fn missing_initial_fails_validation() {
    let yaml = r#"
id: bad
initial: missing
nodes:
  a:
    type: flow
    transitions:
      go: end
  end:
    type: end
"#;
    let err = FlowLoader::from_str(yaml).unwrap_err();
    assert!(matches!(err, FlowError::Validation(_)));
}

#[test]
fn no_end_node_fails_validation() {
    let yaml = r#"
id: bad
initial: a
nodes:
  a:
    type: flow
    transitions:
      loop: a
"#;
    let err = FlowLoader::from_str(yaml).unwrap_err();
    assert!(matches!(err, FlowError::Validation(_)));
}

#[test]
fn unknown_transition_target_fails_validation() {
    let yaml = r#"
id: bad
initial: a
nodes:
  a:
    type: flow
    transitions:
      go: ghost
  end:
    type: end
"#;
    let err = FlowLoader::from_str(yaml).unwrap_err();
    assert!(matches!(err, FlowError::Validation(_)));
}

#[test]
fn transition_action_not_found() {
    let flow = FlowLoader::load(fixture_path()).unwrap();
    let err = flow.transition("greeting", "nonexistent").unwrap_err();
    assert!(matches!(err, FlowError::TransitionNotFound { .. }));
}

#[test]
fn is_end_detects_end_node() {
    let flow = FlowLoader::load(fixture_path()).unwrap();
    assert!(flow.is_end("end"));
    assert!(!flow.is_end("greeting"));
}

#[test]
fn node_payload_preserved() {
    let flow = FlowLoader::load(fixture_path()).unwrap();
    let greeting = flow.node("greeting").unwrap();
    assert_eq!(greeting.payload["task"], "Greeting");
}

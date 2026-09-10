use ai_flow_runtime::{RuntimeState, Session};

use crate::assertions::AssertionFailure;
use crate::scenario::ExpectedFlow;

pub fn assert_flow(expected: &ExpectedFlow, session: &Session) -> Vec<AssertionFailure> {
    let mut failures = Vec::new();

    if let Some(node) = &expected.current_node {
        let actual = session.current_node.as_deref().unwrap_or("-");
        if actual != node {
            failures.push(failure("flow.current_node", node, actual));
        }
    }

    if let Some(node) = &expected.paused_node {
        let actual = session.paused_node.as_deref().unwrap_or("-");
        if actual != node {
            failures.push(failure("flow.paused_node", node, actual));
        }
    }

    if let Some(state) = &expected.runtime_state {
        let actual = runtime_state_str(session.runtime_state);
        if actual != *state {
            failures.push(failure("flow.runtime_state", state, &actual));
        }
    }

    if expected.finished.unwrap_or(false) {
        if session.runtime_state != RuntimeState::Finished {
            failures.push(failure(
                "flow.finished",
                "true",
                &runtime_state_str(session.runtime_state),
            ));
        }
    }

    failures
}

fn runtime_state_str(state: RuntimeState) -> String {
    format!("{state:?}")
}

fn failure(field: &str, expected: &str, actual: &str) -> AssertionFailure {
    AssertionFailure {
        field: field.into(),
        expected: expected.into(),
        actual: actual.into(),
    }
}

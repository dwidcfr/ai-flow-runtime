use ai_router_engine::RouterDecision;

use crate::assertions::AssertionFailure;
use crate::scenario::ExpectedRouter;

pub fn assert_router(expected: &ExpectedRouter, decision: &RouterDecision) -> Vec<AssertionFailure> {
    let mut failures = Vec::new();

    match (expected.decision_type.as_str(), decision) {
        ("flow", RouterDecision::Flow { action, .. }) => {
            if let Some(exp_action) = &expected.action {
                if action != exp_action {
                    failures.push(failure("router.action", exp_action, action));
                }
            }
        }
        ("module", RouterDecision::Module { module_id, section_id, .. }) => {
            if let Some(exp_module) = &expected.module {
                if module_id != exp_module {
                    failures.push(failure("router.module", exp_module, module_id));
                }
            }
            if let Some(exp_section) = &expected.section {
                let actual = section_id.as_deref().unwrap_or("-");
                if actual != exp_section {
                    failures.push(failure("router.section", exp_section, actual));
                }
            }
        }
        ("clarify", RouterDecision::Clarify { .. }) => {}
        ("end", RouterDecision::EndConversation { .. }) => {}
        ("error", RouterDecision::Error { .. }) => {}
        (exp, actual) => {
            failures.push(failure(
                "router.type",
                exp,
                &format!("{actual:?}"),
            ));
        }
    }

    if expected.decision_type != "error" {
        if let RouterDecision::Error { code, message } = decision {
            failures.push(failure(
                "router.error",
                "no error",
                &format!("{code}: {message}"),
            ));
        }
    }

    failures
}

fn failure(field: &str, expected: &str, actual: &str) -> AssertionFailure {
    AssertionFailure {
        field: field.into(),
        expected: expected.into(),
        actual: actual.into(),
    }
}

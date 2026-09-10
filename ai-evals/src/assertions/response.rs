use ai_flow_runtime::Response;

use crate::assertions::AssertionFailure;
use crate::scenario::ExpectedResponse;

pub fn assert_response(expected: &ExpectedResponse, response: &Response) -> Vec<AssertionFailure> {
    let mut failures = Vec::new();

    if expected.non_empty.unwrap_or(false) && response.text.trim().is_empty() {
        failures.push(AssertionFailure {
            field: "response.non_empty".into(),
            expected: "non-empty text".into(),
            actual: "(empty)".into(),
        });
    }

    for needle in &expected.contains {
        if !contains_ci(&response.text, needle) {
            failures.push(AssertionFailure {
                field: "response.contains".into(),
                expected: format!("contains '{needle}'"),
                actual: truncate(&response.text, 120),
            });
        }
    }

    for needle in &expected.not_contains {
        if contains_ci(&response.text, needle) {
            failures.push(AssertionFailure {
                field: "response.not_contains".into(),
                expected: format!("must not contain '{needle}'"),
                actual: truncate(&response.text, 120),
            });
        }
    }

    if let Some(finish) = expected.finish_conversation {
        if response.finish_conversation != finish {
            failures.push(AssertionFailure {
                field: "response.finish_conversation".into(),
                expected: finish.to_string(),
                actual: response.finish_conversation.to_string(),
            });
        }
    }

    failures
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max).collect::<String>())
    }
}

use ai_flow_runtime::{HistoryRole, Session};

use crate::assertions::AssertionFailure;
use crate::scenario::ExpectedSession;

pub fn assert_session(
    expected: &ExpectedSession,
    session: &Session,
    user_message: &str,
    assistant_text: &str,
) -> Vec<AssertionFailure> {
    let mut failures = Vec::new();

    if let Some(min_len) = expected.history_min_len {
        if session.history.len() < min_len {
            failures.push(AssertionFailure {
                field: "session.history_min_len".into(),
                expected: min_len.to_string(),
                actual: session.history.len().to_string(),
            });
        }
    }

    if expected.user_message_recorded.unwrap_or(false) {
        let found = session.history.iter().any(|e| {
            e.role == HistoryRole::User && e.content.contains(user_message)
        });
        if !found {
            failures.push(AssertionFailure {
                field: "session.user_message_recorded".into(),
                expected: format!("user message '{user_message}'"),
                actual: "not found in history".into(),
            });
        }
    }

    if expected.assistant_message_recorded.unwrap_or(false) {
        let found = session
            .history
            .iter()
            .any(|e| e.role == HistoryRole::Assistant && e.content == assistant_text);
        if !found {
            failures.push(AssertionFailure {
                field: "session.assistant_message_recorded".into(),
                expected: "assistant response in history".into(),
                actual: "not found in history".into(),
            });
        }
    }

    failures
}

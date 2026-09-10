use ai_flow_runtime::{Response, RouterDecision, SearchResult, Session};

use crate::scenario::EvalScenario;

mod flow;
mod response;
mod router;
mod search;
mod session;

pub use flow::assert_flow;
pub use response::assert_response;
pub use router::assert_router;
pub use search::assert_search;
pub use session::assert_session;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AssertionFailure {
    pub field: String,
    pub expected: String,
    pub actual: String,
}

pub fn run_assertions(
    scenario: &EvalScenario,
    decision: &RouterDecision,
    response: &Response,
    session: &Session,
    search_results: Option<&[SearchResult]>,
) -> Vec<AssertionFailure> {
    let mut failures = Vec::new();

    if let Some(expected) = &scenario.expected.router {
        failures.extend(assert_router(expected, decision));
    }
    if let Some(expected) = &scenario.expected.response {
        failures.extend(assert_response(expected, response));
    }
    if let Some(expected) = &scenario.expected.flow {
        failures.extend(assert_flow(expected, session));
    }
    if let Some(expected) = &scenario.expected.search {
        failures.extend(assert_search(
            expected,
            search_results.unwrap_or(&[]),
        ));
    }
    if let Some(expected) = &scenario.expected.session {
        failures.extend(assert_session(
            expected,
            session,
            &scenario.user_message,
            &response.text,
        ));
    }
    if let Some(expected_finish) = scenario.expected.finish_conversation {
        if response.finish_conversation != expected_finish {
            failures.push(AssertionFailure {
                field: "finish_conversation".into(),
                expected: expected_finish.to_string(),
                actual: response.finish_conversation.to_string(),
            });
        }
    }

    failures
}

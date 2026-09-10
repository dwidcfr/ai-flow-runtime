use ai_flow_runtime::SearchResult;

use crate::assertions::AssertionFailure;
use crate::scenario::ExpectedSearch;

pub fn assert_search(
    expected: &ExpectedSearch,
    results: &[SearchResult],
) -> Vec<AssertionFailure> {
    let mut failures = Vec::new();

    let found = results.iter().any(|r| {
        let module_ok = r.module_id == expected.expect.module;
        let section_ok = match &expected.expect.section {
            Some(section) => r.section_id.as_deref() == Some(section.as_str()),
            None => true,
        };
        let doc_ok = match &expected.expect.document_id {
            Some(doc_id) => r.document_id == *doc_id,
            None => true,
        };
        module_ok && section_ok && doc_ok
    });

    if !found {
        let actual: Vec<String> = results
            .iter()
            .map(|r| {
                format!(
                    "{}.{}",
                    r.module_id,
                    r.section_id.as_deref().unwrap_or("-")
                )
            })
            .collect();
        failures.push(AssertionFailure {
            field: "search.top_k".into(),
            expected: format!(
                "{}.{} in top {}",
                expected.expect.module,
                expected.expect.section.as_deref().unwrap_or("*"),
                expected.top_k
            ),
            actual: if actual.is_empty() {
                "(no results)".into()
            } else {
                actual.join(", ")
            },
        });
    }

    failures
}

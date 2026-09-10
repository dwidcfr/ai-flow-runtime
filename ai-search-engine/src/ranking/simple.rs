use crate::ranking::Ranker;
use crate::types::{ScoredCandidate, SearchResult};

const SNIPPET_MAX_LEN: usize = 120;

pub struct SimpleRanker;

impl Default for SimpleRanker {
    fn default() -> Self {
        Self
    }
}

impl SimpleRanker {
    pub fn new() -> Self {
        Self
    }

    fn make_snippet(content: &str) -> String {
        let trimmed = content.trim();
        if trimmed.chars().count() <= SNIPPET_MAX_LEN {
            return trimmed.to_string();
        }
        trimmed
            .chars()
            .take(SNIPPET_MAX_LEN)
            .collect::<String>()
            .trim()
            .to_string()
            + "..."
    }
}

impl Ranker for SimpleRanker {
    fn rank(
        &self,
        _query: &str,
        mut candidates: Vec<ScoredCandidate>,
        top_k: usize,
    ) -> Vec<SearchResult> {
        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(top_k);

        candidates
            .into_iter()
            .map(|candidate| SearchResult {
                document_id: candidate.document.document_id.clone(),
                module_id: candidate.document.module_id.clone(),
                section_id: candidate.document.section_id.clone(),
                score: candidate.score,
                title: candidate.document.title.clone(),
                snippet: Self::make_snippet(&candidate.document.content),
                metadata: candidate.document.metadata.clone(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::types::SearchDocument;

    fn candidate(id: &str, score: f32, content: &str) -> ScoredCandidate {
        ScoredCandidate {
            document: SearchDocument {
                document_id: id.to_string(),
                module_id: "company".to_string(),
                section_id: Some("section".to_string()),
                title: "Title".to_string(),
                content: content.to_string(),
                metadata: HashMap::new(),
            },
            score,
        }
    }

    #[test]
    fn sorts_by_score_descending() {
        let ranker = SimpleRanker::new();
        let results = ranker.rank(
            "query",
            vec![
                candidate("low", 0.1, "a"),
                candidate("high", 0.9, "b"),
                candidate("mid", 0.5, "c"),
            ],
            3,
        );
        assert_eq!(results[0].document_id, "high");
        assert_eq!(results[1].document_id, "mid");
        assert_eq!(results[2].document_id, "low");
    }

    #[test]
    fn respects_top_k() {
        let ranker = SimpleRanker::new();
        let results = ranker.rank(
            "query",
            vec![candidate("a", 0.9, "a"), candidate("b", 0.8, "b")],
            1,
        );
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn generates_snippet() {
        let ranker = SimpleRanker::new();
        let long_content = "a".repeat(200);
        let results = ranker.rank("query", vec![candidate("doc", 1.0, &long_content)], 1);
        assert!(results[0].snippet.ends_with("..."));
        assert!(results[0].snippet.chars().count() <= SNIPPET_MAX_LEN + 3);
    }
}

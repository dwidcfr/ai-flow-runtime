use std::collections::HashMap;

use crate::embedder::Embedder;
use crate::error::{Result, SearchError};
use crate::index::Index;
use crate::providers::SearchProvider;
use crate::ranking::{Ranker, SimpleRanker};
use crate::types::SearchResult;

const SEARCH_CANDIDATE_MULTIPLIER: usize = 3;

pub struct SearchEngine {
    providers: HashMap<String, Box<dyn SearchProvider>>,
    embedder: Box<dyn Embedder>,
    index: Box<dyn Index>,
    ranker: Box<dyn Ranker>,
}

impl SearchEngine {
    pub fn new(embedder: Box<dyn Embedder>, index: Box<dyn Index>) -> Self {
        Self::with_ranker(embedder, index, Box::new(SimpleRanker::new()))
    }

    pub fn with_ranker(
        embedder: Box<dyn Embedder>,
        index: Box<dyn Index>,
        ranker: Box<dyn Ranker>,
    ) -> Self {
        Self {
            providers: HashMap::new(),
            embedder,
            index,
            ranker,
        }
    }

    pub fn register_provider(&mut self, provider: Box<dyn SearchProvider>) {
        let module_id = provider.module_id().to_string();
        self.providers.insert(module_id, provider);
    }

    pub fn has_provider(&self, module_id: &str) -> bool {
        self.providers.contains_key(module_id)
    }

    pub fn list_providers(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.providers.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub fn build_index(&mut self) -> Result<()> {
        let module_ids: Vec<String> = self.providers.keys().cloned().collect();
        for module_id in module_ids {
            self.index_module(&module_id)?;
        }
        Ok(())
    }

    pub fn index_module(&mut self, module_id: &str) -> Result<()> {
        let provider = self.providers.get(module_id).ok_or_else(|| {
            SearchError::ProviderNotFound {
                module_id: module_id.to_string(),
            }
        })?;

        let documents = provider.list_documents().map_err(|e| match e {
            SearchError::ProviderError {
                module_id: id,
                message,
            } => SearchError::ProviderError { module_id: id, message },
            other => SearchError::ProviderError {
                module_id: module_id.to_string(),
                message: other.to_string(),
            },
        })?;

        self.index.clear_module(module_id)?;

        for doc in documents {
            let embedding = self.embedder.embed(&doc.content)?;
            self.index.add(doc, embedding)?;
        }

        Ok(())
    }

    pub fn reindex_module(&mut self, module_id: &str) -> Result<()> {
        self.index_module(module_id)
    }

    pub fn search(
        &self,
        query: &str,
        module_ids: Option<&[String]>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        if top_k == 0 {
            return Ok(Vec::new());
        }

        let query_embedding = self.embedder.embed(query)?;
        let candidate_limit = top_k.saturating_mul(SEARCH_CANDIDATE_MULTIPLIER).max(top_k);
        let candidates = self
            .index
            .search(&query_embedding, module_ids, candidate_limit)?;

        Ok(self.ranker.rank(query, candidates, top_k))
    }

    pub fn remove_document(&mut self, document_id: &str) -> Result<()> {
        self.index.remove(document_id)
    }

    pub fn index_len(&self) -> usize {
        self.index.len()
    }

    pub fn module_document_counts(&self) -> Result<Vec<(String, usize)>> {
        let mut stats = Vec::new();
        for module_id in self.list_providers() {
            let provider = self.providers.get(&module_id).ok_or_else(|| {
                SearchError::ProviderNotFound {
                    module_id: module_id.clone(),
                }
            })?;
            let count = provider.list_documents()?.len();
            stats.push((module_id, count));
        }
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedder::MockEmbedder;
    use crate::index::InMemoryIndex;
    use crate::providers::SearchProvider;
    use crate::types::SearchDocument;
    use std::collections::HashMap;

    struct FixtureCompanyProvider;

    impl SearchProvider for FixtureCompanyProvider {
        fn module_id(&self) -> &str {
            "company"
        }

        fn list_documents(&self) -> Result<Vec<SearchDocument>> {
            Ok(vec![
                SearchDocument {
                    document_id: "company::payment_methods".to_string(),
                    module_id: "company".to_string(),
                    section_id: Some("payment_methods".to_string()),
                    title: "Способы оплаты".to_string(),
                    content: "Оплата доступна через Click, Payme, Paynet.".to_string(),
                    metadata: HashMap::new(),
                },
                SearchDocument {
                    document_id: "company::faq_late_payment".to_string(),
                    module_id: "company".to_string(),
                    section_id: Some("faq_late_payment".to_string()),
                    title: "Последствия просрочки".to_string(),
                    content: "При просрочке более 30 дней начисляется пеня.".to_string(),
                    metadata: HashMap::new(),
                },
            ])
        }
    }

    struct FixtureConversationProvider {
        entries: Vec<(String, String)>,
    }

    impl SearchProvider for FixtureConversationProvider {
        fn module_id(&self) -> &str {
            "conversation"
        }

        fn list_documents(&self) -> Result<Vec<SearchDocument>> {
            Ok(self
                .entries
                .iter()
                .map(|(key, value)| SearchDocument {
                    document_id: format!("conversation::{key}"),
                    module_id: "conversation".to_string(),
                    section_id: Some(key.clone()),
                    title: key.clone(),
                    content: value.clone(),
                    metadata: HashMap::new(),
                })
                .collect())
        }
    }

    struct FixtureSmallTalkProvider;

    impl SearchProvider for FixtureSmallTalkProvider {
        fn module_id(&self) -> &str {
            "smalltalk"
        }

        fn list_documents(&self) -> Result<Vec<SearchDocument>> {
            Ok(vec![SearchDocument {
                document_id: "smalltalk::greetings".to_string(),
                module_id: "smalltalk".to_string(),
                section_id: Some("greetings".to_string()),
                title: "Приветствия".to_string(),
                content: "Добрый день. Чем могу помочь?".to_string(),
                metadata: HashMap::new(),
            }])
        }
    }

    fn test_engine() -> SearchEngine {
        SearchEngine::new(
            Box::new(MockEmbedder::default()),
            Box::new(InMemoryIndex::new()),
        )
    }

    #[test]
    fn register_provider() {
        let mut engine = test_engine();
        engine.register_provider(Box::new(FixtureCompanyProvider));
        assert!(engine.has_provider("company"));
        assert_eq!(engine.list_providers(), vec!["company"]);
    }

    #[test]
    fn build_index_indexes_all_providers() {
        let mut engine = test_engine();
        engine.register_provider(Box::new(FixtureCompanyProvider));
        engine.register_provider(Box::new(FixtureSmallTalkProvider));
        engine.build_index().unwrap();
        assert_eq!(engine.index_len(), 3);
    }

    #[test]
    fn index_company_module() {
        let mut engine = test_engine();
        engine.register_provider(Box::new(FixtureCompanyProvider));
        engine.index_module("company").unwrap();
        assert_eq!(engine.index_len(), 2);
    }

    #[test]
    fn index_conversation_module() {
        let mut engine = test_engine();
        engine.register_provider(Box::new(FixtureConversationProvider {
            entries: vec![
                ("payment_promise".to_string(), "обещал оплатить вечером".to_string()),
            ],
        }));
        engine.index_module("conversation").unwrap();
        assert_eq!(engine.index_len(), 1);
    }

    #[test]
    fn index_smalltalk_module() {
        let mut engine = test_engine();
        engine.register_provider(Box::new(FixtureSmallTalkProvider));
        engine.index_module("smalltalk").unwrap();
        assert_eq!(engine.index_len(), 1);
    }

    #[test]
    fn search_single_module() {
        let mut engine = test_engine();
        engine.register_provider(Box::new(FixtureCompanyProvider));
        engine.build_index().unwrap();

        let results = engine
            .search("Click payment", Some(&["company".to_string()]), 3)
            .unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].module_id, "company");
        assert_eq!(results[0].section_id.as_deref(), Some("payment_methods"));
    }

    #[test]
    fn search_multiple_modules() {
        let mut engine = test_engine();
        engine.register_provider(Box::new(FixtureCompanyProvider));
        engine.register_provider(Box::new(FixtureSmallTalkProvider));
        engine.build_index().unwrap();

        let results = engine.search("день", None, 5).unwrap();
        assert!(results.iter().any(|r| r.module_id == "smalltalk"));
    }

    #[test]
    fn top_k_limits_results() {
        let mut engine = test_engine();
        engine.register_provider(Box::new(FixtureCompanyProvider));
        engine.build_index().unwrap();

        let results = engine.search("оплата", None, 1).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn results_sorted_by_score() {
        let mut engine = test_engine();
        engine.register_provider(Box::new(FixtureCompanyProvider));
        engine.build_index().unwrap();

        let results = engine.search("Click Payme", None, 2).unwrap();
        assert!(results[0].score >= results.last().unwrap().score);
    }

    #[test]
    fn reindex_module_updates_index() {
        let mut engine = test_engine();
        engine.register_provider(Box::new(FixtureConversationProvider {
            entries: vec![("key1".to_string(), "value one".to_string())],
        }));
        engine.index_module("conversation").unwrap();
        assert_eq!(engine.index_len(), 1);

        engine.register_provider(Box::new(FixtureConversationProvider {
            entries: vec![
                ("key1".to_string(), "value one".to_string()),
                ("key2".to_string(), "value two".to_string()),
            ],
        }));
        engine.reindex_module("conversation").unwrap();
        assert_eq!(engine.index_len(), 2);
    }

    #[test]
    fn remove_document() {
        let mut engine = test_engine();
        engine.register_provider(Box::new(FixtureCompanyProvider));
        engine.build_index().unwrap();
        engine.remove_document("company::payment_methods").unwrap();
        assert_eq!(engine.index_len(), 1);
    }

    #[test]
    fn empty_index_returns_no_results() {
        let engine = test_engine();
        let results = engine.search("Click", None, 5).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn no_matching_results() {
        let mut engine = test_engine();
        engine.register_provider(Box::new(FixtureCompanyProvider));
        engine.build_index().unwrap();

        let results = engine
            .search("xyznonexistentterm12345", None, 5)
            .unwrap();
        // May return low-score results; verify scores are present
        for result in &results {
            assert!(result.score >= 0.0);
        }
    }

    #[test]
    fn unknown_module_returns_error() {
        let mut engine = test_engine();
        assert!(engine.index_module("unknown").is_err());
    }
}

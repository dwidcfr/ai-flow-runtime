use std::collections::HashMap;

use crate::error::{Result, SearchError};
use crate::index::Index;
use crate::types::{ScoredCandidate, SearchDocument};

#[derive(Debug, Default)]
pub struct InMemoryIndex {
    documents: HashMap<String, (SearchDocument, Vec<f32>)>,
}

impl InMemoryIndex {
    pub fn new() -> Self {
        Self::default()
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    fn matches_module_filter(module_id: &str, module_filter: Option<&[String]>) -> bool {
        match module_filter {
            None => true,
            Some(filter) if filter.is_empty() => true,
            Some(filter) => filter.iter().any(|id| id == module_id),
        }
    }
}

impl Index for InMemoryIndex {
    fn add(&mut self, doc: SearchDocument, embedding: Vec<f32>) -> Result<()> {
        self.documents
            .insert(doc.document_id.clone(), (doc, embedding));
        Ok(())
    }

    fn remove(&mut self, document_id: &str) -> Result<()> {
        self.documents
            .remove(document_id)
            .ok_or_else(|| SearchError::DocumentNotFound {
                document_id: document_id.to_string(),
            })?;
        Ok(())
    }

    fn update(&mut self, doc: SearchDocument, embedding: Vec<f32>) -> Result<()> {
        if !self.documents.contains_key(&doc.document_id) {
            return Err(SearchError::DocumentNotFound {
                document_id: doc.document_id.clone(),
            });
        }
        self.documents
            .insert(doc.document_id.clone(), (doc, embedding));
        Ok(())
    }

    fn clear_module(&mut self, module_id: &str) -> Result<()> {
        self.documents
            .retain(|_, (doc, _)| doc.module_id != module_id);
        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        self.documents.clear();
        Ok(())
    }

    fn search(
        &self,
        query_embedding: &[f32],
        module_filter: Option<&[String]>,
        limit: usize,
    ) -> Result<Vec<ScoredCandidate>> {
        let mut candidates: Vec<ScoredCandidate> = self
            .documents
            .values()
            .filter(|(doc, _)| Self::matches_module_filter(&doc.module_id, module_filter))
            .map(|(doc, embedding)| ScoredCandidate {
                document: doc.clone(),
                score: Self::cosine_similarity(query_embedding, embedding),
            })
            .collect();

        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(limit);
        Ok(candidates)
    }

    fn len(&self) -> usize {
        self.documents.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn sample_doc(id: &str, module_id: &str, content: &str) -> SearchDocument {
        SearchDocument {
            document_id: id.to_string(),
            module_id: module_id.to_string(),
            section_id: Some("section".to_string()),
            title: "Title".to_string(),
            content: content.to_string(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn add_and_search() {
        let mut index = InMemoryIndex::new();
        let embedding = vec![1.0, 0.0, 0.0];
        index
            .add(sample_doc("doc1", "company", "Click payment"), embedding.clone())
            .unwrap();

        let results = index.search(&embedding, None, 5).unwrap();
        assert_eq!(results.len(), 1);
        assert!((results[0].score - 1.0).abs() < 1e-5);
    }

    #[test]
    fn remove_document() {
        let mut index = InMemoryIndex::new();
        index
            .add(sample_doc("doc1", "company", "text"), vec![1.0, 0.0])
            .unwrap();
        index.remove("doc1").unwrap();
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn update_document() {
        let mut index = InMemoryIndex::new();
        let mut doc = sample_doc("doc1", "company", "old");
        index.add(doc.clone(), vec![1.0, 0.0]).unwrap();
        doc.content = "new content".to_string();
        index.update(doc, vec![0.0, 1.0]).unwrap();
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn clear_module() {
        let mut index = InMemoryIndex::new();
        index
            .add(sample_doc("c1", "company", "a"), vec![1.0])
            .unwrap();
        index
            .add(sample_doc("s1", "smalltalk", "b"), vec![1.0])
            .unwrap();
        index.clear_module("company").unwrap();
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn clear_all() {
        let mut index = InMemoryIndex::new();
        index
            .add(sample_doc("doc1", "company", "a"), vec![1.0])
            .unwrap();
        index.clear().unwrap();
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn empty_index_returns_no_results() {
        let index = InMemoryIndex::new();
        let results = index.search(&[1.0, 0.0], None, 5).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn module_filter() {
        let mut index = InMemoryIndex::new();
        index
            .add(sample_doc("c1", "company", "Click"), vec![1.0, 0.0])
            .unwrap();
        index
            .add(sample_doc("s1", "smalltalk", "Hello"), vec![0.0, 1.0])
            .unwrap();

        let results = index
            .search(&[1.0, 0.0], Some(&["company".to_string()]), 5)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document.module_id, "company");
    }
}

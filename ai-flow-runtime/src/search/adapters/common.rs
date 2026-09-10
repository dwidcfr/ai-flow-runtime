use std::collections::HashMap;

use ai_search_engine::{SearchDocument, SearchError};
use serde_json::Value;

pub fn sectioned_documents(
    module_id: &str,
    content: &Value,
) -> Result<Vec<SearchDocument>, SearchError> {
    let sections = content.as_object().ok_or_else(|| SearchError::ProviderError {
        module_id: module_id.to_string(),
        message: "expected object with sections".to_string(),
    })?;

    let mut documents = Vec::new();
    for (section_id, section_value) in sections {
        let title = section_value
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or(section_id)
            .to_string();
        let content_text = section_value
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        documents.push(SearchDocument {
            document_id: format!("{module_id}::{section_id}"),
            module_id: module_id.to_string(),
            section_id: Some(section_id.clone()),
            title,
            content: content_text,
            metadata: HashMap::new(),
        });
    }

    Ok(documents)
}

pub fn key_value_documents(
    module_id: &str,
    content: &Value,
) -> Result<Vec<SearchDocument>, SearchError> {
    let entries = content.as_object().ok_or_else(|| SearchError::ProviderError {
        module_id: module_id.to_string(),
        message: "expected key-value object".to_string(),
    })?;

    let mut documents = Vec::new();
    for (key, value) in entries {
        let content_text = value.as_str().unwrap_or_default().to_string();
        documents.push(SearchDocument {
            document_id: format!("{module_id}::{key}"),
            module_id: module_id.to_string(),
            section_id: Some(key.clone()),
            title: key.clone(),
            content: content_text,
            metadata: HashMap::new(),
        });
    }

    Ok(documents)
}

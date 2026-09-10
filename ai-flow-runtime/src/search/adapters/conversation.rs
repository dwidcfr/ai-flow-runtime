use ai_search_engine::{Result as SearchResult, SearchDocument, SearchProvider};

use crate::error::Result as RuntimeResult;
use crate::modules::{ModuleInstance, CONVERSATION_MODULE_ID};
use crate::search::adapters::common::key_value_documents;

pub struct ConversationSearchProvider {
    documents: Vec<SearchDocument>,
}

impl ConversationSearchProvider {
    pub fn from_instance(instance: &dyn ModuleInstance) -> RuntimeResult<Self> {
        let content = instance.get_content(None)?;
        let documents =
            key_value_documents(CONVERSATION_MODULE_ID, &content.data).map_err(map_error)?;
        Ok(Self { documents })
    }
}

impl SearchProvider for ConversationSearchProvider {
    fn module_id(&self) -> &str {
        CONVERSATION_MODULE_ID
    }

    fn list_documents(&self) -> SearchResult<Vec<SearchDocument>> {
        Ok(self.documents.clone())
    }
}

fn map_error(err: ai_search_engine::SearchError) -> crate::error::RuntimeError {
    crate::error::RuntimeError::SearchError(err.to_string())
}

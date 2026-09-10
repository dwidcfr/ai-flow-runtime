use ai_search_engine::{Result as SearchResult, SearchDocument, SearchProvider};

use crate::error::Result as RuntimeResult;
use crate::modules::{ModuleInstance, SMALLTALK_MODULE_ID};
use crate::search::adapters::common::sectioned_documents;

pub struct SmallTalkSearchProvider {
    documents: Vec<SearchDocument>,
}

impl SmallTalkSearchProvider {
    pub fn from_instance(instance: &dyn ModuleInstance) -> RuntimeResult<Self> {
        let content = instance.get_content(None)?;
        let documents =
            sectioned_documents(SMALLTALK_MODULE_ID, &content.data).map_err(map_error)?;
        Ok(Self { documents })
    }
}

impl SearchProvider for SmallTalkSearchProvider {
    fn module_id(&self) -> &str {
        SMALLTALK_MODULE_ID
    }

    fn list_documents(&self) -> SearchResult<Vec<SearchDocument>> {
        Ok(self.documents.clone())
    }
}

fn map_error(err: ai_search_engine::SearchError) -> crate::error::RuntimeError {
    crate::error::RuntimeError::SearchError(err.to_string())
}

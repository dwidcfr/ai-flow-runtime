use crate::error::Result;
use crate::types::SearchDocument;

pub trait SearchProvider: Send + Sync {
    fn module_id(&self) -> &str;
    fn list_documents(&self) -> Result<Vec<SearchDocument>>;
}

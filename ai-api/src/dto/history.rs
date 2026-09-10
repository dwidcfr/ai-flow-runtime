use serde::Serialize;
use utoipa::ToSchema;

use ai_flow_runtime::HistoryEntry;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HistoryEntryDto {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HistoryResponse {
    pub entries: Vec<HistoryEntryDto>,
}

impl From<&HistoryEntry> for HistoryEntryDto {
    fn from(entry: &HistoryEntry) -> Self {
        Self {
            role: format!("{:?}", entry.role),
            content: entry.content.clone(),
            event: entry.event.clone(),
            timestamp: entry.timestamp.to_rfc3339(),
        }
    }
}

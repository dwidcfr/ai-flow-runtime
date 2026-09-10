pub mod admin;
pub mod error;
pub mod flow;
pub mod modules;
pub mod response;
pub mod router;
pub mod runtime;
pub mod search;
pub mod session;
pub mod state;
pub mod trace;

pub use error::{Result, RuntimeError};
pub use flow::{FlowFacade, TransitionOutcome};
pub use flow::provider::{
    ApiFlowProvider, DatabaseFlowProvider, FlowProvider, YamlFlowProvider,
};
pub use modules::{
    default_registry, ClientModuleFactory, CompanyModuleFactory, ConversationModuleFactory,
    Module, ModuleContent, ModuleFactory, ModuleInfo, ModuleInstance, ModuleRef,
    ModuleRegistry, SmallTalkModuleFactory, CLIENT_MODULE_ID, COMPANY_MODULE_ID,
    CONVERSATION_MODULE_ID, SMALLTALK_MODULE_ID,
};
pub use ai_prompts::PromptRegistry;
pub use runtime::Runtime;
pub use search::{
    CompanySearchProvider, ConversationSearchProvider, SmallTalkSearchProvider,
};
pub use ai_search_engine::{
    Embedder, Index, InMemoryIndex, MockEmbedder, Ranker, SearchDocument, SearchEngine,
    SearchProvider, SearchResult, SimpleRanker,
};
pub use ai_router_engine::{
    MockRouterLLM, RouterContext, RouterDecision, RouterEngine, RouterLLM,
};
pub use ai_response_engine::{
    AgentIdentity, MockResponseLLM, Response, ResponseContext, ResponseEngine, ResponseLLM,
};
#[cfg(feature = "gemini")]
pub use ai_gemini::{GeminiConfig, GeminiResponseLLM, GeminiRouterLLM};
pub use admin::{FlowEdgeExport, FlowExport, FlowNodeExport, FlowStats, ModuleIndexStats, SessionSummary};
pub use session::{ClientData, ConversationContext, HistoryEntry, HistoryRole, Session, SessionMetadata};
pub use state::{RuntimeAction, RuntimeState};
pub use trace::{
    FlowTraceSnapshot, HandleMessageOptions, HandleMessageResult, MessageTrace,
    ModuleTraceSnapshot, PromptTraceSnapshot, TimelineStep,
};

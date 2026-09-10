use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use ai_prompts::{
    bundled_prompts_root, PromptRegistry, RegistryResponsePromptBuilder, RegistryRouterPromptBuilder,
};
use ai_response_engine::{MockResponseLLM, Response, ResponseEngine, ResponseLLM};
use ai_router_engine::{MockRouterLLM, RouterDecision, RouterEngine, RouterLLM};
use ai_search_engine::{InMemoryIndex, MockEmbedder, SearchEngine, SearchResult as EngineSearchResult};
use uuid::Uuid;

use crate::error::{Result, RuntimeError};
use crate::admin::{FlowExport, ModuleIndexStats, SessionSummary};
use crate::flow::provider::{FlowProvider, YamlFlowProvider};
use crate::flow::{FlowFacade, TransitionOutcome};
use crate::modules::{
    default_registry, ModuleContent, ModuleFactory, ModuleInstance, ModuleRef, ModuleRegistry,
    ModuleInfo, CLIENT_MODULE_ID, COMPANY_MODULE_ID, CONVERSATION_MODULE_ID, SMALLTALK_MODULE_ID,
};
use crate::router::execute_router_decision;
use crate::search::{
    CompanySearchProvider, ConversationSearchProvider, SmallTalkSearchProvider,
};
use crate::session::{HistoryRole, InMemorySessionStore, Session};
use crate::state::{can_transition, ensure_active, RuntimeAction, RuntimeState};
use crate::trace::{
    FlowTraceSnapshot, HandleMessageOptions, HandleMessageResult, MessageTrace,
    ModuleTraceSnapshot, PromptTraceSnapshot, TimelineStep, TraceCapture,
    TracingResponseLLM, TracingResponsePromptBuilder, TracingRouterLLM,
    TracingRouterPromptBuilder,
};

pub struct Runtime {
    flow_provider: Box<dyn FlowProvider>,
    flows: HashMap<String, FlowFacade>,
    sessions: InMemorySessionStore,
    module_registry: ModuleRegistry,
    search_engines: HashMap<String, SearchEngine>,
    prompt_registry: Arc<RwLock<PromptRegistry>>,
    trace_capture: TraceCapture,
    router: RouterEngine,
    response_engine: ResponseEngine,
}

impl Runtime {
    fn load_prompt_registry() -> Result<Arc<RwLock<PromptRegistry>>> {
        let registry = PromptRegistry::load(bundled_prompts_root()).map_err(|e| {
            RuntimeError::Configuration {
                message: format!("failed to load prompt registry: {e}"),
            }
        })?;
        Ok(Arc::new(RwLock::new(registry)))
    }

    fn build_engines(
        prompt_registry: Arc<RwLock<PromptRegistry>>,
        trace_capture: &TraceCapture,
        router_llm: Box<dyn RouterLLM>,
        response_llm: Box<dyn ResponseLLM>,
    ) -> (RouterEngine, ResponseEngine) {
        let capture = trace_capture.shared();
        let router_builder = Box::new(TracingRouterPromptBuilder::new(
            Box::new(RegistryRouterPromptBuilder::new(Arc::clone(&prompt_registry))),
            Arc::clone(&capture),
        ));
        let response_builder = Box::new(TracingResponsePromptBuilder::new(
            Box::new(RegistryResponsePromptBuilder::new(Arc::clone(&prompt_registry))),
            Arc::clone(&capture),
        ));
        let router_llm = Box::new(TracingRouterLLM::new(router_llm, Arc::clone(&capture)));
        let response_llm = Box::new(TracingResponseLLM::new(response_llm, Arc::clone(&capture)));
        (
            RouterEngine::with_prompt_builder(router_llm, router_builder),
            ResponseEngine::with_prompt_builder(response_llm, response_builder),
        )
    }

    fn assemble(
        provider: impl FlowProvider + 'static,
        module_registry: ModuleRegistry,
        router_llm: Box<dyn RouterLLM>,
        response_llm: Box<dyn ResponseLLM>,
    ) -> Result<Self> {
        let prompt_registry = Self::load_prompt_registry()?;
        let trace_capture = TraceCapture::new();
        let (router, response_engine) = Self::build_engines(
            Arc::clone(&prompt_registry),
            &trace_capture,
            router_llm,
            response_llm,
        );
        Ok(Self {
            flow_provider: Box::new(provider),
            flows: HashMap::new(),
            sessions: InMemorySessionStore::new(),
            module_registry,
            search_engines: HashMap::new(),
            prompt_registry,
            trace_capture,
            router,
            response_engine,
        })
    }

    pub fn new() -> Self {
        Self::with_provider(YamlFlowProvider::new())
    }

    pub fn with_provider(provider: impl FlowProvider + 'static) -> Self {
        Self::assemble(
            provider,
            default_registry(),
            Box::new(MockRouterLLM::new()),
            Box::new(MockResponseLLM::new()),
        )
        .expect("bundled prompt registry must load")
    }

    pub fn with_provider_and_registry(
        provider: impl FlowProvider + 'static,
        module_registry: ModuleRegistry,
    ) -> Self {
        Self::with_llms_and_registry(
            provider,
            module_registry,
            Box::new(MockRouterLLM::new()),
            Box::new(MockResponseLLM::new()),
        )
    }

    pub fn with_llms(
        provider: impl FlowProvider + 'static,
        router_llm: Box<dyn RouterLLM>,
        response_llm: Box<dyn ResponseLLM>,
    ) -> Self {
        Self::with_llms_and_registry(
            provider,
            default_registry(),
            router_llm,
            response_llm,
        )
    }

    pub fn with_llms_and_registry(
        provider: impl FlowProvider + 'static,
        module_registry: ModuleRegistry,
        router_llm: Box<dyn RouterLLM>,
        response_llm: Box<dyn ResponseLLM>,
    ) -> Self {
        Self::assemble(provider, module_registry, router_llm, response_llm)
            .expect("bundled prompt registry must load")
    }

    #[cfg(feature = "gemini")]
    pub fn with_gemini(
        provider: impl FlowProvider + 'static,
        config: ai_gemini::GeminiConfig,
    ) -> Result<Self> {
        let router_llm = Box::new(
            ai_gemini::GeminiRouterLLM::new(&config).map_err(|e| RuntimeError::Configuration {
                message: e.to_string(),
            })?,
        );
        let response_llm = Box::new(
            ai_gemini::GeminiResponseLLM::new(&config).map_err(|e| RuntimeError::Configuration {
                message: e.to_string(),
            })?,
        );
        Ok(Self::with_llms(provider, router_llm, response_llm))
    }

    #[cfg(feature = "gemini")]
    pub fn with_gemini_from_env(provider: impl FlowProvider + 'static) -> Result<Self> {
        let config = ai_gemini::GeminiConfig::from_env().map_err(|e| RuntimeError::Configuration {
            message: e.to_string(),
        })?;
        Self::with_gemini(provider, config)
    }

    pub fn register_module(&mut self, factory: Box<dyn ModuleFactory>) {
        self.module_registry.register(factory);
    }

    pub fn list_modules(&self) -> Vec<ModuleInfo> {
        self.module_registry.list_modules()
    }

    pub fn has_module(&self, module_id: &str) -> bool {
        self.module_registry.has_module(module_id)
    }

    pub fn load_flow(&mut self, flow_ref: &str) -> Result<String> {
        let facade = self.flow_provider.load(flow_ref)?;
        let flow_id = facade.id().to_string();
        self.flows.insert(flow_id.clone(), facade);
        Ok(flow_id)
    }

    pub fn load_flow_from_path(&mut self, path: &Path) -> Result<String> {
        self.load_flow(path.to_str().ok_or_else(|| RuntimeError::FlowEngineError {
            message: "invalid flow path".to_string(),
        })?)
    }

    pub fn reload_flows_from_dir(&mut self, flows_dir: &Path) -> Result<Vec<String>> {
        use std::fs;
        self.flows.clear();
        let mut paths: Vec<_> = fs::read_dir(flows_dir)
            .map_err(|e| RuntimeError::Configuration {
                message: format!("cannot read flows dir: {e}"),
            })?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("yaml"))
            .collect();
        paths.sort();
        let mut ids = Vec::new();
        for path in paths {
            let id = self.load_flow_from_path(&path)?;
            ids.push(id);
        }
        Ok(ids)
    }

    pub fn create_session(&mut self, flow_id: &str, client_source: &str) -> Result<String> {
        let default_set = self
            .prompt_registry
            .read()
            .map_err(|e| RuntimeError::Configuration {
                message: format!("prompt registry lock poisoned: {e}"),
            })?
            .default_set_id()
            .to_string();
        self.create_session_with_prompt_set(flow_id, client_source, &default_set)
    }

    pub fn create_session_with_prompt_set(
        &mut self,
        flow_id: &str,
        client_source: &str,
        prompt_set: &str,
    ) -> Result<String> {
        {
            let registry = self.prompt_registry.read().map_err(|e| RuntimeError::Configuration {
                message: format!("prompt registry lock poisoned: {e}"),
            })?;
            if !registry.has_set(prompt_set) {
                return Err(RuntimeError::Configuration {
                    message: format!("prompt set not found: {prompt_set}"),
                });
            }
        }

        let flow = self
            .flows
            .get(flow_id)
            .ok_or_else(|| RuntimeError::FlowNotFound(flow_id.to_string()))?;

        let initial = flow.initial_node().to_string();
        let session_id = Uuid::new_v4().to_string();
        let mut session = Session::new(session_id.clone());

        self.init_session_module(&mut session, CONVERSATION_MODULE_ID, None)?;
        self.init_session_module(&mut session, CLIENT_MODULE_ID, Some(client_source))?;

        session.flow_id = Some(flow_id.to_string());
        session.current_node = Some(initial.clone());
        session.runtime_state =
            can_transition(RuntimeState::Idle, RuntimeAction::StartFlow)?;
        session.set_metadata("prompt_set_id", serde_json::json!(prompt_set));
        session.set_metadata("created_at", serde_json::json!(chrono::Utc::now().to_rfc3339()));

        session.record_event(
            "session_created",
            &format!("flow={flow_id}, node={initial}, prompt_set={prompt_set}"),
        );

        self.search_engines
            .insert(session_id.clone(), Self::create_search_engine());
        self.sessions.insert(session);
        self.register_session_search_providers(&session_id)?;
        self.index_session_module(&session_id, CONVERSATION_MODULE_ID)?;

        Ok(session_id)
    }

    pub fn generate_opening_message(&mut self, session_id: &str) -> Result<Response> {
        let context = self.build_opening_response_context(session_id)?;
        let response = self
            .response_engine
            .generate(context)
            .map_err(|e| RuntimeError::ResponseError(e.to_string()))?;
        self.record_assistant_message(session_id, &response.text)?;
        if let Ok(session) = self.sessions.get_mut(session_id) {
            let node = session.current_node.clone().unwrap_or_default();
            session.record_event("call_opening", &format!("node={node}"));
        }
        Ok(response)
    }

    pub fn reload_prompts(&self) -> Result<()> {
        self.prompt_registry
            .write()
            .map_err(|e| RuntimeError::Configuration {
                message: format!("prompt registry lock poisoned: {e}"),
            })?
            .reload()
            .map_err(|e| RuntimeError::Configuration {
                message: format!("failed to reload prompts: {e}"),
            })
    }

    pub fn prompt_registry(&self) -> Arc<RwLock<PromptRegistry>> {
        Arc::clone(&self.prompt_registry)
    }

    pub fn destroy_session(&mut self, session_id: &str) -> Result<()> {
        let session = self.sessions.get(session_id)?;
        can_transition(session.runtime_state, RuntimeAction::DestroySession)?;
        if let Ok(session) = self.sessions.get_mut(session_id) {
            session.clear_modules();
        }
        self.search_engines.remove(session_id);
        self.sessions.remove(session_id)
    }

    pub fn get_session(&self, session_id: &str) -> Result<&Session> {
        self.sessions.get(session_id)
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionSummary>> {
        let default_set = self
            .prompt_registry
            .read()
            .map_err(|e| RuntimeError::Configuration {
                message: format!("prompt registry lock poisoned: {e}"),
            })?
            .default_set_id()
            .to_string();

        let mut summaries = Vec::new();
        for session_id in self.sessions.list_ids() {
            let session = self.sessions.get(&session_id)?;
            summaries.push(SessionSummary::from_session(session, &default_set));
        }
        Ok(summaries)
    }

    pub fn export_flow(&self, flow_id: &str) -> Result<FlowExport> {
        let facade = self
            .flows
            .get(flow_id)
            .ok_or_else(|| RuntimeError::FlowNotFound(flow_id.to_string()))?;
        Ok(FlowExport::from_facade(facade))
    }

    pub fn loaded_flow_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.flows.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub fn session_search_index_len(&self, session_id: &str) -> Result<usize> {
        let engine = self
            .search_engines
            .get(session_id)
            .ok_or_else(|| RuntimeError::SessionNotFound(session_id.to_string()))?;
        Ok(engine.index_len())
    }

    pub fn session_module_index_stats(&self, session_id: &str) -> Result<Vec<ModuleIndexStats>> {
        let engine = self
            .search_engines
            .get(session_id)
            .ok_or_else(|| RuntimeError::SessionNotFound(session_id.to_string()))?;
        Ok(engine
            .module_document_counts()
            .map_err(|e: ai_search_engine::SearchError| RuntimeError::SearchError(e.to_string()))?
            .into_iter()
            .map(|(module_id, document_count)| ModuleIndexStats {
                module_id,
                document_count,
            })
            .collect())
    }

    pub fn append_trace_log(
        &mut self,
        session_id: &str,
        trace: &MessageTrace,
    ) -> Result<()> {
        let entry = serde_json::to_value(trace).map_err(|e| RuntimeError::Configuration {
            message: format!("failed to serialize trace: {e}"),
        })?;
        let session = self.sessions.get_mut(session_id)?;
        let mut log = session
            .get_metadata("trace_log")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        log.push(entry);
        session.set_metadata("trace_log", serde_json::json!(log));
        Ok(())
    }

    pub fn load_session_module(
        &mut self,
        session_id: &str,
        module_id: &str,
        source: &str,
    ) -> Result<()> {
        if !self.module_registry.has_module(module_id) {
            return Err(RuntimeError::ModuleNotFound(module_id.to_string()));
        }

        let session = self.sessions.get_mut(session_id)?;
        ensure_active(session.runtime_state)?;

        if !session.has_module_instance(module_id) {
            let instance = self.module_registry.create_instance(module_id)?;
            session.ensure_module_instance(module_id, instance);
        }

        session.with_module_instance_mut(module_id, |instance| instance.load(source))?;

        if module_id == CLIENT_MODULE_ID {
            let client_data = extract_client_data(session.module_instance(CLIENT_MODULE_ID).unwrap())?;
            session.client_data = client_data;
        }

        session.record_event(
            "load_module",
            &format!("module={module_id}, source={source}"),
        );

        if module_id == COMPANY_MODULE_ID || module_id == SMALLTALK_MODULE_ID {
            self.index_session_module(session_id, module_id)?;
        }

        Ok(())
    }

    pub fn get_module_content(
        &self,
        session_id: &str,
        module_id: &str,
        section: Option<&str>,
    ) -> Result<ModuleContent> {
        let session = self.sessions.get(session_id)?;
        let instance = session
            .module_instance(module_id)
            .ok_or_else(|| RuntimeError::ModuleInstanceNotLoaded(module_id.to_string()))?;
        instance.get_content(section)
    }

    pub fn set_conversation_entry(
        &mut self,
        session_id: &str,
        key: &str,
        value: &str,
    ) -> Result<()> {
        let session = self.sessions.get_mut(session_id)?;
        ensure_active(session.runtime_state)?;

        if !session.has_module_instance(CONVERSATION_MODULE_ID) {
            let instance = self
                .module_registry
                .create_instance(CONVERSATION_MODULE_ID)?;
            session.ensure_module_instance(CONVERSATION_MODULE_ID, instance);
        }

        session.with_module_instance_mut(CONVERSATION_MODULE_ID, |instance| {
            instance.set_entry(key, value)
        })?;

        session.record_event(
            "conversation_entry",
            &format!("{key}={value}"),
        );

        self.index_session_module(session_id, CONVERSATION_MODULE_ID)?;
        Ok(())
    }

    pub fn available_transitions(&self, session_id: &str) -> Result<Vec<String>> {
        let (flow_id, current_node) = {
            let session = self.sessions.get(session_id)?;
            ensure_active(session.runtime_state)?;
            let flow_id = session
                .flow_id
                .clone()
                .ok_or_else(|| RuntimeError::FlowNotFound("none".to_string()))?;
            let current_node = session
                .current_node
                .clone()
                .ok_or_else(|| RuntimeError::NodeNotFound {
                    flow_id: flow_id.clone(),
                    node_id: "none".to_string(),
                })?;
            (flow_id, current_node)
        };

        let flow = self
            .flows
            .get(&flow_id)
            .ok_or_else(|| RuntimeError::FlowNotFound(flow_id.clone()))?;

        flow.available_transitions(&current_node)
    }

    pub fn transition(
        &mut self,
        session_id: &str,
        transition_id: &str,
    ) -> Result<TransitionOutcome> {
        self.goto_transition(session_id, transition_id)
    }

    pub fn goto_transition(
        &mut self,
        session_id: &str,
        action: &str,
    ) -> Result<TransitionOutcome> {
        let (flow_id, current_node, current_state) = {
            let session = self.sessions.get(session_id)?;
            ensure_active(session.runtime_state)?;
            can_transition(session.runtime_state, RuntimeAction::Transition)?;
            let flow_id = session
                .flow_id
                .clone()
                .ok_or_else(|| RuntimeError::FlowNotFound("none".to_string()))?;
            let current_node = session
                .current_node
                .clone()
                .ok_or_else(|| RuntimeError::NodeNotFound {
                    flow_id: flow_id.clone(),
                    node_id: "none".to_string(),
                })?;
            (flow_id, current_node, session.runtime_state)
        };

        let flow = self
            .flows
            .get(&flow_id)
            .ok_or_else(|| RuntimeError::FlowNotFound(flow_id.clone()))?;

        let result = flow.transition(&current_node, action)?;

        let session = self.sessions.get_mut(session_id)?;
        session.current_node = Some(result.to.clone());

        if result.target_is_end {
            session.runtime_state =
                can_transition(current_state, RuntimeAction::FinishSession)?;
            session.record_event(
                "flow_finished",
                &format!(
                    "transition={action}, node={} -> {}",
                    result.from, result.to
                ),
            );
        } else {
            session.runtime_state = RuntimeState::Flow;
            session.record_event(
                "transition",
                &format!(
                    "transition={action}, node={} -> {}",
                    result.from, result.to
                ),
            );
        }

        Ok(result)
    }

    pub fn goto_node(&mut self, session_id: &str, node_id: &str) -> Result<()> {
        let flow_id = {
            let session = self.sessions.get(session_id)?;
            ensure_active(session.runtime_state)?;
            if session.runtime_state != RuntimeState::Flow {
                return Err(RuntimeError::InvalidStateTransition {
                    from: session.runtime_state,
                    action: RuntimeAction::Transition,
                });
            }
            session
                .flow_id
                .clone()
                .ok_or_else(|| RuntimeError::FlowNotFound("none".to_string()))?
        };

        let flow = self
            .flows
            .get(&flow_id)
            .ok_or_else(|| RuntimeError::FlowNotFound(flow_id.clone()))?;

        if !flow.contains_node(node_id) {
            return Err(RuntimeError::NodeNotFound {
                flow_id: flow_id.clone(),
                node_id: node_id.to_string(),
            });
        }

        let from_node = self.sessions.get(session_id)?.current_node.clone();
        let is_end = flow.is_end(node_id);

        let session = self.sessions.get_mut(session_id)?;
        session.current_node = Some(node_id.to_string());

        if is_end {
            session.runtime_state =
                can_transition(session.runtime_state, RuntimeAction::FinishSession)?;
            session.record_event(
                "goto_end",
                &format!("node={node_id} (from {:?})", from_node),
            );
        } else {
            session.record_event(
                "goto_node",
                &format!("node={node_id} (from {:?})", from_node),
            );
        }

        Ok(())
    }

    pub fn pause_flow(&mut self, session_id: &str) -> Result<()> {
        let session = self.sessions.get_mut(session_id)?;
        ensure_active(session.runtime_state)?;

        let current_node = session
            .current_node
            .clone()
            .ok_or_else(|| RuntimeError::NodeNotFound {
                flow_id: session.flow_id.clone().unwrap_or_default(),
                node_id: "none".to_string(),
            })?;

        session.runtime_state =
            can_transition(session.runtime_state, RuntimeAction::PauseFlow)?;
        session.paused_node = Some(current_node.clone());
        session.record_event("pause_flow", &format!("paused_node={current_node}"));

        Ok(())
    }

    pub fn resume_flow(&mut self, session_id: &str) -> Result<()> {
        let session = self.sessions.get_mut(session_id)?;
        ensure_active(session.runtime_state)?;

        if session.runtime_state == RuntimeState::Flow {
            return Ok(());
        }

        if session.runtime_state != RuntimeState::Paused {
            return Err(RuntimeError::InvalidStateTransition {
                from: session.runtime_state,
                action: RuntimeAction::ResumeFlow,
            });
        }

        let paused_node = session
            .paused_node
            .clone()
            .ok_or(RuntimeError::NoPausedNode)?;

        session.runtime_state =
            can_transition(session.runtime_state, RuntimeAction::ResumeFlow)?;
        session.current_node = Some(paused_node.clone());
        session.paused_node = None;
        session.record_event("resume_flow", &format!("resumed_node={paused_node}"));

        Ok(())
    }

    pub fn open_module(&mut self, session_id: &str, module_ref: &str) -> Result<()> {
        let module = ModuleRef::parse(module_ref)?;

        if !self.module_registry.has_module(&module.module_id) {
            return Err(RuntimeError::ModuleNotFound(module.module_id.clone()));
        }

        let session = self.sessions.get_mut(session_id)?;
        ensure_active(session.runtime_state)?;

        if session.runtime_state != RuntimeState::Paused {
            return Err(RuntimeError::PauseRequiredBeforeModule);
        }

        if session.active_module.is_some() {
            return Err(RuntimeError::ModuleAlreadyOpen);
        }

        session.runtime_state =
            can_transition(session.runtime_state, RuntimeAction::OpenModule)?;
        session.active_module = Some(module.clone());
        session.record_event("open_module", &format!("module={}", module.id()));

        Ok(())
    }

    pub fn close_module(&mut self, session_id: &str) -> Result<()> {
        {
            let session = self.sessions.get(session_id)?;
            ensure_active(session.runtime_state)?;
            can_transition(session.runtime_state, RuntimeAction::CloseModule)?;
            if session.active_module.is_none() {
                return Err(RuntimeError::ModuleNotOpen);
            }
        }

        let session = self.sessions.get_mut(session_id)?;
        let module_id = session
            .active_module
            .as_ref()
            .map(|m| m.id())
            .unwrap_or_default();

        session.active_module = None;
        session.runtime_state =
            can_transition(RuntimeState::Module, RuntimeAction::CloseModule)?;

        let paused_node = session
            .paused_node
            .clone()
            .ok_or(RuntimeError::NoPausedNode)?;

        session.current_node = Some(paused_node.clone());
        session.paused_node = None;
        session.record_event(
            "close_module",
            &format!("module={module_id}, auto_resume={paused_node}"),
        );

        Ok(())
    }

    pub fn finish_session(&mut self, session_id: &str) -> Result<()> {
        let session = self.sessions.get_mut(session_id)?;
        ensure_active(session.runtime_state)?;

        session.runtime_state =
            can_transition(session.runtime_state, RuntimeAction::FinishSession)?;
        session.clear_modules();
        session.record_event("finish_session", "session finished");
        self.search_engines.remove(session_id);

        Ok(())
    }

    pub fn register_session_search_providers(&mut self, session_id: &str) -> Result<()> {
        let providers = {
            let session = self.sessions.get(session_id)?;
            let mut providers: Vec<Box<dyn ai_search_engine::SearchProvider>> = Vec::new();

            if let Some(instance) = session.module_instance(COMPANY_MODULE_ID) {
                providers.push(Box::new(CompanySearchProvider::from_instance(instance)?));
            }
            if let Some(instance) = session.module_instance(CONVERSATION_MODULE_ID) {
                providers.push(Box::new(ConversationSearchProvider::from_instance(instance)?));
            }
            if let Some(instance) = session.module_instance(SMALLTALK_MODULE_ID) {
                providers.push(Box::new(SmallTalkSearchProvider::from_instance(instance)?));
            }

            providers
        };

        let engine = self
            .search_engines
            .get_mut(session_id)
            .ok_or_else(|| RuntimeError::SessionNotFound(session_id.to_string()))?;

        for provider in providers {
            engine.register_provider(provider);
        }

        Ok(())
    }

    pub fn build_session_index(&mut self, session_id: &str) -> Result<()> {
        self.register_session_search_providers(session_id)?;
        let engine = self
            .search_engines
            .get_mut(session_id)
            .ok_or_else(|| RuntimeError::SessionNotFound(session_id.to_string()))?;
        engine
            .build_index()
            .map_err(|e| RuntimeError::SearchError(e.to_string()))?;
        Ok(())
    }

    pub fn index_session_module(&mut self, session_id: &str, module_id: &str) -> Result<()> {
        let provider: Box<dyn ai_search_engine::SearchProvider> = {
            let session = self.sessions.get(session_id)?;
            let instance = session
                .module_instance(module_id)
                .ok_or_else(|| RuntimeError::ModuleInstanceNotLoaded(module_id.to_string()))?;

            match module_id {
                COMPANY_MODULE_ID => Box::new(CompanySearchProvider::from_instance(instance)?),
                CONVERSATION_MODULE_ID => {
                    Box::new(ConversationSearchProvider::from_instance(instance)?)
                }
                SMALLTALK_MODULE_ID => Box::new(SmallTalkSearchProvider::from_instance(instance)?),
                _ => return Err(RuntimeError::ModuleNotFound(module_id.to_string())),
            }
        };

        let engine = self
            .search_engines
            .get_mut(session_id)
            .ok_or_else(|| RuntimeError::SessionNotFound(session_id.to_string()))?;
        engine.register_provider(provider);
        engine
            .reindex_module(module_id)
            .map_err(|e| RuntimeError::SearchError(e.to_string()))?;
        Ok(())
    }

    pub fn search_session(
        &self,
        session_id: &str,
        query: &str,
        module_ids: Option<&[String]>,
        top_k: usize,
    ) -> Result<Vec<EngineSearchResult>> {
        let engine = self
            .search_engines
            .get(session_id)
            .ok_or_else(|| RuntimeError::SessionNotFound(session_id.to_string()))?;
        engine
            .search(query, module_ids, top_k)
            .map_err(|e| RuntimeError::SearchError(e.to_string()))
    }

    pub fn get_current_node_payload(&self, session_id: &str) -> Result<serde_json::Value> {
        let session = self.get_session(session_id)?;
        let flow_id = session
            .flow_id
            .clone()
            .ok_or_else(|| RuntimeError::FlowNotFound("none".to_string()))?;
        let node_id = session
            .current_node
            .clone()
            .ok_or_else(|| RuntimeError::NodeNotFound {
                flow_id: flow_id.clone(),
                node_id: "none".to_string(),
            })?;

        let flow = self
            .flows
            .get(&flow_id)
            .ok_or_else(|| RuntimeError::FlowNotFound(flow_id))?;

        flow.node_payload(&node_id)
    }

    pub fn flow_snapshot(
        &self,
        session_id: &str,
        previous_node: Option<String>,
    ) -> Result<FlowTraceSnapshot> {
        self.build_flow_trace_snapshot(session_id, previous_node)
    }

    pub fn generate_response(
        &self,
        session_id: &str,
        user_message: &str,
        decision: &RouterDecision,
    ) -> Result<Response> {
        let context = self.build_response_context(session_id, user_message, decision)?;
        self.response_engine
            .generate(context)
            .map_err(|e| RuntimeError::ResponseError(e.to_string()))
    }

    pub fn record_assistant_message(&mut self, session_id: &str, content: &str) -> Result<()> {
        let session = self.sessions.get_mut(session_id)?;
        if session.runtime_state == RuntimeState::Finished {
            return Err(RuntimeError::SessionFinished);
        }
        session.record_message(HistoryRole::Assistant, content);
        Ok(())
    }

    pub fn route_user_message(
        &self,
        session_id: &str,
        user_message: &str,
    ) -> Result<RouterDecision> {
        let context = self.build_router_context(session_id, user_message)?;
        Ok(self.router.route(context))
    }

    pub fn execute_router_decision(
        &mut self,
        session_id: &str,
        decision: RouterDecision,
    ) -> Result<()> {
        execute_router_decision(self, session_id, decision)
    }

    pub fn handle_user_message(
        &mut self,
        session_id: &str,
        user_message: &str,
    ) -> Result<(RouterDecision, Response)> {
        let result = self.handle_user_message_with_options(
            session_id,
            user_message,
            HandleMessageOptions::default(),
        )?;
        Ok((result.decision, result.response))
    }

    pub fn handle_user_message_with_options(
        &mut self,
        session_id: &str,
        user_message: &str,
        opts: HandleMessageOptions,
    ) -> Result<HandleMessageResult> {
        let total_start = Instant::now();
        let collect_trace = opts.collect_trace;

        let previous_node = if collect_trace {
            self.get_session(session_id)?.current_node.clone()
        } else {
            None
        };

        let (search_results, search_ms) = if collect_trace {
            let search_start = Instant::now();
            let results = self.search_session(session_id, user_message, None, 5)?;
            (results, search_start.elapsed().as_millis() as u64)
        } else {
            (Vec::new(), 0)
        };

        if collect_trace {
            self.trace_capture.reset();
        }

        {
            let session = self.sessions.get_mut(session_id)?;
            if session.runtime_state == RuntimeState::Finished {
                return Err(RuntimeError::SessionFinished);
            }
            session.record_message(HistoryRole::User, user_message);
        }

        let router_start = Instant::now();
        let decision = self.route_user_message(session_id, user_message)?;
        let router_ms = router_start.elapsed().as_millis() as u64;

        let llm_capture = if collect_trace {
            Some(self.trace_capture.snapshot())
        } else {
            None
        };

        let response_start = Instant::now();
        let response = match &decision {
            RouterDecision::EndConversation { .. } => {
                let response = self.generate_response(session_id, user_message, &decision)?;
                self.record_assistant_message(session_id, &response.text)?;
                self.execute_router_decision(session_id, decision.clone())?;
                response
            }
            RouterDecision::Clarify { .. } | RouterDecision::Error { .. } => {
                let response = self.generate_response(session_id, user_message, &decision)?;
                self.record_assistant_message(session_id, &response.text)?;
                response
            }
            _ => {
                self.execute_router_decision(session_id, decision.clone())?;
                let response = self.generate_response(session_id, user_message, &decision)?;
                self.record_assistant_message(session_id, &response.text)?;
                response
            }
        };
        let response_ms = response_start.elapsed().as_millis() as u64;
        let total_ms = total_start.elapsed().as_millis() as u64;

        let trace = if collect_trace {
            let capture = llm_capture.unwrap_or_default();
            let flow = self.build_flow_trace_snapshot(session_id, previous_node)?;
            let modules = self.build_module_trace_snapshot(session_id)?;
            let prompts = self.build_prompt_trace_snapshot(session_id)?;
            let mut timeline = vec![
                TimelineStep {
                    step: "search".into(),
                    duration_ms: search_ms,
                },
                TimelineStep {
                    step: "router".into(),
                    duration_ms: router_ms,
                },
            ];
            if matches!(
                &decision,
                RouterDecision::Flow { .. } | RouterDecision::Module { .. }
            ) {
                timeline.push(TimelineStep {
                    step: "flow_or_module".into(),
                    duration_ms: 0,
                });
            }
            timeline.push(TimelineStep {
                step: "response".into(),
                duration_ms: response_ms,
            });
            timeline.push(TimelineStep {
                step: "total".into(),
                duration_ms: total_ms,
            });

            Some(MessageTrace {
                search_results,
                search_ms,
                router_prompt: capture.router_prompt,
                router_raw_json: capture.router_raw,
                router_decision: decision.clone(),
                router_ms,
                response_prompt: capture.response_prompt,
                response_raw_json: capture.response_raw,
                response_ms,
                total_ms,
                timeline,
                flow,
                modules,
                prompts,
            })
        } else {
            None
        };

        if let Some(ref trace) = trace {
            if collect_trace {
                let _ = self.append_trace_log(session_id, trace);
            }
        }

        Ok(HandleMessageResult {
            decision,
            response,
            trace,
        })
    }

    fn build_flow_trace_snapshot(
        &self,
        session_id: &str,
        previous_node: Option<String>,
    ) -> Result<FlowTraceSnapshot> {
        let session = self.get_session(session_id)?;
        let flow_id = session.flow_id.clone();
        let current_node = session.current_node.clone();
        let available_transitions = self
            .available_transitions(session_id)
            .unwrap_or_default();
        let node_payload = self
            .get_current_node_payload(session_id)
            .unwrap_or(serde_json::Value::Null);
        let is_end = match (&flow_id, &current_node) {
            (Some(fid), Some(node)) => self
                .flows
                .get(fid)
                .map(|f| f.is_end(node))
                .unwrap_or(false),
            _ => false,
        };

        Ok(FlowTraceSnapshot {
            flow_id,
            previous_node,
            current_node,
            paused_node: session.paused_node.clone(),
            available_transitions,
            node_payload,
            is_end,
            runtime_state: session.runtime_state,
        })
    }

    fn build_module_trace_snapshot(&self, session_id: &str) -> Result<ModuleTraceSnapshot> {
        let session = self.get_session(session_id)?;
        let active_module = session.active_module.as_ref().map(|m| m.id());
        let conversation_entries = self
            .get_module_content(session_id, CONVERSATION_MODULE_ID, None)
            .map(|content| conversation_map_from_value(&content.data))
            .unwrap_or_default();

        Ok(ModuleTraceSnapshot {
            active_module,
            conversation_entries,
        })
    }

    fn build_prompt_trace_snapshot(&self, session_id: &str) -> Result<PromptTraceSnapshot> {
        let session = self.get_session(session_id)?;
        let prompt_set = session
            .get_metadata("prompt_set_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                self.prompt_registry
                    .read()
                    .map(|r| r.default_set_id().to_string())
                    .unwrap_or_else(|_| "demo".into())
            });
        let identity = self
            .prompt_registry
            .read()
            .map_err(|e| RuntimeError::Configuration {
                message: format!("prompt registry lock poisoned: {e}"),
            })?
            .identity(&prompt_set)
            .map_err(|e| RuntimeError::Configuration {
                message: format!("failed to load identity: {e}"),
            })?;

        Ok(PromptTraceSnapshot {
            prompt_set,
            identity,
        })
    }

    fn create_search_engine() -> SearchEngine {
        SearchEngine::new(
            Box::new(MockEmbedder::default()),
            Box::new(InMemoryIndex::new()),
        )
    }

    fn init_session_module(
        &mut self,
        session: &mut Session,
        module_id: &str,
        source: Option<&str>,
    ) -> Result<()> {
        let mut instance = self.module_registry.create_instance(module_id)?;
        if let Some(path) = source {
            instance.load(path)?;
        }
        session.ensure_module_instance(module_id, instance);

        if module_id == CLIENT_MODULE_ID {
            if let Some(instance) = session.module_instance(CLIENT_MODULE_ID) {
                session.client_data = extract_client_data(instance)?;
            }
        }
        Ok(())
    }
}

fn extract_client_data(instance: &dyn ModuleInstance) -> Result<crate::session::ClientData> {
    let content = instance.get_content(None)?;
    if let Some(map) = content.data.as_object() {
        Ok(map.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    } else {
        Ok(crate::session::ClientData::new())
    }
}

fn conversation_map_from_value(value: &serde_json::Value) -> HashMap<String, String> {
    value
        .as_object()
        .map(|map| {
            map.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::COMPANY_MODULE_ID;
    use std::path::PathBuf;

    fn data_path(relative: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
    }

    fn setup_runtime() -> (Runtime, String, String) {
        let mut rt = Runtime::new();
        let flow_id = rt
            .load_flow(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
            .unwrap();
        let session_id = rt
            .create_session(
                &flow_id,
                data_path("data/clients/sample.json").to_str().unwrap(),
            )
            .unwrap();
        (rt, flow_id, session_id)
    }

    #[test]
    fn list_modules_returns_four() {
        let rt = Runtime::new();
        assert_eq!(rt.list_modules().len(), 4);
        assert!(rt.has_module("client"));
        assert!(rt.has_module("company"));
        assert!(rt.has_module("conversation"));
        assert!(rt.has_module("smalltalk"));
    }

    #[test]
    fn create_session_starts_at_initial_node() {
        let (rt, _, session_id) = setup_runtime();
        let session = rt.get_session(&session_id).unwrap();
        assert_eq!(session.runtime_state, RuntimeState::Flow);
        assert_eq!(session.current_node.as_deref(), Some("greeting"));
        assert!(session.module_instance("client").is_some());
        assert!(session.module_instance("conversation").is_some());
    }

    #[test]
    fn available_transitions_at_greeting() {
        let (rt, _, session_id) = setup_runtime();
        let transitions = rt.available_transitions(&session_id).unwrap();
        assert!(transitions.contains(&"confirm".to_string()));
    }

    #[test]
    fn full_lifecycle_with_module() {
        let (mut rt, _, session_id) = setup_runtime();

        rt.load_session_module(
            &session_id,
            COMPANY_MODULE_ID,
            data_path("data/company/acme.yaml").to_str().unwrap(),
        )
        .unwrap();

        rt.goto_transition(&session_id, "confirm").unwrap();
        rt.pause_flow(&session_id).unwrap();
        rt.open_module(&session_id, "company.payment_methods")
            .unwrap();

        assert_eq!(
            rt.get_session(&session_id).unwrap().runtime_state,
            RuntimeState::Module
        );

        rt.close_module(&session_id).unwrap();
        let session = rt.get_session(&session_id).unwrap();
        assert_eq!(session.runtime_state, RuntimeState::Flow);
        assert_eq!(session.current_node.as_deref(), Some("payment"));
        assert!(session.paused_node.is_none());
        assert!(session.active_module.is_none());
    }

    #[test]
    fn conversation_cleared_on_finish() {
        let (mut rt, _, session_id) = setup_runtime();
        rt.set_conversation_entry(&session_id, "promise", "вечером")
            .unwrap();
        rt.finish_session(&session_id).unwrap();
        let err = rt
            .get_module_content(&session_id, CONVERSATION_MODULE_ID, None)
            .unwrap_err();
        assert!(matches!(err, RuntimeError::ModuleInstanceNotLoaded(_)));
    }

    #[test]
    fn open_module_without_pause_fails() {
        let (mut rt, _, session_id) = setup_runtime();
        let err = rt
            .open_module(&session_id, "company.payment_methods")
            .unwrap_err();
        assert!(matches!(err, RuntimeError::PauseRequiredBeforeModule));
    }
}

use std::time::Instant;

use ai_flow_runtime::{Response, RouterDecision, Runtime};
#[cfg(feature = "gemini")]
use ai_flow_runtime::flow::provider::YamlFlowProvider;

use crate::assertions::{run_assertions, AssertionFailure};
use crate::error::{EvalError, Result};
use crate::scenario::{
    EvalScenario, FixturesManifest, LlmMode, PostStep, SetupStep,
};

#[derive(Debug, Clone)]
pub struct ScenarioTimings {
    pub router_ms: u64,
    pub response_ms: u64,
    pub search_ms: u64,
    pub total_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioStatus {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct ScenarioResult {
    pub scenario_name: String,
    pub domain: Option<String>,
    pub status: ScenarioStatus,
    pub skip_reason: Option<String>,
    pub decision: Option<RouterDecision>,
    pub response: Option<Response>,
    pub failures: Vec<AssertionFailure>,
    pub timings: Option<ScenarioTimings>,
    pub router_error: bool,
    pub response_error: bool,
}

pub struct ScenarioRunner {
    fixtures: FixturesManifest,
    llm_override: Option<LlmMode>,
    gemini_enabled: bool,
}

impl ScenarioRunner {
    pub fn new(fixtures: FixturesManifest) -> Self {
        Self {
            fixtures,
            llm_override: None,
            gemini_enabled: cfg!(feature = "gemini"),
        }
    }

    pub fn with_llm_override(mut self, mode: LlmMode) -> Self {
        self.llm_override = Some(mode);
        self
    }

    pub fn run(&self, scenario: &EvalScenario) -> ScenarioResult {
        let started = Instant::now();
        let llm_mode = self.llm_override.unwrap_or(scenario.llm);

        if llm_mode == LlmMode::Gemini {
            if !self.gemini_enabled {
                return skipped(scenario, "gemini feature not enabled");
            }
            if !gemini_api_key_set() {
                return skipped(scenario, "GEMINI_API_KEY not set");
            }
        }

        match self.run_inner(scenario, llm_mode) {
            Ok((failures, decision, response, timings, router_error, response_error)) => {
                let status = if failures.is_empty() {
                    ScenarioStatus::Passed
                } else {
                    ScenarioStatus::Failed
                };
                ScenarioResult {
                    scenario_name: scenario.name.clone(),
                    domain: scenario.domain.clone(),
                    status,
                    skip_reason: None,
                    decision: Some(decision),
                    response: Some(response),
                    failures,
                    timings: Some(timings),
                    router_error,
                    response_error,
                    ..Default::default()
                }
            }
            Err(err) => ScenarioResult {
                scenario_name: scenario.name.clone(),
                domain: scenario.domain.clone(),
                status: ScenarioStatus::Failed,
                skip_reason: None,
                failures: vec![AssertionFailure {
                    field: "runner".into(),
                    expected: "successful execution".into(),
                    actual: err.to_string(),
                }],
                timings: Some(ScenarioTimings {
                    router_ms: 0,
                    response_ms: 0,
                    search_ms: 0,
                    total_ms: started.elapsed().as_millis() as u64,
                }),
                router_error: false,
                response_error: false,
                decision: None,
                response: None,
            },
        }
    }

    fn run_inner(
        &self,
        scenario: &EvalScenario,
        llm_mode: LlmMode,
    ) -> Result<(
        Vec<AssertionFailure>,
        RouterDecision,
        Response,
        ScenarioTimings,
        bool,
        bool,
    )> {
        let total_start = Instant::now();
        let mut runtime = self.build_runtime(llm_mode)?;

        let flow_path = self.fixtures.flow_path(&scenario.flow)?;
        let client_path = self.fixtures.client_path(&scenario.client)?;

        let flow_id = runtime
            .load_flow(flow_path.to_str().ok_or_else(|| {
                EvalError::Runtime("invalid flow path".into())
            })?)
            .map_err(runtime_err)?;

        let session_id = runtime
            .create_session(
                &flow_id,
                client_path.to_str().ok_or_else(|| {
                    EvalError::Runtime("invalid client path".into())
                })?,
            )
            .map_err(runtime_err)?;

        for (module_id, fixture_alias) in &scenario.modules {
            let module_path = self.fixtures.module_path(fixture_alias)?;
            runtime
                .load_session_module(
                    &session_id,
                    module_id,
                    module_path.to_str().ok_or_else(|| {
                        EvalError::Runtime("invalid module path".into())
                    })?,
                )
                .map_err(runtime_err)?;
        }

        self.apply_setup(&mut runtime, &session_id, scenario)?;
        self.seed_conversation(&mut runtime, &session_id, scenario)?;

        let handle_start = Instant::now();
        let (decision, response) = runtime
            .handle_user_message(&session_id, &scenario.user_message)
            .map_err(runtime_err)?;
        let handle_ms = handle_start.elapsed().as_millis() as u64;
        let router_ms = handle_ms / 2;
        let response_ms = handle_ms - router_ms;

        self.apply_post_steps(&mut runtime, &session_id, scenario)?;

        let search_start = Instant::now();
        let search_results = if scenario.expected.search.is_some() {
            let search = scenario.expected.search.as_ref().unwrap();
            Some(
                runtime
                    .search_session(&session_id, &search.query, None, search.top_k)
                    .map_err(runtime_err)?,
            )
        } else {
            None
        };
        let search_ms = search_start.elapsed().as_millis() as u64;

        let session = runtime.get_session(&session_id).map_err(runtime_err)?;
        let failures = run_assertions(
            scenario,
            &decision,
            &response,
            session,
            search_results.as_deref(),
        );

        let router_error = matches!(decision, RouterDecision::Error { .. });
        let response_error = response.text.contains("не удалось сформировать ответ");

        Ok((
            failures,
            decision,
            response,
            ScenarioTimings {
                router_ms,
                response_ms,
                search_ms,
                total_ms: total_start.elapsed().as_millis() as u64,
            },
            router_error,
            response_error,
        ))
    }

    fn build_runtime(&self, llm_mode: LlmMode) -> Result<Runtime> {
        match llm_mode {
            LlmMode::Mock => Ok(Runtime::new()),
            LlmMode::Gemini => {
                #[cfg(feature = "gemini")]
                {
                    let _ = dotenvy::dotenv();
                    Runtime::with_gemini_from_env(YamlFlowProvider::new())
                        .map_err(|e| EvalError::Runtime(e.to_string()))
                }
                #[cfg(not(feature = "gemini"))]
                {
                    Err(EvalError::Runtime("gemini feature not enabled".into()))
                }
            }
        }
    }

    fn apply_setup(
        &self,
        runtime: &mut Runtime,
        session_id: &str,
        scenario: &EvalScenario,
    ) -> Result<()> {
        for step in &scenario.setup {
            match step {
                SetupStep::Transition { transition } => {
                    runtime
                        .goto_transition(session_id, transition)
                        .map_err(runtime_err)?;
                }
                SetupStep::GotoNode { goto_node } => {
                    runtime
                        .goto_node(session_id, goto_node)
                        .map_err(runtime_err)?;
                }
                SetupStep::ConversationEntry { conversation_entry } => {
                    runtime
                        .set_conversation_entry(
                            session_id,
                            &conversation_entry.key,
                            &conversation_entry.value,
                        )
                        .map_err(runtime_err)?;
                }
            }
        }
        Ok(())
    }

    fn seed_conversation(
        &self,
        runtime: &mut Runtime,
        session_id: &str,
        scenario: &EvalScenario,
    ) -> Result<()> {
        for turn in &scenario.conversation {
            match turn.role.to_lowercase().as_str() {
                "assistant" => {
                    runtime
                        .record_assistant_message(session_id, &turn.text)
                        .map_err(runtime_err)?;
                }
                "user" => {
                    runtime
                        .handle_user_message(session_id, &turn.text)
                        .map_err(runtime_err)?;
                }
                other => {
                    return Err(EvalError::Load {
                        path: scenario.source_path.display().to_string(),
                        message: format!("unsupported conversation role: {other}"),
                    });
                }
            }
        }
        Ok(())
    }

    fn apply_post_steps(
        &self,
        runtime: &mut Runtime,
        session_id: &str,
        scenario: &EvalScenario,
    ) -> Result<()> {
        for step in &scenario.steps {
            match step {
                PostStep::CloseModule => {
                    runtime.close_module(session_id).map_err(runtime_err)?;
                }
            }
        }
        Ok(())
    }
}

fn runtime_err(e: ai_flow_runtime::RuntimeError) -> EvalError {
    EvalError::Runtime(e.to_string())
}

fn gemini_api_key_set() -> bool {
    std::env::var("GEMINI_API_KEY")
        .ok()
        .is_some_and(|k| !k.trim().is_empty())
}

fn skipped(scenario: &EvalScenario, reason: &str) -> ScenarioResult {
    ScenarioResult {
        scenario_name: scenario.name.clone(),
        domain: scenario.domain.clone(),
        status: ScenarioStatus::Skipped,
        skip_reason: Some(reason.into()),
        failures: Vec::new(),
        timings: None,
        router_error: false,
        response_error: false,
        decision: None,
        response: None,
    }
}

impl Default for ScenarioResult {
    fn default() -> Self {
        Self {
            scenario_name: String::new(),
            domain: None,
            status: ScenarioStatus::Failed,
            skip_reason: None,
            decision: None,
            response: None,
            failures: Vec::new(),
            timings: None,
            router_error: false,
            response_error: false,
        }
    }
}

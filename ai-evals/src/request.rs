use std::time::Instant;

use crate::error::{EvalError, Result};
use crate::loader::ScenarioLoader;
use crate::metrics::collect_metrics;
use crate::reporter::{build_report, EvalReport};
use crate::runner::{ScenarioResult, ScenarioRunner};
use crate::scenario::LlmMode;

#[derive(Debug, Clone)]
pub struct EvalRunRequest {
    pub scenario: Option<String>,
    pub domain: Option<String>,
    pub llm: LlmMode,
}

impl EvalRunRequest {
    pub fn from_llm_str(llm: Option<&str>) -> LlmMode {
        match llm.map(|s| s.to_lowercase()).as_deref() {
            Some("gemini") => LlmMode::Gemini,
            _ => LlmMode::Mock,
        }
    }
}

pub fn run_evals_request(request: EvalRunRequest) -> Result<EvalReport> {
    let loader = ScenarioLoader::new(&crate::crate_root())?;
    let scenarios = if let Some(name) = request.scenario.as_deref() {
        vec![loader.load_by_name(name)?]
    } else if let Some(domain) = request.domain.as_deref() {
        loader.load_by_domain(domain)?
    } else {
        loader.discover_all()?
    };

    if scenarios.is_empty() {
        return Err(EvalError::ScenarioNotFound(
            request
                .scenario
                .clone()
                .unwrap_or_else(|| "all".into()),
        ));
    }

    let runner = ScenarioRunner::new(loader.fixtures().clone()).with_llm_override(request.llm);
    let started = Instant::now();
    let results: Vec<ScenarioResult> = scenarios.iter().map(|s| runner.run(s)).collect();
    let metrics = collect_metrics(&results, started.elapsed());
    Ok(build_report(metrics, &results))
}

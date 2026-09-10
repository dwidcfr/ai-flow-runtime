use axum::Json;
use axum::extract::{Path, State};

use ai_evals::{crate_root, EvalRunRequest, ScenarioLoader};

use crate::dto::{
    AssertionFailureDto, EvalAsyncRunResponse, EvalMetricsDto, EvalReportDto, EvalRunApiRequest,
    EvalRunResponse, EvalRunStatusResponse, EvalScenarioDto, EvalScenarioResultDto, EvalScenariosResponse,
};
use crate::errors::ApiError;
use crate::eval_store::{spawn_async_eval, EvalRunState};
use crate::state::AppState;

fn map_report(report: ai_evals::EvalReport) -> EvalReportDto {
    EvalReportDto {
        metrics: EvalMetricsDto {
            total: report.metrics.total,
            passed: report.metrics.passed,
            failed: report.metrics.failed,
            skipped: report.metrics.skipped,
            duration_ms: report.metrics.duration_ms,
        },
        results: report
            .results
            .into_iter()
            .map(|r| EvalScenarioResultDto {
                name: r.name,
                domain: r.domain,
                status: r.status,
                skip_reason: r.skip_reason,
                failures: r
                    .failures
                    .into_iter()
                    .map(|f| AssertionFailureDto {
                        field: f.field,
                        expected: f.expected,
                        actual: f.actual,
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn to_eval_request(body: &EvalRunApiRequest) -> EvalRunRequest {
    EvalRunRequest {
        scenario: body.scenario.clone(),
        domain: body.domain.clone(),
        llm: EvalRunRequest::from_llm_str(body.llm.as_deref()),
    }
}

pub async fn list_eval_scenarios() -> Result<Json<EvalScenariosResponse>, ApiError> {
    let loader = ScenarioLoader::new(&crate_root()).map_err(|e| ApiError::internal(e.to_string()))?;
    let scenarios = loader
        .discover_all()
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(EvalScenariosResponse {
        scenarios: scenarios
            .into_iter()
            .map(|s| EvalScenarioDto {
                name: s.name,
                domain: s.domain,
                llm: Some(format!("{:?}", s.llm).to_lowercase()),
                source_path: s.source_path.display().to_string(),
            })
            .collect(),
    }))
}

pub async fn run_eval_sync(
    Json(body): Json<EvalRunApiRequest>,
) -> Result<Json<EvalRunResponse>, ApiError> {
    let request = to_eval_request(&body);
    let report = tokio::task::spawn_blocking(move || {
        ai_evals::run_evals_request(request)
    })
    .await
    .map_err(|e| ApiError::internal(format!("eval task failed: {e}")))?
    .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(EvalRunResponse {
        run_id: None,
        status: if report.metrics.failed > 0 {
            "failed"
        } else {
            "completed"
        }
        .into(),
        report: Some(map_report(report)),
    }))
}

pub async fn run_eval_async(
    State(state): State<AppState>,
    Json(body): Json<EvalRunApiRequest>,
) -> Result<Json<EvalAsyncRunResponse>, ApiError> {
    let run_id = state.eval_runs.insert_pending();
    let request = to_eval_request(&body);
    spawn_async_eval(state.eval_runs.clone(), run_id.clone(), request);
    Ok(Json(EvalAsyncRunResponse {
        run_id,
        status: "pending".into(),
    }))
}

pub async fn get_eval_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<EvalRunStatusResponse>, ApiError> {
    let record = state
        .eval_runs
        .get(&run_id)
        .ok_or_else(|| ApiError::invalid_request(format!("eval run not found: {run_id}")))?;

    let (status, report, error) = match record.state {
        EvalRunState::Pending => ("pending".to_string(), None, None),
        EvalRunState::Running => ("running".to_string(), None, None),
        EvalRunState::Completed { report } => (
            "completed".to_string(),
            Some(map_report(report)),
            None,
        ),
        EvalRunState::Failed { error } => ("failed".to_string(), None, Some(error)),
    };

    Ok(Json(EvalRunStatusResponse {
        run_id,
        status,
        report,
        error,
    }))
}

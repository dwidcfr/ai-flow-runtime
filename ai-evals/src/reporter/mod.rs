use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::assertions::AssertionFailure;
use crate::metrics::RunMetrics;
use crate::runner::{ScenarioResult, ScenarioStatus};

#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvalReport {
    pub metrics: RunMetrics,
    pub results: Vec<ReportScenario>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportScenario {
    pub name: String,
    pub domain: Option<String>,
    pub status: String,
    pub skip_reason: Option<String>,
    pub failures: Vec<AssertionFailure>,
}

pub fn build_report(metrics: RunMetrics, results: &[ScenarioResult]) -> EvalReport {
    EvalReport {
        metrics,
        results: results.iter().map(report_scenario).collect(),
    }
}

fn report_scenario(result: &ScenarioResult) -> ReportScenario {
    ReportScenario {
        name: result.scenario_name.clone(),
        domain: result.domain.clone(),
        status: status_str(result.status).into(),
        skip_reason: result.skip_reason.clone(),
        failures: result.failures.clone(),
    }
}

pub fn render_text(report: &EvalReport) -> String {
    let m = &report.metrics;
    let mut out = String::new();
    out.push_str("=== AI Flow Evals ===\n");
    out.push_str(&format!("Total:   {}\n", m.total));
    out.push_str(&format!("Passed:  {}\n", m.passed));
    out.push_str(&format!("Failed:  {}\n", m.failed));
    out.push_str(&format!("Skipped: {}\n", m.skipped));
    out.push_str(&format!(
        "Duration: {:.2}s\n",
        m.duration_ms as f64 / 1000.0
    ));
    out.push_str(&format!(
        "Avg scenario: {:.1}ms | router: {:.1}ms | response: {:.1}ms\n",
        m.avg_scenario_ms, m.avg_router_ms, m.avg_response_ms
    ));

    if m.router_errors > 0 || m.response_errors > 0 {
        out.push_str(&format!(
            "Errors: router={} response={}\n",
            m.router_errors, m.response_errors
        ));
    }

    for result in &report.results {
        if result.status == "failed" {
            out.push_str(&format!("\nFAILED: {}\n", result.name));
            for failure in &result.failures {
                out.push_str(&format!(
                    "  [{}] expected={} actual={}\n",
                    failure.field, failure.expected, failure.actual
                ));
            }
        }
    }

    for result in &report.results {
        if result.status == "skipped" {
            out.push_str(&format!(
                "\nSKIPPED: {} ({})\n",
                result.name,
                result.skip_reason.as_deref().unwrap_or("unknown")
            ));
        }
    }

    out
}

pub fn write_report(report: &EvalReport, format: ReportFormat, output: Option<&Path>) -> crate::error::Result<()> {
    let content = match format {
        ReportFormat::Text => render_text(report),
        ReportFormat::Json => serde_json::to_string_pretty(report)?,
    };

    if let Some(path) = output {
        fs::write(path, content)?;
    } else {
        print!("{content}");
    }

    Ok(())
}

pub fn exit_code(results: &[ScenarioResult]) -> i32 {
    let failed = results
        .iter()
        .any(|r| r.status == ScenarioStatus::Failed);
    if failed { 1 } else { 0 }
}

fn status_str(status: ScenarioStatus) -> &'static str {
    match status {
        ScenarioStatus::Passed => "passed",
        ScenarioStatus::Failed => "failed",
        ScenarioStatus::Skipped => "skipped",
    }
}

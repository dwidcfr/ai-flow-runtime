pub mod assertions;
pub mod cli;
pub mod error;
pub mod loader;
pub mod metrics;
pub mod request;
pub mod reporter;
pub mod runner;
pub mod scenario;

pub use error::{EvalError, Result};
pub use loader::ScenarioLoader;
pub use metrics::{collect_metrics, RunMetrics};
pub use request::{run_evals_request, EvalRunRequest};
pub use reporter::{build_report, exit_code, write_report, EvalReport, ReportFormat};
pub use runner::{ScenarioResult, ScenarioRunner, ScenarioStatus};
pub use scenario::{EvalScenario, LlmMode};

use std::path::PathBuf;

pub fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn run_evals(cli: &cli::Cli) -> Result<i32> {
    let request = EvalRunRequest {
        scenario: cli.scenario.clone(),
        domain: cli.domain.clone(),
        llm: cli.llm.into(),
    };
    let report = run_evals_request(request)?;

    let format = match cli.format {
        cli::FormatArg::Text => ReportFormat::Text,
        cli::FormatArg::Json => ReportFormat::Json,
    };

    write_report(
        &report,
        format,
        cli.output.as_deref().map(std::path::Path::new),
    )?;

    Ok(exit_code_from_report(&report))
}

fn exit_code_from_report(report: &EvalReport) -> i32 {
    if report.metrics.failed > 0 {
        1
    } else {
        0
    }
}

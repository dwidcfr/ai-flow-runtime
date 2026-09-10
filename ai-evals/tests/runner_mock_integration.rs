use ai_evals::{crate_root, ScenarioLoader, ScenarioRunner};

#[test]
fn runner_passes_greeting_confirm() {
    let loader = ScenarioLoader::new(&crate_root()).unwrap();
    let scenario = loader.load_by_name("greeting_confirm").unwrap();
    let runner = ScenarioRunner::new(loader.fixtures().clone());
    let result = runner.run(&scenario);
    assert_eq!(result.status, ai_evals::ScenarioStatus::Passed, "{:?}", result.failures);
}

#[test]
fn runner_passes_payment_methods() {
    let loader = ScenarioLoader::new(&crate_root()).unwrap();
    let scenario = loader.load_by_name("payment_methods").unwrap();
    let runner = ScenarioRunner::new(loader.fixtures().clone());
    let result = runner.run(&scenario);
    assert_eq!(result.status, ai_evals::ScenarioStatus::Passed, "{:?}", result.failures);
}

#[test]
fn runner_passes_flow_resume() {
    let loader = ScenarioLoader::new(&crate_root()).unwrap();
    let scenario = loader.load_by_name("flow_resume").unwrap();
    let runner = ScenarioRunner::new(loader.fixtures().clone());
    let result = runner.run(&scenario);
    assert_eq!(result.status, ai_evals::ScenarioStatus::Passed, "{:?}", result.failures);
}

#[test]
fn run_evals_all_mock_passes() {
    let cli = ai_evals::cli::Cli {
        scenario: None,
        domain: None,
        llm: ai_evals::cli::LlmArg::Mock,
        format: ai_evals::cli::FormatArg::Text,
        output: None,
        verbose: false,
    };
    let code = ai_evals::run_evals(&cli).unwrap();
    assert_eq!(code, 0);
}

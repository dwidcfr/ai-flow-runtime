use ai_evals::cli::Cli;
use ai_evals::run_evals;
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("ai_evals=debug")
            .init();
    }

    match run_evals(&cli) {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(2);
        }
    }
}

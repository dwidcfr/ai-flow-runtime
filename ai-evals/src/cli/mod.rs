use clap::{Parser, ValueEnum};

use crate::scenario::LlmMode;

#[derive(Debug, Parser)]
#[command(name = "ai-evals", about = "AI Flow evaluation harness")]
pub struct Cli {
    /// Run a single scenario by name (substring match)
    pub scenario: Option<String>,

    /// Filter scenarios by domain folder
    #[arg(long)]
    pub domain: Option<String>,

    /// LLM backend override
    #[arg(long, value_enum, default_value = "mock")]
    pub llm: LlmArg,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    pub format: FormatArg,

    /// Write report to file instead of stdout
    #[arg(long)]
    pub output: Option<String>,

    /// Verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LlmArg {
    Mock,
    Gemini,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FormatArg {
    Text,
    Json,
}

impl From<LlmArg> for LlmMode {
    fn from(value: LlmArg) -> Self {
        match value {
            LlmArg::Mock => LlmMode::Mock,
            LlmArg::Gemini => LlmMode::Gemini,
        }
    }
}

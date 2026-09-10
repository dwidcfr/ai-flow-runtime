mod parser;

pub use parser::JsonDecisionParser;

use crate::error::Result;
use crate::types::ParsedDecision;

pub trait DecisionParser: Send + Sync {
    fn parse(&self, raw: &str) -> Result<ParsedDecision>;
}

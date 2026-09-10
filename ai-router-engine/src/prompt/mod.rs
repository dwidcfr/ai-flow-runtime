mod builder;

pub use builder::DefaultPromptBuilder;

use crate::types::RouterContext;

pub trait PromptBuilder: Send + Sync {
    fn build(&self, context: &RouterContext, normalized_message: &str) -> String;
}

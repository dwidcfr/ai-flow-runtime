mod builder;

pub use builder::DefaultPromptBuilder;

use crate::types::ResponseContext;

pub trait PromptBuilder: Send + Sync {
    fn build(&self, context: &ResponseContext) -> String;
}

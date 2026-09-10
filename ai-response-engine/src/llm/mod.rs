mod mock;

pub use mock::MockResponseLLM;

use crate::error::Result;

pub trait ResponseLLM: Send + Sync {
    fn complete(&self, prompt: &str) -> Result<String>;
}

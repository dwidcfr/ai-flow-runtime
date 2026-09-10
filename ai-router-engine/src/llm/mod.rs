mod mock;

pub use mock::MockRouterLLM;

use crate::error::Result;

pub trait RouterLLM: Send + Sync {
    fn complete(&self, prompt: &str) -> Result<String>;
}

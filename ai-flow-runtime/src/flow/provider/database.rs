use super::FlowProvider;
use crate::error::{Result, RuntimeError};
use crate::flow::FlowFacade;

#[derive(Debug, Default, Clone)]
pub struct DatabaseFlowProvider;

impl DatabaseFlowProvider {
    pub fn new() -> Self {
        Self
    }
}

impl FlowProvider for DatabaseFlowProvider {
    fn load(&self, _flow_ref: &str) -> Result<FlowFacade> {
        Err(RuntimeError::FlowProviderNotImplemented("database"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_provider_returns_not_implemented() {
        let provider = DatabaseFlowProvider::new();
        let err = provider.load("payment_flow").unwrap_err();
        assert!(matches!(
            err,
            RuntimeError::FlowProviderNotImplemented("database")
        ));
    }
}

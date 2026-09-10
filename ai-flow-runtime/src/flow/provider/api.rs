use super::FlowProvider;
use crate::error::{Result, RuntimeError};
use crate::flow::FlowFacade;

#[derive(Debug, Default, Clone)]
pub struct ApiFlowProvider;

impl ApiFlowProvider {
    pub fn new() -> Self {
        Self
    }
}

impl FlowProvider for ApiFlowProvider {
    fn load(&self, _flow_ref: &str) -> Result<FlowFacade> {
        Err(RuntimeError::FlowProviderNotImplemented("api"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_provider_returns_not_implemented() {
        let provider = ApiFlowProvider::new();
        let err = provider.load("payment_flow").unwrap_err();
        assert!(matches!(
            err,
            RuntimeError::FlowProviderNotImplemented("api")
        ));
    }
}

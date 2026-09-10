use std::path::Path;

use ai_flow_engine::FlowLoader;

use super::FlowProvider;
use crate::error::{Result, RuntimeError};
use crate::flow::FlowFacade;

#[derive(Debug, Default, Clone)]
pub struct YamlFlowProvider;

impl YamlFlowProvider {
    pub fn new() -> Self {
        Self
    }
}

impl FlowProvider for YamlFlowProvider {
    fn load(&self, flow_ref: &str) -> Result<FlowFacade> {
        let flow = FlowLoader::load(Path::new(flow_ref)).map_err(|e| RuntimeError::FlowEngineError {
            message: e.to_string(),
        })?;
        Ok(FlowFacade::from_engine(flow))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn data_path(relative: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
    }

    #[test]
    fn yaml_provider_loads_flow() {
        let provider = YamlFlowProvider::new();
        let facade = provider
            .load(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
            .unwrap();
        assert_eq!(facade.id(), "payment_flow");
        assert_eq!(facade.initial_node(), "greeting");
    }
}

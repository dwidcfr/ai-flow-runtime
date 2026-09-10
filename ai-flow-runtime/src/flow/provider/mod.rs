mod api;
mod database;
mod yaml;

use crate::error::Result;
use crate::flow::FlowFacade;

pub trait FlowProvider: Send + Sync {
    fn load(&self, flow_ref: &str) -> Result<FlowFacade>;
}

pub use api::ApiFlowProvider;
pub use database::DatabaseFlowProvider;
pub use yaml::YamlFlowProvider;

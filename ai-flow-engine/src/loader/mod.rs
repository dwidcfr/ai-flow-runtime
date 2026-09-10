use std::fs;
use std::path::Path;

use crate::error::{FlowError, Result};
use crate::model::Flow;
use crate::parse::{build_flow, parse_yaml};
use crate::validate::validate_flow;

pub struct FlowLoader;

impl FlowLoader {
    pub fn load(path: impl AsRef<Path>) -> Result<Flow> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|e| FlowError::Io {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;
        Self::from_str(&content)
    }

    pub fn from_str(yaml: &str) -> Result<Flow> {
        let doc = parse_yaml(yaml)?;
        let flow = build_flow(doc);
        validate_flow(&flow)?;
        Ok(flow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("ai-flow-runtime/data/flows/payment_flow.yaml")
    }

    #[test]
    fn load_fixture_file() {
        let flow = FlowLoader::load(fixture_path()).unwrap();
        assert_eq!(flow.id(), "payment_flow");
        assert_eq!(flow.initial_node(), "greeting");
    }
}

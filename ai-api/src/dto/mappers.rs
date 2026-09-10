use ai_flow_runtime::RuntimeState;

use crate::dto::session::{ActiveModuleDto, RuntimeStateDto};

pub fn runtime_state_to_dto(state: RuntimeState) -> RuntimeStateDto {
    state.into()
}

pub fn map_active_module(module_id: &str, section: Option<String>) -> ActiveModuleDto {
    ActiveModuleDto {
        module_id: module_id.to_string(),
        section,
    }
}

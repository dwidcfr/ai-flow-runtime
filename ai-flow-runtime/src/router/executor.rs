use ai_router_engine::RouterDecision;

use crate::error::{Result, RuntimeError};
use crate::runtime::Runtime;

pub fn execute_router_decision(
    runtime: &mut Runtime,
    session_id: &str,
    decision: RouterDecision,
) -> Result<()> {
    match decision {
        RouterDecision::Flow { action, .. } => {
            runtime.goto_transition(session_id, &action)?;
        }
        RouterDecision::Module {
            module_id,
            section_id,
            ..
        } => {
            let module_ref = match section_id {
                Some(section) => format!("{module_id}.{section}"),
                None => module_id,
            };
            runtime.pause_flow(session_id)?;
            runtime.open_module(session_id, &module_ref)?;
        }
        RouterDecision::Clarify { .. } => {}
        RouterDecision::EndConversation { .. } => {
            runtime.finish_session(session_id)?;
        }
        RouterDecision::Error { message, .. } => {
            return Err(RuntimeError::RouterError(message));
        }
    }
    Ok(())
}

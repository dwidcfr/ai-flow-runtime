use serde::{Deserialize, Serialize};

use crate::error::{Result, RuntimeError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuntimeState {
    Idle,
    Flow,
    Paused,
    Module,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeAction {
    StartFlow,
    Transition,
    PauseFlow,
    ResumeFlow,
    OpenModule,
    CloseModule,
    FinishSession,
    DestroySession,
}

pub fn can_transition(from: RuntimeState, action: RuntimeAction) -> Result<RuntimeState> {
    if action == RuntimeAction::DestroySession {
        return Ok(from);
    }

    let next = match (from, action) {
        (RuntimeState::Idle, RuntimeAction::StartFlow) => RuntimeState::Flow,

        (RuntimeState::Flow, RuntimeAction::Transition) => RuntimeState::Flow,
        (RuntimeState::Flow, RuntimeAction::PauseFlow) => RuntimeState::Paused,
        (RuntimeState::Flow, RuntimeAction::FinishSession) => RuntimeState::Finished,

        (RuntimeState::Paused, RuntimeAction::OpenModule) => RuntimeState::Module,
        (RuntimeState::Paused, RuntimeAction::ResumeFlow) => RuntimeState::Flow,

        (RuntimeState::Module, RuntimeAction::CloseModule) => RuntimeState::Flow,

        _ => {
            return Err(RuntimeError::InvalidStateTransition { from, action });
        }
    };

    Ok(next)
}

pub fn ensure_active(from: RuntimeState) -> Result<()> {
    match from {
        RuntimeState::Finished => Err(RuntimeError::SessionFinished),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_to_flow_on_start() {
        assert_eq!(
            can_transition(RuntimeState::Idle, RuntimeAction::StartFlow).unwrap(),
            RuntimeState::Flow
        );
    }

    #[test]
    fn flow_to_paused() {
        assert_eq!(
            can_transition(RuntimeState::Flow, RuntimeAction::PauseFlow).unwrap(),
            RuntimeState::Paused
        );
    }

    #[test]
    fn paused_to_module() {
        assert_eq!(
            can_transition(RuntimeState::Paused, RuntimeAction::OpenModule).unwrap(),
            RuntimeState::Module
        );
    }

    #[test]
    fn module_to_flow_on_close() {
        assert_eq!(
            can_transition(RuntimeState::Module, RuntimeAction::CloseModule).unwrap(),
            RuntimeState::Flow
        );
    }

    #[test]
    fn paused_to_flow_on_resume() {
        assert_eq!(
            can_transition(RuntimeState::Paused, RuntimeAction::ResumeFlow).unwrap(),
            RuntimeState::Flow
        );
    }

    #[test]
    fn flow_to_finished() {
        assert_eq!(
            can_transition(RuntimeState::Flow, RuntimeAction::FinishSession).unwrap(),
            RuntimeState::Finished
        );
    }

    #[test]
    fn open_module_without_pause_fails() {
        let err = can_transition(RuntimeState::Flow, RuntimeAction::OpenModule).unwrap_err();
        assert!(matches!(
            err,
            RuntimeError::InvalidStateTransition { .. }
        ));
    }

    #[test]
    fn transition_while_paused_fails() {
        let err = can_transition(RuntimeState::Paused, RuntimeAction::Transition).unwrap_err();
        assert!(matches!(
            err,
            RuntimeError::InvalidStateTransition { .. }
        ));
    }

    #[test]
    fn close_module_without_open_fails() {
        let err = can_transition(RuntimeState::Flow, RuntimeAction::CloseModule).unwrap_err();
        assert!(matches!(
            err,
            RuntimeError::InvalidStateTransition { .. }
        ));
    }
}

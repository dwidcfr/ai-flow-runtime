use std::collections::HashMap;

use crate::error::{Result, RuntimeError};
use crate::session::Session;

#[derive(Default)]
pub struct InMemorySessionStore {
    sessions: HashMap<String, Session>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, session: Session) {
        self.sessions.insert(session.session_id.clone(), session);
    }

    pub fn get(&self, session_id: &str) -> Result<&Session> {
        self.sessions
            .get(session_id)
            .ok_or_else(|| RuntimeError::SessionNotFound(session_id.to_string()))
    }

    pub fn get_mut(&mut self, session_id: &str) -> Result<&mut Session> {
        self.sessions
            .get_mut(session_id)
            .ok_or_else(|| RuntimeError::SessionNotFound(session_id.to_string()))
    }

    pub fn remove(&mut self, session_id: &str) -> Result<()> {
        self.sessions
            .remove(session_id)
            .map(|_| ())
            .ok_or_else(|| RuntimeError::SessionNotFound(session_id.to_string()))
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn list_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.sessions.keys().cloned().collect();
        ids.sort();
        ids
    }
}

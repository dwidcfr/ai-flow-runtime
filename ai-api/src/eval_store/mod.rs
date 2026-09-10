use std::sync::{Arc, Mutex};
use std::time::Instant;

use ai_evals::{EvalReport, EvalRunRequest, run_evals_request};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum EvalRunState {
    Pending,
    Running,
    Completed { report: EvalReport },
    Failed { error: String },
}

#[derive(Debug, Clone)]
pub struct EvalRunRecord {
    pub run_id: String,
    pub state: EvalRunState,
    pub started_at: Instant,
}

#[derive(Clone, Default)]
pub struct EvalRunStore {
    inner: Arc<Mutex<std::collections::HashMap<String, EvalRunRecord>>>,
}

impl EvalRunStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_pending(&self) -> String {
        let run_id = Uuid::new_v4().to_string();
        let mut guard = self.inner.lock().expect("eval store lock");
        guard.insert(
            run_id.clone(),
            EvalRunRecord {
                run_id: run_id.clone(),
                state: EvalRunState::Pending,
                started_at: Instant::now(),
            },
        );
        run_id
    }

    pub fn set_running(&self, run_id: &str) {
        let mut guard = self.inner.lock().expect("eval store lock");
        if let Some(record) = guard.get_mut(run_id) {
            record.state = EvalRunState::Running;
        }
    }

    pub fn set_completed(&self, run_id: &str, report: EvalReport) {
        let mut guard = self.inner.lock().expect("eval store lock");
        if let Some(record) = guard.get_mut(run_id) {
            record.state = EvalRunState::Completed { report };
        }
    }

    pub fn set_failed(&self, run_id: &str, error: String) {
        let mut guard = self.inner.lock().expect("eval store lock");
        if let Some(record) = guard.get_mut(run_id) {
            record.state = EvalRunState::Failed { error };
        }
    }

    pub fn get(&self, run_id: &str) -> Option<EvalRunRecord> {
        self.inner.lock().ok()?.get(run_id).cloned()
    }
}

pub fn spawn_async_eval(store: EvalRunStore, run_id: String, request: EvalRunRequest) {
    tokio::task::spawn_blocking(move || {
        store.set_running(&run_id);
        match run_evals_request(request) {
            Ok(report) => store.set_completed(&run_id, report),
            Err(e) => store.set_failed(&run_id, e.to_string()),
        }
    });
}

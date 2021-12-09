use std::sync::Arc;
use std::sync::RwLock;

/* ---------- */

enum RunStatePrivate {
    Running,
    Aborted,
    RunnerDropped,
}

impl Default for RunStatePrivate {
    fn default() -> Self {
        Self::Running
    }
}

/* ---------- */

pub struct RunState(Arc<RwLock<RunStatePrivate>>);

impl RunState {
    pub fn abort(&mut self) {
        if let Ok(mut state) = self.0.write() {
            *state = RunStatePrivate::Aborted
        }
    }

    pub fn is_running(&self) -> bool {
        match self.0.read() {
            Ok(state) => matches!(*state, RunStatePrivate::Running),
            _ => false
        }
    }

    pub(crate) fn runner_dropped_abort(&mut self) {
        if let Ok(mut state) = self.0.write() {
            *state = RunStatePrivate::RunnerDropped
        }
    }

    pub(crate) fn has_runner_dropped(&self) -> bool {
        match self.0.read() {
            Ok(state) => matches!(*state, RunStatePrivate::RunnerDropped),
            _ => false
        }
    }
}

impl Default for RunState {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(RunStatePrivate::default())))
    }
}

impl Clone for RunState {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

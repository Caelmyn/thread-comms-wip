use std::sync::Arc;

use crate::run_state::RunState;

pub struct Runtime<T: Send + 'static>(Arc<dyn Fn(&RunState, T) + Sync + Send + 'static>);

impl<T: Send + 'static> Runtime<T> {
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(&RunState, T) + Send + Sync + 'static
    {
        Self(Arc::new(func))
    }

    pub(crate) fn run(&self, state: &RunState, arg: T) {
        self.0(state, arg);
    }
}

impl<T:  Send + 'static> Clone for Runtime<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

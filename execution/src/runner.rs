use std::thread::{self, JoinHandle};

use conversation::MessageSender;

use crate::run_state::RunState;
use crate::runtime::Runtime;
use crate::commands::Abort;

/* ---------- */

pub(crate) struct Runner {
    state: RunState,
    thread: Option<JoinHandle<()>>
}

impl Runner {
    pub(crate) fn spawn<T: Send + 'static>(name: &'static str, runtime: Runtime<T>, arg: T, chan: MessageSender) -> Self {
        let state = RunState::default();
        let state_ref = state.clone();

        let thread = thread::spawn(move || {
            runtime.run(&state_ref, arg);

            if !state_ref.has_runner_dropped() {
                println!("notif terminated");
                let e = chan.send_msg(Abort(name));
                println!("notif sent {}", e.is_err());
            }
        });

        Self {
            state,
            thread: Some(thread)
        }
    }

    pub(crate) fn wait(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            self.state.runner_dropped_abort();
            let _ = thread.join();
        }
    }
}

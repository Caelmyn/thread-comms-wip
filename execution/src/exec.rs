use std::collections::HashMap;

use conversation::{Unit, Builder, WithContext, MessageSender, Content, IntoContent};

use crate::runner::Runner;
use crate::commands::*;
use crate::runtime::Runtime;

/* ---------- */

pub struct Exec<T: Send + 'static> {
    channel: Option<MessageSender>,
    runners: HashMap<&'static str, Runner>,
    runtime: Runtime<T>
}

impl<T: Send + 'static> Exec<T> {
    pub fn build(runtime: Runtime<T>) -> Builder<'static, Self> {
        Self {
            channel: None,
            runners: HashMap::default(),
            runtime
        }.build_unit().with_context()
    }

    fn new_task(&mut self, name: &'static str, arg: T) {
        if let Some(channel) = &self.channel {
            if self.runners.contains_key(name) {
                return
            }

            let name_ref = <&'static str>::clone(&name);
            let chan_ref = channel.clone();

            let runner = Runner::spawn(name_ref, self.runtime.clone(), arg, chan_ref);
            self.runners.insert(name, runner);
        }
    }

    #[inline(always)]
    fn abort(&mut self, name: &'static str) {
        self.runners.remove(name);
    }

    fn wait(&mut self) {
        self.runners.iter_mut().for_each(|(_, runner)| runner.wait());
        self.runners.clear()
    }
}

impl<T: Send + 'static> Unit for Exec<T> {
    fn on_message(&mut self, msg: Content) {
        if msg.is::<Spawn<T>>() {
            if let Some(Spawn::<T>(name, arg)) = msg.into() {
                self.new_task(name, arg)
            }
        } else if msg.is::<Abort>() {
            if let Some(Abort(name)) = msg.into() {
                self.abort(name)
            }
        } else if msg.is::<Wait>() {
            if let Some(Wait) = msg.into() {
                self.wait()
            }
        }
    }

    fn on_message_with_reply(&mut self, _: Content) -> Content { ().into_content() }
}

impl<T: Send + 'static> WithContext for Exec<T> {
    fn with_context(&mut self, channel: MessageSender) {
        self.channel = Some(channel)
    }
}

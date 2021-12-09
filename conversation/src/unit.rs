use std::any::Any;
use std::thread::{self, JoinHandle};

use crate::{SendMessageWithReplyError, SendMessageError, Cluster, ClusterError};
use crate::message::{Content, Message};
use crate::channel::{self, MessageReceiver, MessageSender};

/* ---------- */

pub trait Unit: Send + 'static {
    fn on_message(&mut self, data: Content);
    fn on_message_with_reply(&mut self, data: Content) -> Content;

    fn build_unit(self) -> Builder<'static, Self> where Self: Sized {
        Builder::<Self>::new(self)
    }
}

/* ---------- */

pub trait WithContext {
    fn with_context(&mut self, channel: MessageSender);
}

/* ---------- */

pub struct Pipe {
    thread: Option<JoinHandle<()>>,
    msg_send: MessageSender
}

impl Pipe {
    pub fn builder<T: Unit + Send + 'static>(obj: T) -> Builder<'static, T> {
        Builder::new(obj)
    }

    pub fn send<M: Any + Send + 'static>(&self, data: M) -> Result<(), SendMessageError> {
        self.msg_send.send_msg(data)
    }

    pub fn send_with_reply<M, R>(&self, data: M) -> Result<R, SendMessageWithReplyError>
    where
        M: Any + Send + 'static,
        R: Any + Send + 'static
    {
        self.msg_send.send_msg_with_reply(data)?;
        let reply = self.msg_send.recv_reply()?;

        match reply.into::<R>() {
            Some(val) => Ok(val),
            None => Err(SendMessageWithReplyError::ConvertContentError)
        }
    }
}

impl Drop for Pipe {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = self.msg_send.disconnect();
            let _ = thread.join();
        }
    }
}

/* ---------- */

pub struct Builder<'a, T> {
    obj: T,
    sender: MessageSender,
    recver: MessageReceiver,
    cluster: Option<&'a mut Cluster>,
    id: Option<&'static str>
}

impl<'a, T: Unit + Send + 'static> Builder<'a, T> {
    pub(crate) fn new(obj: T) -> Self {
        let (send, recv) = channel::channel();

        Self {
            obj,
            sender: send,
            recver: recv,
            cluster: None,
            id: None
        }
    }

    pub fn spawn_pipe(self) -> Pipe {
        let thread = thread::spawn(move || receive_loop_thread(self.obj, self.recver));

        Pipe {
            thread: Some(thread),
            msg_send: self.sender
        }
    }

    pub fn spawn(self) -> Result<(), ClusterError<'a>> {
        let cluster = self.cluster.ok_or(ClusterError::RegistrationError)?;
        let id = self.id.ok_or(ClusterError::UnsetIdError)?;

        cluster.add_unique(id, self.obj, self.sender, self.recver)
    }

    pub fn with_name(mut self, id: &'static str) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_cluster(mut self, cluster_ref: &'a mut Cluster) -> Self {
        self.cluster = Some(cluster_ref);
        self
    }
}

impl<T: WithContext> Builder<'_, T> {
    pub fn with_context(mut self) -> Self {
        self.obj.with_context(self.sender.clone());
        self
    }
}

/* ---------- */

fn receive_loop_thread<T: Unit>(mut obj: T, recv: MessageReceiver) {

    while let Ok(msg) = recv.recv_msg() {
        match msg {
            Message::Disconnect => return,
            Message::Simple(content) => {
                obj.on_message(content)
            }
            Message::WithReply(content) => {
                let reply = obj.on_message_with_reply(content);
                if let Err(err) = recv.send_reply(reply) {
                    println!("failed to send reply : {}", err);
                    return;
                }
            }
        }
    }

    println!("failed to rev msg")
}

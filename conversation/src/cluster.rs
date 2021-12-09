use std::any::Any;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::thread::{self, JoinHandle};

use crossbeam_channel::{Sender, Receiver, Select};

use crate::{IntoMessage, Unit, MessageReceiver, MessageSender};
use crate::error::{RecvError, SendReplyError, ClusterError};
use crate::message::{Message, Content};
use crate::unit::Builder;

/* ---------- */

enum ClusterMessage {
    NewMessageEvent(MessageEventHandle),
    Stop
}
/* ---------- */

pub struct Cluster {
    thread: Option<JoinHandle<()>>,
    inner_sender: Sender<ClusterMessage>,
    msger_pool: HashMap<&'static str, MessageSender>
}

impl Cluster {
    pub fn new() -> Self {
        let (send, recv) = crossbeam_channel::bounded(0);

        let thread = thread::spawn(|| group_recv_loop_thread(recv));

        Self {
            thread: Some(thread),
            inner_sender: send,
            msger_pool: HashMap::new()
        }
    }
}

impl Cluster {
    pub fn register<T>(&mut self, obj: T) -> Builder<T>
    where
        T: Unit + Send + 'static
    {
        Builder::new(obj).with_cluster(self)
    }

    pub fn remove<'a>(&mut self, id: &'a str) -> Result<(), ClusterError<'a>> {
        if let Some(sender) = self.msger_pool.remove(id) {
            let _ = sender.send_msg(Message::Disconnect);
            return Ok(())
        }

        Err(ClusterError::IdNotFound(id))
    }

    pub fn send_to<'a, T>(&self, id: &'a str, data: T) -> Result<(), ClusterError<'a>>
    where
        T: Any + Send + 'static
    {
        if let Some(sender) = self.msger_pool.get(id) {
            if sender.send_msg(data.into_msg()).is_err() {
                return Err(ClusterError::AlreadyDisconnected)
            }

            return Ok(())
        }

        Err(ClusterError::IdNotFound(id))
    }

    pub fn send_to_with_reply<'a, T, R>(&self, id: &'a str, data: T) -> Result<R, ClusterError<'a>>
    where
        T: Any + Send + 'static,
        R: Any + Send + 'static,
    {
        if let Some(sender) = self.msger_pool.get(id) {
            if sender.send_msg_with_reply(data).is_err() {
                return Err(ClusterError::AlreadyDisconnected)
            }

            match sender.recv_reply() {
                Ok(reply) => return reply.into::<R>().ok_or(ClusterError::ContentConversionError),
                _ => return Err(ClusterError::AlreadyDisconnected)
            }
        }

        Err(ClusterError::IdNotFound(id))
    }

    pub(crate) fn add_unique<T>(&mut self, id: &'static str, obj: T, tx: MessageSender, rx: MessageReceiver) -> Result<(), ClusterError>
    where
        T: Unit + Send + 'static
    {
        if self.msger_pool.get(&id).is_none() {
            let handle = MessageEventHandle::new(obj, rx);

            if self.inner_sender.send(ClusterMessage::NewMessageEvent(handle)).is_err() {
                return Err(ClusterError::RegistrationError)
            }

            self.msger_pool.insert(id, tx);
            return Ok(())
        }

        Err(ClusterError::IdAlreadyUsed(id))
    }
}

impl Drop for Cluster {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = self.inner_sender.send(ClusterMessage::Stop);
            let _ = thread.join();
        }
    }
}

impl Default for Cluster {
    fn default() -> Self {
        Self::new()
    }
}

/* ---------- */

struct MessageEventHandle {
    msg_event: Box<dyn Unit + 'static>,
    rx: MessageReceiver
}

impl MessageEventHandle {
    fn new<T: Unit + Send + 'static>(obj: T, recv: MessageReceiver) -> Self {
        Self {
            msg_event: Box::new(obj),
            rx: recv
        }
    }

    fn recv(&self) -> Result<Message, RecvError> {
        self.rx.recv_msg()
    }

    fn send(&self, data: Content) -> Result<(), SendReplyError> {
        self.rx.send_reply(data)
    }

    fn inner_recver(&self) -> &Receiver<Message> {
        self.rx.msg_recver()
    }
}

impl Deref for MessageEventHandle {
    type Target = dyn Unit + 'static;
    fn deref(&self) -> &Self::Target {
        &*self.msg_event
    }
}

impl DerefMut for MessageEventHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.msg_event
    }
}

/* ---------- */

#[derive(Default)]
struct HandleList(Vec<(usize, MessageEventHandle)>);

impl HandleList {
    fn new() -> Self {
        Self::default()
    }

    fn register_all<'a>(&'a mut self, sel: &mut Select<'a>) {
        self.0.iter_mut().for_each(|(idx, handle)| {
            *idx = sel.recv(handle.inner_recver());
        });
    }

    fn add(&mut self, handle: MessageEventHandle) {
        self.0.push((0, handle))
    }

    fn remove(&mut self, idx: usize) {
        if let Some(pos) = self.0.iter().position(|(handle_idx, _)| idx == *handle_idx) {
            self.0.remove(pos);
        }
    }

    fn get_handle_mut(&mut self, idx: usize) -> Option<&mut MessageEventHandle> {
        self.0.iter_mut()
            .find(|(handle_idx, _)| idx == *handle_idx)
            .map(|(_, handle)| handle)
    }
}

/* ---------- */

fn group_recv_loop_thread(group_recv: Receiver<ClusterMessage>) {
    let mut handles = HandleList::new();

    loop {
        let mut sel = Select::new();
        let group_index = sel.recv(&group_recv);

        handles.register_all(&mut sel);

        let idx = sel.ready();

        if idx == group_index {
            match group_recv.recv() {
                Ok(ClusterMessage::Stop) => return,
                Ok(ClusterMessage::NewMessageEvent(handle)) => {
                    handles.add(handle)
                }
                _ => return
            }
        } else if let Some(handle) = handles.get_handle_mut(idx) {
            match handle.recv() {
                Ok(Message::Simple(data)) => handle.on_message(data),
                Ok(Message::WithReply(data)) => {
                    let reply = handle.on_message_with_reply(data);
                    let _ = handle.send(reply);
                }
                Ok(Message::Disconnect) => handles.remove(idx),
                Err(_) => handles.remove(idx)
            }
        }
    }
}

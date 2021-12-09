use std::any::Any;

use crossbeam_channel::{Sender, Receiver};

use crate::message::{Message, Content};
use crate::{RecvError, SendMessageError, SendReplyError, IntoMessage};

/* ---------- */

pub struct MessageSender {
    msg_sender: Sender<Message>,
    reply_recver: Receiver<Content>
}

impl MessageSender {
    pub fn send_msg<T: Any + Send + 'static>(&self, msg: T) -> Result<(), SendMessageError> {
        Ok(self.msg_sender.send(msg.into_msg())?)
    }

    pub fn send_msg_with_reply<T: Any + Send + 'static>(&self, msg: T) -> Result<(), SendMessageError> {
        Ok(self.msg_sender.send(msg.into_msg_with_reply())?)
    }

    pub fn recv_reply(&self) -> Result<Content, RecvError> {
        Ok(self.reply_recver.recv()?)
    }

    pub(crate) fn disconnect(&self) {
        let _ = self.msg_sender.send(Message::Disconnect);
    }

    pub(crate) fn from(msg_send: Sender<Message>, reply_recv: Receiver<Content>) -> Self {
        Self {
            msg_sender: msg_send,
            reply_recver: reply_recv
        }
    }
}

impl Clone for MessageSender {
    fn clone(&self) -> Self {
        Self {
            msg_sender: self.msg_sender.clone(),
            reply_recver: self.reply_recver.clone()
        }
    }
}

/* ---------- */

pub struct MessageReceiver {
    msg_recver: Receiver<Message>,
    reply_sender: Sender<Content>
}

impl MessageReceiver {
    pub fn recv_msg(&self) -> Result<Message, RecvError> {
        Ok(self.msg_recver.recv()?)
    }

    pub fn send_reply(&self, reply: Content) -> Result<(), SendReplyError> {
        Ok(self.reply_sender.send(reply)?)
    }

    pub(crate) fn from(msg_recv: Receiver<Message>, reply_send: Sender<Content>) -> Self {
        Self {
            msg_recver: msg_recv,
            reply_sender: reply_send
        }
    }

    pub(crate) fn msg_recver(&self) -> &Receiver<Message> {
        &self.msg_recver
    }
}

impl Clone for MessageReceiver {
    fn clone(&self) -> Self {
        Self {
            msg_recver: self.msg_recver.clone(),
            reply_sender: self.reply_sender.clone()
        }
    }
}

/* ---------- */

pub fn channel() -> (MessageSender, MessageReceiver) {
    let (msg_send, msg_recv) = crossbeam_channel::unbounded();
    let (reply_send, reply_recv) = crossbeam_channel::unbounded();

    (MessageSender::from(msg_send, reply_recv), MessageReceiver::from(msg_recv, reply_send))
}

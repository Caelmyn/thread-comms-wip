use std::error::Error;
use std::fmt::{Display, Debug, Formatter, Result};

use crate::message::{Content, Message};

/* ---------- */

#[derive(Debug)]
pub struct ConvertContentError;

impl Error for ConvertContentError {}

impl Display for ConvertContentError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "failed to convert Content into given type")
    }
}

/* ---------- */

#[derive(Debug)]
pub struct RecvError;

impl Error for RecvError {}

impl Display for RecvError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "failed to recv: channel empty or disconnected")
    }
}

impl From<crossbeam_channel::RecvError> for RecvError {
    fn from(_: crossbeam_channel::RecvError) -> Self {
        Self
    }
}

/* ---------- */

pub struct SendMessageError(Message);

impl SendMessageError {
    pub fn into<T: 'static>(self) -> Option<T> {
        self.0.into()
    }
}

impl Error for SendMessageError {}

impl Display for SendMessageError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "failed to send: channel disconnected")
    }
}

impl Debug for SendMessageError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "SendError(...)")
    }
}

impl From<crossbeam_channel::SendError<Message>> for SendMessageError {
    fn from(err: crossbeam_channel::SendError<Message>) -> Self {
        Self(err.into_inner())
    }
}

/* ---------- */

pub struct SendReplyError(Content);

impl SendReplyError {
    pub fn into<T: 'static>(self) -> Option<T> {
        self.0.into()
    }
}

impl Error for SendReplyError {}

impl Display for SendReplyError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "failed to send: channel disconnected")
    }
}

impl Debug for SendReplyError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "SendError(...)")
    }
}

impl From<crossbeam_channel::SendError<Content>> for SendReplyError {
    fn from(err: crossbeam_channel::SendError<Content>) -> Self {
        Self(err.into_inner())
    }
}

/* ---------- */

pub enum SendMessageWithReplyError {
    SendError(Message),
    RecvError,
    ConvertContentError
}

impl SendMessageWithReplyError {
    pub fn into<T: 'static>(self) -> Option<T> {
        match self {
            Self::SendError(msg) => msg.into(),
            _ => None
        }
    }
}

impl Error for SendMessageWithReplyError {}

impl Display for SendMessageWithReplyError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::SendError(_) => write!(f, "failed to send: channel disconnected"),
            Self::RecvError => write!(f, "failed to recv reply: channel disconnected"),
            Self::ConvertContentError => write!(f, "failed to convert Content into given type")
        }
    }
}

impl Debug for SendMessageWithReplyError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::SendError(_) => write!(f, "SendError(...)"),
            Self::RecvError => write!(f, "RecvError"),
            Self::ConvertContentError => write!(f, "ConvertContentError")
        }
    }
}

impl From<SendMessageError> for SendMessageWithReplyError {
    fn from(err: SendMessageError) -> Self {
        Self::SendError(err.0)
    }
}

impl From<RecvError> for SendMessageWithReplyError {
    fn from(_: RecvError) -> Self {
        Self::RecvError
    }
}

impl From<ConvertContentError> for SendMessageWithReplyError {
    fn from(_: ConvertContentError) -> Self {
        Self::ConvertContentError
    }
}

/* ---------- */

pub enum ClusterError<'a> {
    RegistrationError,
    UnsetIdError,
    ContentConversionError,
    AlreadyDisconnected,
    IdAlreadyUsed(&'a str),
    IdNotFound(&'a str)
}

impl Error for ClusterError<'_> {}

impl Display for ClusterError<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::RegistrationError => write!(f, "failed to register to cluster"),
            Self::UnsetIdError => write!(f, "error: id not set"),
            Self::ContentConversionError => write!(f, "failed to perform conversion"),
            Self::AlreadyDisconnected => write!(f, "already disconnected"),
            Self::IdAlreadyUsed(id) => write!(f, "id {} already in used", id),
            Self::IdNotFound(id) => write!(f, "id {} not found", id)
        }
    }
}

impl Debug for ClusterError<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::RegistrationError => write!(f, "RegistrationError"),
            Self::UnsetIdError => write!(f, "UnsetIdError"),
            Self::ContentConversionError => write!(f, "ContentConversionError"),
            Self::AlreadyDisconnected => write!(f, "Disconnected"),
            Self::IdAlreadyUsed(id) => write!(f, "IdAlreadyUsed({:?})", id),
            Self::IdNotFound(id) => write!(f, "IdNotFound({:?})", id)
        }
    }
}

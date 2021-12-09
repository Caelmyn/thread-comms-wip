use std::any::Any;

/* ---------- */

pub struct Content(Box<dyn Any + Send + 'static>);

impl Content {
    pub fn from<T: Any + Send + 'static>(obj: T) -> Self {
        Content(Box::new(obj))
    }

    pub fn into<T: 'static>(self) -> Option<T> {
        match self.0.downcast::<T>() {
            Ok(boxed) => Some(*boxed),
            _ => None
        }
    }

    pub fn as_ref<T: 'static>(&self) -> Option<&T> {
        self.0.downcast_ref::<T>()
    }

    pub fn is<T: 'static>(&self) -> bool {
        std::any::TypeId::of::<T>() == (*self.0).type_id()
    }
}

/* ---------- */

pub trait IntoContent {
    fn into_content(self) -> Content;
}

impl<T: Any + Send + 'static> IntoContent for T {
    fn into_content(self) -> Content {
        Content::from(self)
    }
}

/* ---------- */

pub enum Message {
    Simple(Content),
    WithReply(Content),
    Disconnect
}

impl Message {
    pub fn into<T: 'static>(self) -> Option<T> {
        match self {
            Self::Simple(cont) | Self::WithReply(cont) => cont.into(),
            _ => None
        }
    }

    pub fn as_ref<T: 'static>(&self) -> Option<&T> {
        match self {
            Self::Simple(cont) | Self::WithReply(cont) => cont.as_ref(),
            _ => None
        }
    }
}

/* ---------- */

pub trait IntoMessage {
    fn into_msg(self) -> Message;
    fn into_msg_with_reply(self) -> Message;
}

impl<T: Any + Send + 'static> IntoMessage for T {
    fn into_msg(self) -> Message {
        Message::Simple(Content::from(self))
    }

    fn into_msg_with_reply(self) -> Message {
        Message::WithReply(Content::from(self))
    }
}

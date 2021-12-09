use std::any::Any;
use conversation::{Content, IntoContent, WithContext, Unit, MessageSender};

struct Dummy;

impl Unit for Dummy {
    fn on_message(&mut self, _: Content) {}
    fn on_message_with_reply(&mut self, _: Content) -> Content {
        ().into_content()
    }
}

struct DummyWithChannel;

impl Unit for DummyWithChannel {
    fn on_message(&mut self, _: Content) {}
    fn on_message_with_reply(&mut self, _: Content) -> Content {
        ().into_content()
    }
}

impl WithContext for DummyWithChannel {
    fn with_context(&mut self, _: MessageSender) {}
}

#[derive(Default)]
struct Type<T: Any + Send + 'static>(std::marker::PhantomData<T>);

impl<T: Any + Send + 'static> Unit for Type<T> {
    fn on_message(&mut self, _: Content) {}
    fn on_message_with_reply(&mut self, data: Content) -> Content {
        let inner = data.into::<T>();

        match inner {
            Some(_) => Ok(()),
            None => Err(())
        }.into_content()
    }
}

#[test]
fn build() {
    let _ = Dummy.build_unit().spawn_pipe();
    let _ = DummyWithChannel.build_unit().with_context().spawn_pipe();
    let _ = Type::<i32>::default().build_unit().spawn_pipe();
}

#[test]
fn send() {
    let msger = Type::<i32>::default().build_unit().spawn_pipe();

    msger.send(0).unwrap();

    let reply: Result<(), ()> = msger.send_with_reply(0).unwrap();
    assert!(reply.is_ok());

    let reply: Result<(), ()> = msger.send_with_reply("hello").unwrap();
    assert!(reply.is_err());
}

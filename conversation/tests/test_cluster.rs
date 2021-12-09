use conversation::{Cluster, ClusterError, Content, IntoContent, Unit};

struct Dummy;

impl Unit for Dummy {
    fn on_message(&mut self, _: Content) {}
    fn on_message_with_reply(&mut self, _: Content) -> Content { ().into_content() }
}

struct DummyString;

impl Unit for DummyString {
    fn on_message(&mut self, _: Content) {}
    fn on_message_with_reply(&mut self, _: Content) -> Content {
        String::from("String").into_content()
    }
}

struct DummyI32;

impl Unit for DummyI32 {
    fn on_message(&mut self, _: Content) {}
    fn on_message_with_reply(&mut self, _: Content) -> Content {
        1.into_content()
    }
}

#[test]
fn manage_cluster() {
    let mut group = Cluster::new();

    group.register(Dummy).with_name("test").spawn().unwrap();
    assert!(group.register(Dummy).with_name("test").spawn().is_err());

    group.remove("test").unwrap();
    assert!(group.remove("test").is_err());

    group.register(Dummy).with_name("1").spawn().unwrap();
    group.register(Dummy).with_name("2").spawn().unwrap();

    group.remove("1").unwrap();
    group.remove("2").unwrap();

}

#[test]
fn send_to() {
    let mut group = Cluster::new();

    group.register(DummyI32).with_name("i32").spawn().unwrap();
    group.register(DummyString).with_name("String").spawn().unwrap();

    group.send_to("i32", ()).unwrap();
    group.send_to("String", ()).unwrap();

    assert!(group.send_to("foo", ()).is_err());
}

#[test]
fn send_to_with_reply() {
    let mut group = Cluster::new();

    group.register(DummyI32).with_name("i32").spawn().unwrap();
    group.register(DummyString).with_name("String").spawn().unwrap();

    let rep: i32 = group.send_to_with_reply("i32", ()).unwrap();
    assert_eq!(rep, 1);

    let rep: String = group.send_to_with_reply("String", ()).unwrap();
    assert_eq!(rep, "String");

    let rep: Result<i32, ClusterError> = group.send_to_with_reply("String", ());
    assert!(matches!(rep, Err(ClusterError::ContentConversionError)));

    assert!(group.send_to("foo", ()).is_err());
}

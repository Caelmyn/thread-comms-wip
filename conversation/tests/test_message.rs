use conversation::{IntoContent, IntoMessage};

#[test]
fn create_content() {
    let a = 1.into_content();
    let b = 2.into_content();

    assert_eq!(a.into::<i32>(), Some(1));
    assert_eq!(b.into::<String>(), None);
}

#[test]
fn create_message() {
    let a = 1.into_msg();
    let b = 2.into_msg_with_reply();
    let c = "three".into_msg();

    assert_eq!(a.into::<i32>(), Some(1));
    assert_eq!(b.into::<String>(), None);
    assert_eq!(c.into::<&str>(), Some("three"));
}

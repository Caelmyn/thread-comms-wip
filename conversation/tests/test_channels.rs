use std::thread;

use conversation::IntoContent;

#[test]
fn send_message() {
    let (tx, rx) = conversation::channel();

    let handle = thread::spawn(move || {
        let data = rx.recv_msg().unwrap();
        assert_eq!(data.into(), Some(1));

        let data = rx.recv_msg().unwrap();
        assert_eq!(data.into(), Some("Hello"));
    });

    tx.send_msg(1).unwrap();
    tx.send_msg("Hello").unwrap();

    handle.join().unwrap();
}

#[test]
fn send_message_with_reply() {
    let (tx, rx) = conversation::channel();

    let handle = thread::spawn(move || {
        let data = rx.recv_msg().unwrap();
        assert_eq!(data.into(), Some(1));
        rx.send_reply(2.into_content()).unwrap();

        let data = rx.recv_msg().unwrap();
        assert_eq!(data.into(), Some("Hello"));
        rx.send_reply("It's a reply".into_content()).unwrap();
    });

    tx.send_msg(1).unwrap();
    let resp = tx.recv_reply().unwrap();
    assert_eq!(resp.into(), Some(2));

    tx.send_msg("Hello").unwrap();
    let resp = tx.recv_reply().unwrap();
    assert_eq!(resp.into(), Some("It's a reply"));

    handle.join().unwrap();
}

#[test]
fn error_on_send() {
    let (tx, rx) = conversation::channel();

    let handle = thread::spawn(move || {
        rx.recv_msg().unwrap();
    });

    tx.send_msg(()).unwrap();

    handle.join().unwrap();

    let ret = tx.send_msg(());
    assert!(ret.is_err())
}

#[test]
fn error_on_recv() {
    let (tx, rx) = conversation::channel();

    let handle = thread::spawn(move || {
        tx.send_msg(()).unwrap();
    });

    rx.recv_msg().unwrap();

    handle.join().unwrap();

    let ret = rx.recv_msg();
    assert!(ret.is_err())
}

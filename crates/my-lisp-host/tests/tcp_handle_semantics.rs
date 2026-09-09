use my_lisp::{eval_program, load_tcp_library, Session};
use my_lisp_host::install;
use std::net::TcpListener;
use std::thread;

fn tcp_session() -> Session {
    install();
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    load_tcp_library(&mut session).unwrap();
    session
}

#[test]
fn listener_handles_expose_only_class_and_identity() {
    // Port 0 asks the OS for a fresh ephemeral listener each time, avoiding
    // the race inherent in probing a free port and then reopening it later.
    let source = r#"
        (def a (tcp-listen-on "127.0.0.1" 0))
        (def b (tcp-listen-on "127.0.0.1" 0))
        (list (eq a a) (eq a b) a b)
    "#;

    let value = eval_program(source, &mut tcp_session())
        .expect("two listener handles should be ordinary opaque runtime values")
        .value;

    assert_eq!(
        value.to_string(),
        "(t () <tcp-listener> <tcp-listener>)",
        "listener observables are pointer identity plus resource class, not std::net details"
    );
}

#[test]
fn connection_handle_keeps_identity_and_display_across_close() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("client should connect");
        drop(stream);
    });

    let source = format!(
        r#"
        (def c (tcp-connect "127.0.0.1" {port}))
        (def same (eq c c))
        (tcp-close c)
        (list same c)
        "#
    );
    let value = eval_program(&source, &mut tcp_session())
        .expect("connection identity remains meaningful after lifecycle operation")
        .value;

    assert_eq!(value.to_string(), "(t <tcp-connection>)");
    server.join().expect("server thread should finish");
}

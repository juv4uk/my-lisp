use my_lisp::{eval_program, load_tcp_library, Session};
use my_lisp_host::install;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn tcp_session() -> Session {
    install();
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    load_tcp_library(&mut session).unwrap();
    session
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("binding port 0 should succeed")
        .local_addr()
        .expect("bound listener has an address")
        .port()
}

#[test]
fn listener_handles_expose_only_class_and_identity() {
    let first = free_port();
    let second = free_port();
    let source = format!(
        r#"
        (def a (tcp-listen-on "127.0.0.1" {first}))
        (def b (tcp-listen-on "127.0.0.1" {second}))
        (list (eq a a) (eq a b) a b)
        "#
    );

    let value = eval_program(&source, &mut tcp_session())
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

#[test]
fn host_resource_representation_is_not_required_by_external_tcp_peer() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || listener.accept().map(|_| ()).unwrap());

    let mut session = tcp_session();
    eval_program(
        &format!(
            r#"(def c (tcp-connect "127.0.0.1" {port})) (tcp-close c)"#
        ),
        &mut session,
    )
    .expect("a native peer only observes TCP behavior, not the core handle payload type");

    server.join().unwrap();
    let _ = TcpStream::connect; // keep the test's host-side std::net dependency explicit
}

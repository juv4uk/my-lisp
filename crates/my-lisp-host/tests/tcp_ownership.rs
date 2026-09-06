use my_lisp::{load_core_library, load_tcp_library, installed_capabilities, Session, Value};
use my_lisp_host::install;

#[test]
fn tcp_read_public_binding_is_language_owned_over_raw_bytes() {
    install();
    let installed = installed_capabilities();
    assert!(installed.iter().any(|name| name == "tcp-read-raw"));
    assert!(!installed.iter().any(|name| name == "tcp-read"));

    let mut session = Session::default();
    load_core_library(&mut session).unwrap();
    assert!(session.environment.get("tcp-read").is_none());

    load_tcp_library(&mut session).unwrap();
    assert!(matches!(
        session.environment.get("tcp-read"),
        Some(Value::Closure(_))
    ));
}

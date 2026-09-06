use my_lisp::{load_core_library, load_time_library, Session, Value};

#[test]
fn timezone_detect_public_binding_becomes_language_owned() {
    let mut session = Session::default();
    load_core_library(&mut session).unwrap();

    assert!(matches!(
        session.environment.get("timezone-detect"),
        Some(Value::Builtin(_))
    ));
    assert!(session.environment.get("timezone-detect-raw").is_none());

    load_time_library(&mut session).unwrap();

    assert!(matches!(
        session.environment.get("timezone-detect"),
        Some(Value::Closure(_))
    ));
    assert!(matches!(
        session.environment.get("timezone-detect-raw"),
        Some(Value::Builtin(_))
    ));
}

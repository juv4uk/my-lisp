use my_lisp::{load_core_library, load_time_library, Session, Value};

#[test]
fn timezone_detect_public_binding_is_language_owned_over_raw_declarations() {
    let mut session = Session::default();
    load_core_library(&mut session).unwrap();

    assert!(session.environment.get("timezone-detect").is_none());
    assert!(session.environment.get("timezone-detect-raw").is_none());
    assert!(matches!(
        session.environment.get("timezone-declarations-raw"),
        Some(Value::Builtin(_))
    ));

    load_time_library(&mut session).unwrap();

    assert!(matches!(
        session.environment.get("timezone-detect"),
        Some(Value::Closure(_))
    ));
    assert!(session.environment.get("timezone-detect-raw").is_none());
    assert!(matches!(
        session.environment.get("timezone-declarations-raw"),
        Some(Value::Builtin(_))
    ));
}

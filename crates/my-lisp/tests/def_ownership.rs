use my_lisp::{eval_program, ErrorKind, Session, Value};

#[test]
fn default_session_binds_historical_def_as_a_language_macro() {
    let session = Session::default();
    assert!(matches!(
        session.environment.get("def"),
        Some(Value::Macro(_))
    ));
}

#[test]
fn aliased_def_uses_the_language_owned_macro_path() {
    let mut session = Session::default();
    let result = eval_program(
        "(define compatibility-def def) (compatibility-def answer (+ 20 22)) answer",
        &mut session,
    )
    .expect("aliased def must work without literal head-name dispatch");

    assert_eq!(result.value.to_string(), "42");
}

#[test]
fn aliased_def_preserves_definition_error_classes() {
    let mut arity_session = Session::default();
    eval_program("(define compatibility-def def)", &mut arity_session)
        .expect("compatibility macro must be bindable as a value");
    let arity = eval_program("(compatibility-def answer)", &mut arity_session)
        .expect_err("wrong def arity must remain named");
    assert_eq!(arity.kind, ErrorKind::Arity);

    let mut alias_session = Session::default();
    eval_program("(define compatibility-def def)", &mut alias_session)
        .expect("compatibility macro must be bindable as a value");
    let alias_invalid = eval_program("(compatibility-def 42 value)", &mut alias_session)
        .expect_err("non-symbol definition name must remain invalid");

    let mut canonical_session = Session::default();
    let canonical_invalid = eval_program("(define 42 value)", &mut canonical_session)
        .expect_err("canonical define rejects the same malformed name");

    assert_eq!(alias_invalid.kind, canonical_invalid.kind);
    assert_eq!(alias_invalid.kind, ErrorKind::InvalidForm);
}

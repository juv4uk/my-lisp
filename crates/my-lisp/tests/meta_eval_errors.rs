use my_lisp::{eval_program, ErrorKind, Session};

fn escape_lisp_string(source: &str) -> String {
    source.replace('\\', "\\\\").replace('"', "\\\"")
}

fn eval_meta(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/meta-eval.my"), &mut session).unwrap();

    let wrapper = format!(
        r#"(my-eval (read "{}") (quote ()))"#,
        escape_lisp_string(source),
    );

    eval_program(&wrapper, &mut session)
        .unwrap()
        .value
        .to_string()
}

fn eval_native_error(source: &str, expected_kind: ErrorKind, normalized: &str) -> String {
    let mut session = Session::default();
    let error = eval_program(source, &mut session).unwrap_err();
    assert_eq!(error.kind, expected_kind, "wrong native error kind for {source}");

    // Error wording and source spans are presentation details. S2 contracts the
    // named category; tests normalize only the semantic observation shared with
    // the Lisp meta-evaluator.
    normalized.to_string()
}

#[test]
fn unresolved_callable_has_named_error_parity() {
    for symbol in ["missing", "future-function", "not-defined-yet"] {
        let source = format!("({symbol})");
        let expected = format!("(error unbound-symbol {symbol})");
        let via_meta = eval_meta(&source);
        let via_native = eval_native_error(&source, ErrorKind::UnknownSymbol, &expected);

        assert_eq!(
            via_meta, via_native,
            "unbound-symbol parity failed for {source}"
        );
    }
}

#[test]
fn non_callable_values_are_type_failures_not_unknown_symbols() {
    for (source, expected) in [
        ("(42)", "(error not-callable 42)"),
        ("(t)", "(error not-callable t)"),
        ("((quote missing))", "(error not-callable missing)"),
    ] {
        let via_meta = eval_meta(source);
        let via_native = eval_native_error(source, ErrorKind::Type, expected);

        assert_eq!(
            via_meta, via_native,
            "not-callable parity failed for {source}"
        );
    }
}

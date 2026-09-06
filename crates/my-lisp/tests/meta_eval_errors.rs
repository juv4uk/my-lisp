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

fn eval_native_unknown_symbol(source: &str, symbol: &str) -> String {
    let mut session = Session::default();
    let error = eval_program(source, &mut session).unwrap_err();
    assert_eq!(error.kind, ErrorKind::UnknownSymbol);

    // Error wording and source spans are presentation details. S2 contracts the
    // named category; the symbol is the semantic detail supplied by the same
    // source form to both evaluators.
    format!("(error unbound-symbol {symbol})")
}

#[test]
fn unresolved_callable_has_named_error_parity() {
    for symbol in ["missing", "future-function", "not-defined-yet"] {
        let source = format!("({symbol})");
        let via_meta = eval_meta(&source);
        let via_native = eval_native_unknown_symbol(&source, symbol);

        assert_eq!(
            via_meta, via_native,
            "unbound-symbol parity failed for {source}"
        );
    }
}

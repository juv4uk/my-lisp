//! Vector family acceptance — contract 2.1 style, value-level semantics.
//!
//! Covers the audit findings: structural `eq` on vectors (was falling
//! through to `_ => false` even for the same object), O(1)-style
//! `vector-ref` access promised by the Value::Vector docstring, and
//! named-failure bounds/type errors instead of panics.

use my_lisp::{eval_program, parse, ErrorKind, Expr, ExprKind, Session};

fn session_with_core() -> Session {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.lisp"), &mut session)
        .expect("core.my should preload cleanly");
    session
}

fn session_with_witness_runner() -> Session {
    let mut session = session_with_core();
    eval_program(
        include_str!("../../../tests/fixtures/witness-runner.lisp"),
        &mut session,
    )
    .expect("witness-runner.lisp must load");
    session
}

fn alist_str<'a>(entries: &'a [Expr], key: &str) -> Option<&'a str> {
    entries.iter().find_map(|entry| {
        let ExprKind::Pair(k, v) = &entry.kind else {
            return None;
        };
        let ExprKind::Symbol(name) = &k.kind else {
            return None;
        };
        if &**name != key {
            return None;
        }
        match &v.kind {
            ExprKind::String(s) => Some(s.as_ref()),
            _ => None,
        }
    })
}

/// Rust transports the actual outcome; the expected value lives in the
/// referenced Lisp-owned fixture, never authored here (#114/#220).
fn assert_witness_fixture_passes(fixture_source: &str) {
    let mut session = session_with_witness_runner();
    let rows = parse(fixture_source).expect("witness fixture must parse");
    for row in &rows {
        let ExprKind::List(entries) = &row.kind else {
            panic!("each witness fixture row must be an alist: {row:?}");
        };
        let expr = alist_str(entries, "expr").expect("witness row needs \"expr\"");
        let actual = match eval_program(expr, &mut session) {
            Ok(result) => format!(
                "(value \"{}\")",
                result.value.to_string().replace('\\', "\\\\").replace('"', "\\\"")
            ),
            Err(error) => format!("(error \"{:?}\")", error.kind),
        };
        let program = format!(
            r#"(witness-status (witness-verdict (quote {}) (quote {})))"#,
            &fixture_source[row.span.start..row.span.end],
            actual
        );
        let status = eval_program(&program, &mut session)
            .expect("witness-verdict must evaluate")
            .value
            .to_string();
        assert_eq!(status, "pass", "witness failed for {expr}: actual {actual}");
    }
}

fn eval_source(source: &str) -> String {
    let result = eval_program(source, &mut session_with_core()).expect("eval should succeed");
    result.value.to_string()
}

fn eval_err(source: &str) -> ErrorKind {
    let mut session = session_with_core();
    eval_program(source, &mut session)
        .expect_err("should fail named")
        .kind
}

#[test]
fn vector_literal_and_ref() {
    assert_eq!(eval_source("(vector-ref (vector 10 20 30) 0)"), "10");
    assert_eq!(eval_source("(vector-ref (vector 10 20 30) 2)"), "30");
}

#[test]
fn make_vector_fills_with_nil_and_length_reports_size() {
    assert_eq!(eval_source("(vector-length (make-vector 5))"), "5");
    assert_eq!(eval_source("(vector-ref (make-vector 3) 1)"), "()");
}

#[test]
fn vectors_are_structurally_equal_across_objects_including_nested_elements() {
    assert_witness_fixture_passes(include_str!(
        "../../../tests/fixtures/vectors-eq-witness.lisp"
    ));
}

#[test]
fn vector_set_bang_mutates_through_the_binding() {
    assert_eq!(
        eval_source("(def v (vector 10 20 30))\n(vector-set! v 0 99)\n(vector-ref v 0)"),
        "99"
    );
    // other slots untouched
    assert_eq!(
        eval_source("(def v (vector 10 20 30))\n(vector-set! v 0 99)\n(vector-ref v 1)"),
        "20"
    );
}

#[test]
fn out_of_bounds_fails_named() {
    assert_eq!(
        eval_err("(vector-ref (vector 1) 5)"),
        ErrorKind::InvalidForm
    );
    assert_eq!(
        eval_err("(vector-set! (make-vector 2) 7 99)"),
        ErrorKind::InvalidForm
    );
}

#[test]
fn type_errors_fail_named() {
    assert_eq!(eval_err("(vector-ref (quote (1 2)) 0)"), ErrorKind::Type);
    assert_eq!(eval_err("(vector-length 42)"), ErrorKind::Type);
    assert_eq!(eval_err("(make-vector -1)"), ErrorKind::Type);
    assert_eq!(eval_err("(make-vector 2.5)"), ErrorKind::Type);
    assert_eq!(eval_err("(vector-ref (vector 1) 1.5)"), ErrorKind::Type);
}

#[test]
fn write_to_string_round_trip_uses_hash_paren_syntax() {
    assert_eq!(
        eval_source("(write-to-string (vector 1 (list 2) \"a\"))"),
        "\"#(1 (2) \\\"a\\\")\""
    );
}

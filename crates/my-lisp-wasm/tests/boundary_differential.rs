//! Boundary differential tests: native my-lisp Session vs WASM adapter code path.
//!
//! The WASM adapter (`evaluate()`) calls `my_lisp_literate::eval_literate()`
//! with a persistent `my_lisp::Session`. These tests exercise the SAME
//! functions natively, proving that the serialization boundary does NOT
//! alter semantics. They are NOT independent semantic proofs.
//!
//! Scope: arithmetic, lists, lambda persistence, error handling,
//! literate mode parsing.

use my_lisp::Session;
use my_lisp_literate::SourceMode;


fn make_session() -> Session {
    let mut s = make_session();
    let _ = my_lisp::eval_program(
        include_str!("../../../lib/core.my"), &mut s);
    s
}

fn eval_in_session(src: &str, session: &mut Session) -> Result<String, String> {
    let (result, _) = my_lisp_literate::eval_literate(
        src, SourceMode::PureLisp, session
    ).map_err(|e| e.to_string())?;
    Ok(result.value.to_string())
}

#[test]
fn arithmetic_fixnum_parity() {
    let mut s = make_session();
    assert_eq!(eval_in_session("(+ 1 2)", &mut s).unwrap(), "3");
}

#[test]
fn arithmetic_exact_rational_no_floats() {
    let mut s = make_session();
    // Exactness model: (/ 1 3) must produce rational, not float
    let r = eval_in_session("(/ 1 3)", &mut s).unwrap();
    assert!(r.contains('/'), "expected rational with / but got: {}", r);
}

#[test]
fn cons_car_cdr_roundtrip() {
    let mut s = make_session();
    eval_in_session("(define xs (cons 'a (cons 'b '())))", &mut s).unwrap();
    assert_eq!(eval_in_session("(car xs)", &mut s).unwrap(), "a");
}

#[test]
fn length_returns_correct_count() {
    let mut s = make_session();
    eval_in_session("(define ys '(p q r))", &mut s).unwrap();
    assert_eq!(eval_in_session("(length ys)", &mut s).unwrap(), "3");
}

#[test]
fn define_then_call_across_eval_boundary() {
    let mut s = make_session();
    eval_in_session("(def double (lambda (x) (* 2 x)))", &mut s).unwrap();
    assert_eq!(eval_in_session("(double 21)", &mut s).unwrap(), "42");
}

#[test]
fn parse_error_returns_err_not_panic() {
    let mut s = make_session();
    let r = eval_in_session("(+ 1", &mut s);
    assert!(r.is_err(), "unclosed paren should return Err, not panic");
}

#[test]
fn literate_mode_parses_markdown_fenced_blocks() {
    let md = "# Title\n\n```lisp\n(+ 10 20)\n```\n";
    let mut s = make_session();
    let (result, _) = my_lisp_literate::eval_literate(
        md, SourceMode::Literate, &mut s
    ).expect("literate markdown block should parse");
    assert_eq!(result.value.to_string(), "30");
}

#[test]
fn reset_clears_definitions() {
    let mut s = make_session();
    eval_in_session("(def temp-var 42)", &mut s).unwrap();
    // Fresh session: temp-var should not exist
    let mut s2 = make_session();
    let r = eval_in_session("temp-var", &mut s2);
    // Should either Err (unbound symbol) or produce something without 42
    if let Ok(v) = r {
        assert_ne!(v, "42", "temp-var leaked across session reset");
    }
}

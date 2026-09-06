//! Contract-2.1 witness for the metacircular evaluator path.
//! The expressions below are copied from the already-existing native
//! first-class-builtin acceptance matrix; this test does not invent a new
//! semantic target for the Lisp evaluator.

use my_lisp::{eval_program, Session};

const HIGHER_ORDER_BUILTIN: &str = "((lambda (f) (f 2 3)) +)";
const SHADOW_BUILTIN: &str = "((lambda (+) (+ 2 3)) (lambda (a b) (* a b)))";

fn eval_native(expr: &str) -> String {
    let mut session = Session::default();
    eval_program(expr, &mut session)
        .unwrap_or_else(|e| panic!("native evaluator failed: {e}\nexpr: {expr}"))
        .value
        .to_string()
}

fn eval_via_first_class_meta(expr: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session)
        .expect("core.my should load");
    eval_program(include_str!("../../../lib/meta-eval-first-class.my"), &mut session)
        .expect("first-class metacircular layer should load");

    let source = format!(
        r#"(my-eval-first-class (read "{}"))"#,
        expr.replace('\\', "\\\\").replace('"', "\\\"")
    );
    eval_program(&source, &mut session)
        .unwrap_or_else(|e| panic!("first-class meta-eval failed: {e}\nexpr: {expr}\nsource: {source}"))
        .value
        .to_string()
}

#[test]
fn witness_expressions_are_existing_contract_21_acceptance_cases() {
    let matrix = include_str!("first_class_builtins.rs");
    assert!(matrix.contains(HIGHER_ORDER_BUILTIN));
    assert!(matrix.contains(SHADOW_BUILTIN));
}

#[test]
fn metacircular_evaluator_accepts_builtin_as_higher_order_value() {
    assert_eq!(eval_native(HIGHER_ORDER_BUILTIN), "5");
    assert_eq!(eval_via_first_class_meta(HIGHER_ORDER_BUILTIN), "5");
}

#[test]
fn metacircular_evaluator_obeys_lexical_builtin_shadowing() {
    assert_eq!(eval_native(SHADOW_BUILTIN), "6");
    assert_eq!(eval_via_first_class_meta(SHADOW_BUILTIN), "6");
}

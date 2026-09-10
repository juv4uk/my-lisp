//! First code step of the evaluator ownership migration.
//! Keep the native evaluator as oracle and verify that the metacircular
//! evaluator preserves closure semantics.

use my_lisp::{eval_program, Session};

fn native(expr: &str) -> String {
    let mut session = Session::default();
    eval_program(expr, &mut session)
        .expect("native evaluator should succeed")
        .value
        .to_string()
}

fn meta(expr: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session)
        .expect("core should load");
    eval_program(include_str!("../../../lib/meta-eval.my"), &mut session)
        .expect("meta evaluator should load");

    let source = format!("(my-eval (read \"{}\") (quote ()))", expr);
    eval_program(&source, &mut session)
        .expect("my-eval should succeed")
        .value
        .to_string()
}

#[test]
fn closure_capture_has_native_parity() {
    let expr = "((lambda (x) ((lambda (y) (+ x y)) 2)) 40)";
    assert_eq!(meta(expr), native(expr));
}

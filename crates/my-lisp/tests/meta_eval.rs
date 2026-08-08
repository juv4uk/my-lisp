//! Exercises lib/meta-eval.my — the metacircular evaluator written in
//! my-lisp itself (see PLAN.md, Крок 9, item 1). Loads lib/core.my (for
//! `second`/`third`) and lib/meta-eval.my into one session, then runs
//! `(my-eval (read "...") env)` the same way a user would from a REPL.
//! Перевіряє lib/meta-eval.my — метациркулярний evaluator, написаний
//! самою my-lisp (див. PLAN.md, Крок 9, пункт 1). Завантажує lib/core.my
//! (заради `second`/`third`) і lib/meta-eval.my в одну сесію, тоді
//! запускає `(my-eval (read "...") env)` так само, як користувач з REPL.
//! Prüft lib/meta-eval.my — den metazirkulären Evaluator, geschrieben in
//! my-lisp selbst (siehe PLAN.md, Schritt 9, Punkt 1). Lädt lib/core.my
//! (wegen `second`/`third`) und lib/meta-eval.my in eine Sitzung, führt
//! dann `(my-eval (read "...") env)` genauso aus wie ein Nutzer aus der REPL.

use my_lisp::{eval_program, Session};

fn eval_meta(expr_source: &str, env_source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/meta-eval.my"), &mut session).unwrap();
    let source = format!(
        r#"(my-eval (read "{}") {})"#,
        expr_source.replace('\\', "\\\\").replace('"', "\\\""),
        env_source
    );
    eval_program(&source, &mut session).unwrap().value.to_string()
}

#[test]
fn self_evaluates_numbers_and_symbols_not_bound_in_env() {
    assert_eq!(eval_meta("42", "'()"), "42");
    assert_eq!(eval_meta("t", "'()"), "t");
}

#[test]
fn quote_returns_data_unevaluated() {
    assert_eq!(eval_meta("(quote radio)", "'()"), "radio");
}

#[test]
fn arithmetic_dispatches_to_the_real_primitives() {
    assert_eq!(eval_meta("(+ 1 2)", "'()"), "3");
    assert_eq!(eval_meta("(* 3 4)", "'()"), "12");
}

#[test]
fn cond_picks_the_first_truthy_clause() {
    assert_eq!(eval_meta("(cond (() 1) (t 2))", "'()"), "2");
}

#[test]
fn list_primitives_dispatch_to_the_real_primitives() {
    assert_eq!(eval_meta("(cons 1 (cons 2 (quote ())))", "'()"), "(1 2)");
    assert_eq!(eval_meta("(car (cons 1 2))", "'()"), "1");
    assert_eq!(eval_meta("(atom (quote ()))", "'()"), "t");
    assert_eq!(eval_meta("(eq 1 1)", "'()"), "t");
}

#[test]
fn lambda_application_binds_parameters_and_evaluates_the_body() {
    assert_eq!(eval_meta("((lambda (x) (+ x 1)) 5)", "'()"), "6");
}

#[test]
fn multi_expression_lambda_bodies_evaluate_in_sequence_returning_the_last() {
    // (+ x y) runs and is discarded; (* x y) is the returned value — proves
    // my-eval-body actually sequences instead of only ever reading the head.
    assert_eq!(
        eval_meta("((lambda (x y) (+ x y) (* x y)) 3 4)", "'()"),
        "12"
    );
}

#[test]
fn closures_capture_free_variables_from_the_calling_env() {
    // env = ((x . 5)); the lambda only takes y, so x must resolve through
    // the captured/passed-in env, not through its own parameter list.
    assert_eq!(
        eval_meta("((lambda (y) (+ x y)) 10)", "(cons (cons 'x 5) '())"),
        "15"
    );
}

#[test]
fn higher_order_functions_pass_a_closure_as_an_argument() {
    assert_eq!(
        eval_meta("((lambda (f x) (f x)) (lambda (n) (* n n)) 6)", "'()"),
        "36"
    );
}

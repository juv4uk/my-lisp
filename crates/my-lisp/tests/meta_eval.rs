//! Exercises lib/meta-eval.my — the metacircular evaluator written in
//! my-lisp itself (see PLAN.md, Krok 9, item 1). Loads lib/core.my (for
//! `second`/`third`) and lib/meta-eval.my into one session, then runs
//! `(my-eval (read "...") env)` the same way a user would from a REPL.
//! Pereviriaie lib/meta-eval.my — metatsyrkuliarnyi evaluator, napysanyi
//! samoiu my-lisp (dyv. PLAN.md, Krok 9, punkt 1). Zavantazhuie lib/core.my
//! (zarady `second`/`third`) i lib/meta-eval.my v odnu sesiiu, todi
//! zapuskaie `(my-eval (read "...") env)` tak samo, yak korystuvach z REPL.
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
    eval_program(&source, &mut session)
        .unwrap()
        .value
        .to_string()
}

/// Runs a sequence of top-level forms (as source text) through
/// `my-eval-program`, the `def`/`defmacro`-aware layer above `my-eval`
/// (PLAN.md item 8), then evaluates a final probe expression against the
/// resulting environment — the same "load a program, then use what it
/// defined" shape a real my-lisp session goes through.
fn eval_meta_program(program_source: &str, probe_source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/meta-eval.my"), &mut session).unwrap();
    let source = format!(
        r#"(let ((loaded (my-eval-program (read-all "{}") (quote ()))))
             (my-eval (read "{}") (car loaded)))"#,
        program_source.replace('\\', "\\\\").replace('"', "\\\""),
        probe_source.replace('\\', "\\\\").replace('"', "\\\""),
    );
    eval_program(&source, &mut session)
        .unwrap()
        .value
        .to_string()
}

#[test]
fn self_evaluates_numbers_and_symbols_not_bound_in_env() {
    assert_eq!(eval_meta("42", "(quote ())"), "42");
    assert_eq!(eval_meta("t", "(quote ())"), "t");
}

#[test]
fn quote_returns_data_unevaluated() {
    assert_eq!(eval_meta("(quote radio)", "(quote ())"), "radio");
}

#[test]
fn arithmetic_dispatches_to_the_real_primitives() {
    assert_eq!(eval_meta("(+ 1 2)", "(quote ())"), "3");
    assert_eq!(eval_meta("(* 3 4)", "(quote ())"), "12");
}

#[test]
fn cond_picks_the_first_truthy_clause() {
    assert_eq!(eval_meta("(cond (() 1) (t 2))", "(quote ())"), "2");
}

#[test]
fn list_primitives_dispatch_to_the_real_primitives() {
    assert_eq!(
        eval_meta("(cons 1 (cons 2 (quote ())))", "(quote ())"),
        "(1 2)"
    );
    assert_eq!(eval_meta("(car (cons 1 2))", "(quote ())"), "1");
    assert_eq!(eval_meta("(atom (quote ()))", "(quote ())"), "t");
    assert_eq!(eval_meta("(eq 1 1)", "(quote ())"), "t");
}

#[test]
fn lambda_application_binds_parameters_and_evaluates_the_body() {
    assert_eq!(eval_meta("((lambda (x) (+ x 1)) 5)", "(quote ())"), "6");
}

#[test]
fn multi_expression_lambda_bodies_evaluate_in_sequence_returning_the_last() {
    // (+ x y) runs and is discarded; (* x y) is the returned value — proves
    // my-eval-body actually sequences instead of only ever reading the head.
    assert_eq!(
        eval_meta("((lambda (x y) (+ x y) (* x y)) 3 4)", "(quote ())"),
        "12"
    );
}

#[test]
fn closures_capture_free_variables_from_the_calling_env() {
    // env = ((x . 5)); the lambda only takes y, so x must resolve through
    // the captured/passed-in env, not through its own parameter list.
    assert_eq!(
        eval_meta(
            "((lambda (y) (+ x y)) 10)",
            "(cons (cons (quote x) 5) (quote ()))"
        ),
        "15"
    );
}

#[test]
fn higher_order_functions_pass_a_closure_as_an_argument() {
    assert_eq!(
        eval_meta(
            "((lambda (f x) (f x)) (lambda (n) (* n n)) 6)",
            "(quote ())"
        ),
        "36"
    );
}

/// PLAN.md item 8: `my-eval` itself has no `def` — `my-eval-program`
/// threads an extended env across a sequence of top-level forms instead,
/// the same functional-fold shape used elsewhere in this project (e.g.
/// `lib/knowledge.my`'s journal projection) rather than real mutation.
#[test]
fn def_extends_the_environment_visible_to_later_top_level_forms() {
    assert_eq!(eval_meta_program("(def x 10) (def y (+ x 5))", "y"), "15");
}

#[test]
fn def_can_bind_a_lambda_callable_from_a_later_top_level_form() {
    assert_eq!(
        eval_meta_program("(def square (lambda (n) (* n n))) (square 6)", "(square 6)"),
        "36"
    );
}

#[test]
fn defmacro_expands_before_evaluating_using_unevaluated_argument_forms() {
    // `my-if`'s params are bound to the raw (unevaluated) call-site forms —
    // `then`/`else` here are `(quote yes)`/`(quote no)`, not their values —
    // and my-eval runs the expansion once more after the macro body builds it.
    assert_eq!(
        eval_meta_program(
            "(defmacro my-if (test then else) \
             (cons (quote cond) (cons (cons test (cons then (quote ()))) \
                                (cons (cons 't (cons else (quote ()))) (quote ())))))",
            "(my-if t (quote yes) (quote no))"
        ),
        "yes"
    );
}

/// The strongest claim PLAN.md item 8 makes: `my-eval-program` can load a
/// verbatim slice of the real `lib/core.my` bootstrap library — not a
/// hand-simplified stand-in — through `read-all`, then run the functions
/// it defines through `my-eval` itself, getting the same answer the host
/// Rust evaluator gives for the identical call.
#[test]
fn loads_a_real_verbatim_slice_of_lib_core_my_and_runs_it_through_my_eval() {
    let core_slice = r#"
(def identity (lambda (value) value))
(def not (lambda (value) (cond (value (quote ())) (t t))))
(def pair (lambda (left right) (cons left (cons right (quote ())))))
(def second (lambda (values) (car (cdr values))))
(def third (lambda (values) (car (cdr (cdr values)))))
"#;
    assert_eq!(
        eval_meta_program(core_slice, "(second (quote (a b c)))"),
        "b"
    );
    assert_eq!(
        eval_meta_program(core_slice, "(third (quote (a b c)))"),
        "c"
    );
    assert_eq!(eval_meta_program(core_slice, "(not (quote ()))"), "t");
    assert_eq!(
        eval_meta_program(core_slice, "(identity (quote radio))"),
        "radio"
    );
}

/// Documented, not hidden (PLAN.md item 8, mccarthy-principles.md §7): a
/// top-level `def` whose value refers to its own name doesn't see itself,
/// because environments here are an immutable alist, not the host's real
/// mutable frame (`environment.rs:70`) — the closure captures `env` from
/// *before* its own binding exists.
///
/// The metacircular evaluator does not yet own native `LanguageError` parity.
/// Therefore the semantic witness is the explicit Lisp failure value below,
/// not the accidental Rust `Type` error that the older implementation leaked
/// when it tried to take `car` of an unbound atom. This test pins the actual
/// limitation without turning substrate behavior into language semantics.
#[test]
fn self_recursive_top_level_def_reports_unbound_self_as_not_callable() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/meta-eval.my"), &mut session).unwrap();
    let source = r#"
        (let ((loaded (my-eval-program
                        (read-all "(def count-down (lambda (n) (cond ((eq n 0) (quote done)) (t (count-down (- n 1))))))")
                        (quote ()))))
          (my-eval (read "(count-down 3)") (car loaded)))
    "#;
    let result = eval_program(source, &mut session)
        .expect("the meta-evaluator should represent this known semantic limitation explicitly");
    assert_eq!(result.value.to_string(), "(not-callable count-down)");
}

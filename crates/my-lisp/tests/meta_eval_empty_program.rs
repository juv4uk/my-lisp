//! Regression witness for TASK-001: an empty metacircular program is a no-op.

use my_lisp::{eval_program, Session};

#[test]
fn empty_program_preserves_environment_and_returns_empty_result() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.lisp"), &mut session).unwrap();
    my_lisp::load_meta_evaluator_library(&mut session).unwrap();

    let witness = r#"
(let ((env (list (cons (quote sentinel) 42))))
  (let ((loaded (my-eval-program (quote ()) env)))
    (list (equal? (car loaded) env)
          (cdr loaded))))
"#;

    let result = eval_program(witness, &mut session).unwrap();

    // Expected shape queried live from Lisp, not hardcoded (#114/#220).
    let equal_same_shape = eval_program("(equal? 1 1)", &mut session)
        .unwrap()
        .value
        .to_string();
    assert_eq!(
        result.value.to_string(),
        format!("({equal_same_shape} ())")
    );
}

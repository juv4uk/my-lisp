//! Regression witness for TASK-001: an empty metacircular program is a no-op.

use my_lisp::{eval_program, Session};

#[test]
fn empty_program_preserves_environment_and_returns_empty_result() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/meta-eval.my"), &mut session).unwrap();

    let witness = r#"
(let ((env (list (cons (quote sentinel) 42))))
  (let ((loaded (my-eval-program (quote ()) env)))
    (list (equal? (car loaded) env)
          (cdr loaded))))
"#;

    let result = eval_program(witness, &mut session).unwrap();
    assert_eq!(result.value.to_string(), "(t ())");
}

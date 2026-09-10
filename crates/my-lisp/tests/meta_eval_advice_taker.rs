//! Vertical self-hosting witness: run the real Advice Taker reasoning stack
//! through the Lisp-owned meta-evaluator and require exact native/meta output
//! parity, including proof/provenance and epistemic outcome structure.

use my_lisp::{eval_program, Session};

fn escaped(source: &str) -> String {
    source.replace('\\', "\\\\").replace('"', "\\\"")
}

fn advice_program() -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}",
        include_str!("../../../lib/core.my"),
        include_str!("../../../lib/unify.my"),
        include_str!("../../../lib/reason.my"),
        include_str!("../../../lib/result-status.my"),
        r#"
(def meta-advice-rules
  (quote (
    ((parent alice bob))
    ((parent bob carol))
    ((middle (var x) (var y))
      (parent (var x) (var y)))
    ((grandparent (var x) (var z))
      (middle (var x) (var y))
      (parent (var y) (var z)))
    ((safe sky))
    ((not (safe sky)))
  )))

(def meta-advice-observe
  (lambda ()
    (let* ((proved
             (reason-observe
               (quote (grandparent alice carol))
               meta-advice-rules))
           (proof
             (second (car (third proved)))))
      (list
        (list (quote proved-outcome) proved)
        (list (quote proof-provenance) (provenance proof))
        (list
          (quote disputed-outcome)
          (reason-observe (quote (safe sky)) meta-advice-rules))
        (list
          (quote unknown-outcome)
          (reason-observe (quote (safe ocean)) meta-advice-rules))
        (list
          (quote invalid-outcome)
          (reason-observe (quote (not)) meta-advice-rules))))))

(meta-advice-observe)
"#,
    )
}

fn native_result(program: &str) -> String {
    let mut session = Session::default();
    eval_program(program, &mut session)
        .unwrap_or_else(|error| panic!("native Advice Taker witness failed: {error}"))
        .value
        .to_string()
}

fn meta_result(program: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).expect("core bootstrap");
    my_lisp::load_meta_evaluator_library(&mut session).expect("meta-eval bootstrap");

    eval_program(
        &format!(
            r#"(cdr (my-eval-program (read-all "{}") (quote ())))"#,
            escaped(program)
        ),
        &mut session,
    )
    .unwrap_or_else(|error| panic!("host failure while running meta Advice Taker witness: {error}"))
    .value
    .to_string()
}

#[test]
fn real_advice_taker_stack_has_exact_native_meta_parity() {
    let program = advice_program();
    let native = native_result(&program);
    let meta = meta_result(&program);

    assert!(native.contains("proved-outcome"));
    assert!(native.contains("proof-provenance"));
    assert!(native.contains("disputed-outcome"));
    assert!(native.contains("unknown-outcome"));
    assert!(native.contains("invalid-outcome"));
    assert_eq!(meta, native);
}

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
#[ignore = "deep meta-eval witness: release CI + debug nightly"]
fn real_advice_taker_stack_has_exact_native_meta_parity() {
    // Native and meta are each checked independently against the same
    // authored set of expected substrings — a known-correct shape of the
    // Advice Taker's epistemic outcome structure — never against each
    // other. Per this project's oracle-direction rule (see
    // tests/fixtures/README.md), a bug shared by both evaluators must not
    // hide behind a native-vs-meta equality check that stays green while
    // both are wrong.
    let expected_substrings = [
        "proved-outcome",
        "proof-provenance",
        "disputed-outcome",
        "unknown-outcome",
        "invalid-outcome",
    ];

    let program = advice_program();
    let native = native_result(&program);
    let meta = meta_result(&program);

    for substring in expected_substrings {
        assert!(
            native.contains(substring),
            "native Advice Taker output missing {substring:?}: {native}"
        );
        assert!(
            meta.contains(substring),
            "meta Advice Taker output missing {substring:?}: {meta}"
        );
    }
}

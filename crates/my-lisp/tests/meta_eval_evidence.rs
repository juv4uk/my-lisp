use my_lisp::{eval_program, ErrorKind, Session};

fn meta_session() -> Session {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).expect("core bootstrap");
    eval_program(include_str!("../../../lib/meta-eval.my"), &mut session)
        .expect("meta-eval bootstrap");
    session
}

fn escaped(source: &str) -> String {
    source.replace('\\', "\\\\").replace('"', "\\\"")
}

fn meta_eval(expr: &str) -> String {
    let mut session = meta_session();
    eval_program(
        &format!(r#"(my-eval (read \"{}\") (quote ()))"#, escaped(expr)),
        &mut session,
    )
    .unwrap_or_else(|error| panic!("host failure while meta-evaluating {expr}: {error}"))
    .value
    .to_string()
}

fn meta_eval_program(program: &str, probe: &str) -> String {
    let mut session = meta_session();
    let source = format!(
        r#"(let ((loaded (my-eval-program (read-all \"{}\") (quote ()))))
             (my-eval (read \"{}\") (car loaded)))"#,
        escaped(program),
        escaped(probe),
    );
    eval_program(&source, &mut session)
        .unwrap_or_else(|error| panic!("host failure while probing meta program: {error}"))
        .value
        .to_string()
}

fn meta_program_result(program: &str) -> String {
    let mut session = meta_session();
    eval_program(
        &format!(
            r#"(cdr (my-eval-program (read-all \"{}\") (quote ())))"#,
            escaped(program)
        ),
        &mut session,
    )
    .unwrap_or_else(|error| panic!("host failure while running meta program: {error}"))
    .value
    .to_string()
}

fn native_value(program: &str) -> String {
    let mut session = Session::default();
    eval_program(program, &mut session)
        .unwrap_or_else(|error| panic!("reference evaluator failed for {program}: {error}"))
        .value
        .to_string()
}

fn native_error_kind(program: &str) -> ErrorKind {
    let mut session = Session::default();
    eval_program(program, &mut session)
        .expect_err("reference program must fail")
        .kind
}

fn meta_error_kind(value: &str) -> Option<&str> {
    let mut tokens = value.trim_start_matches('(').split_whitespace();
    if tokens.next() == Some("error") {
        tokens.next()
    } else {
        None
    }
}

#[test]
fn named_error_pressure_records_exact_current_kind_matches_and_divergences() {
    struct Case {
        source: &'static str,
        native: ErrorKind,
        meta: &'static str,
        kind_parity: bool,
    }

    let cases = [
        Case {
            source: "(missing)",
            native: ErrorKind::UnknownSymbol,
            meta: "unbound-symbol",
            kind_parity: false,
        },
        Case {
            source: "((quote missing))",
            native: ErrorKind::Type,
            meta: "not-callable",
            kind_parity: false,
        },
        Case {
            source: "((lambda (x) x))",
            native: ErrorKind::Arity,
            meta: "arity",
            kind_parity: true,
        },
        Case {
            source: "(lambda (x x) x)",
            native: ErrorKind::InvalidForm,
            meta: "invalid-form",
            kind_parity: true,
        },
        Case {
            source: "(lambda (car) car)",
            native: ErrorKind::InvalidForm,
            meta: "invalid-form",
            kind_parity: true,
        },
    ];

    for case in cases {
        assert_eq!(native_error_kind(case.source), case.native, "source: {}", case.source);
        let meta = meta_eval(case.source);
        assert_eq!(meta_error_kind(&meta), Some(case.meta), "source: {}", case.source);

        let native_label = match case.native {
            ErrorKind::UnknownSymbol => "unknown-symbol",
            ErrorKind::Arity => "arity",
            ErrorKind::Type => "type",
            ErrorKind::InvalidForm => "invalid-form",
            _ => panic!("case uses an error kind outside the self-hosting evidence vocabulary"),
        };
        assert_eq!(
            native_label == case.meta,
            case.kind_parity,
            "matrix parity classification drifted for {}",
            case.source
        );
    }
}

#[test]
fn macro_arity_is_an_explicit_named_error_gap() {
    let program = "(defmacro one (x) x) (one)";
    assert_eq!(native_error_kind(program), ErrorKind::Arity);

    let meta = meta_program_result(program);
    assert_eq!(
        meta_error_kind(&meta),
        Some("unbound-symbol"),
        "the current meta evaluator re-evaluates its arity error as expansion data; if this changes, update the evidence matrix"
    );
}

#[test]
fn later_binding_visibility_is_recorded_as_a_real_reference_meta_divergence() {
    let program = r#"
(def f (lambda () (g)))
(def separator 0)
(def g (lambda () 42))
"#;

    assert_eq!(native_value(&format!("{program} (f)")), "42");
    let via_meta = meta_eval_program(program, "(f)");
    assert_eq!(meta_error_kind(&via_meta), Some("unbound-symbol"));
}

#[test]
fn recursive_group_captures_outer_lexical_environment() {
    let program = r#"
(def offset 7)
(def left
  (lambda (n)
    (cond
      ((eq n 0) offset)
      (t (right (- n 1))))))
(def right
  (lambda (n)
    (cond
      ((eq n 0) offset)
      (t (left (- n 1))))))
"#;
    for probe in ["(left 5)", "(right 6)"] {
        assert_eq!(meta_eval_program(program, probe), native_value(&format!("{program} {probe}")));
    }
}

#[test]
fn recursive_group_members_can_create_nested_closures_with_capture() {
    let program = r#"
(def offset 10)
(def make-step
  (lambda (n)
    (cond
      ((eq n 0) (lambda (x) (+ x offset)))
      (t (bounce (- n 1))))))
(def bounce (lambda (n) (make-step n)))
"#;
    let probe = "((make-step 3) 5)";
    assert_eq!(meta_eval_program(program, probe), "15");
    assert_eq!(meta_eval_program(program, probe), native_value(&format!("{program} {probe}")));
}

#[test]
fn ordinary_parameter_shadowing_beats_recursive_group_bindings() {
    let program = r#"
(def call-local (lambda (peer) (peer 5)))
(def peer (lambda (x) (+ x 1)))
"#;
    let probe = "(call-local (lambda (x) (* x 2)))";
    assert_eq!(meta_eval_program(program, probe), "10");
    assert_eq!(meta_eval_program(program, probe), native_value(&format!("{program} {probe}")));
}

#[test]
fn adjacent_non_recursive_lambda_defs_are_currently_false_grouped() {
    let program = r#"
(def inc (lambda (x) (+ x 1)))
(def double (lambda (x) (* x 2)))
"#;
    let result = meta_program_result(program);
    assert!(
        result.starts_with("(recursive-group-closure double "),
        "current recognizer groups every contiguous lambda-def block; this is a deliberate broken-row witness, got {result}"
    );
}

#[test]
fn malformed_recursive_group_preserves_invalid_form_kind() {
    let program = r#"
(def good (lambda (x) x))
(def bad (lambda (x x) x))
"#;
    assert_eq!(native_error_kind(program), ErrorKind::InvalidForm);
    let meta = meta_program_result(program);
    assert_eq!(meta_error_kind(&meta), Some("invalid-form"));
}

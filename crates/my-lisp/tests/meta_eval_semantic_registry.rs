use my_lisp::{eval_program, load_core_library, load_meta_evaluator_library, Session};

fn meta_session() -> Session {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core bootstrap");
    load_meta_evaluator_library(&mut session).expect("meta-eval bootstrap");
    session
}

fn native_value(source: &str) -> String {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core bootstrap");
    eval_program(source, &mut session)
        .unwrap_or_else(|error| panic!("native evaluator failed for {source}: {error}"))
        .value
        .to_string()
}

fn meta_value(source: &str) -> String {
    let mut session = meta_session();
    let escaped = source.replace('\\', "\\\\").replace('"', "\\\"");
    eval_program(
        &format!(r#"(my-eval (read "{escaped}") (quote ()))"#),
        &mut session,
    )
    .unwrap_or_else(|error| panic!("meta evaluator host failure for {source}: {error}"))
    .value
    .to_string()
}

fn meta_program_result(source: &str) -> String {
    let mut session = meta_session();
    let escaped = source.replace('\\', "\\\\").replace('"', "\\\"");
    eval_program(
        &format!(
            r#"(cdr (my-eval-program (read-all "{escaped}") (quote ())))"#
        ),
        &mut session,
    )
    .unwrap_or_else(|error| panic!("meta program host failure for {source}: {error}"))
    .value
    .to_string()
}

#[test]
fn generated_projection_exposes_only_admitted_runtime_surfaces() {
    let mut session = meta_session();
    let result = eval_program(
        r#"
(and
  (equal? (my-semantic-id-for-surface (quote quote)) "0001")
  (equal? (my-semantic-id-for-surface (quote як-є)) "0001")
  (equal? (my-semantic-id-for-surface (quote lambda)) "0010")
  (equal? (my-semantic-id-for-surface (quote функція)) "0010")
  (equal? (my-semantic-id-for-surface (quote define)) "0011")
  (equal? (my-semantic-id-for-surface (quote визначити)) "0011")
  (equal? (my-semantic-id-for-surface (quote def)) "1000")
  (atom (my-semantic-id-for-surface (quote rūpa))))
"#,
        &mut session,
    )
    .expect("projection witness should execute");
    assert_eq!(result.value.to_string(), "t");
}

#[test]
fn canonical_surface_parity_survives_registry_indirection() {
    // Every surface spelling below names the same semantic identity, so
    // every one of them must evaluate to the same authored expected value —
    // checked independently for native and meta, never against each other.
    let expected = "t";
    for source in [
        "(atom (quote x))",
        "(атом? (як-є x))",
        "(aṇu (svarūpa x))",
        "(.? (quote x))",
    ] {
        assert_eq!(native_value(source), expected, "source: {source}");
        assert_eq!(meta_value(source), expected, "source: {source}");
    }
}

#[test]
fn necessary_form_surfaces_route_by_semantic_identity() {
    let lambda_expected = "42";
    for source in [
        "((lambda (x) x) 42)",
        "((функція (x) x) 42)",
    ] {
        assert_eq!(native_value(source), lambda_expected, "source: {source}");
        assert_eq!(meta_value(source), lambda_expected, "source: {source}");
    }

    let define_expected = "42";
    for source in [
        "(define answer 42)",
        "(визначити answer 42)",
        "(def answer 42)",
    ] {
        assert_eq!(native_value(source), define_expected, "source: {source}");
        assert_eq!(meta_program_result(source), define_expected, "source: {source}");
    }
}

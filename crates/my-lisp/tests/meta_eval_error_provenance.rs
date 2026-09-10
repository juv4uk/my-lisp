use my_lisp::{eval_program, Session};

fn escaped(source: &str) -> String {
    source.replace('\\', "\\\\").replace('"', "\\\"")
}

fn meta_eval(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).expect("core bootstrap");
    my_lisp::load_meta_evaluator_library(&mut session)
        .expect("meta-eval bootstrap");
    eval_program(
        &format!(r#"(my-eval (read "{}") (quote ()))"#, escaped(source)),
        &mut session,
    )
    .unwrap_or_else(|error| panic!("host failure while meta-evaluating {source}: {error}"))
    .value
    .to_string()
}

fn native_value(source: &str) -> String {
    let mut session = Session::default();
    eval_program(source, &mut session)
        .unwrap_or_else(|error| panic!("reference evaluator failed for {source}: {error}"))
        .value
        .to_string()
}

#[test]
fn error_shaped_user_data_keeps_value_provenance() {
    let source = "((lambda (x) (car x)) (quote (error ordinary data)))";
    let native = native_value(source);
    assert_eq!(native, "error");
    assert_eq!(meta_eval(source), native);
}

#[test]
fn fail_shaped_user_data_is_not_internal_failure() {
    let source = "((lambda (x) (car x)) (quote (fail ordinary data)))";
    let native = native_value(source);
    assert_eq!(native, "fail");
    assert_eq!(meta_eval(source), native);
}

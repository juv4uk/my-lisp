use my_lisp::{eval_program, load_core_library, Session};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_lisp_file(path: &str, session: &mut Session) {
    let path = repo_root().join(path);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", path.display()));
    eval_program(&source, session)
        .unwrap_or_else(|error| panic!("{} must load as ordinary my-lisp: {error}", path.display()));
}

fn native_first_session() -> Session {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core must bootstrap before native-first classification");
    load_lisp_file("lib/machine/encoding/x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/layout/pair-x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/operands/x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/lowering/semantic-x86-64.lisp", &mut session);

    let dispatch = repo_root().join("lib/machine/dispatch/native-first.lisp");
    if dispatch.exists() {
        load_lisp_file("lib/machine/dispatch/native-first.lisp", &mut session);
    }

    session
}

fn classify(source: &str) -> String {
    eval_program(source, &mut native_first_session())
        .unwrap_or_else(|error| panic!("native-first classifier failed for {source}: {error}"))
        .value
        .to_string()
}

#[test]
fn bounded_literal_car_cons_gets_a_native_plan() {
    assert_eq!(
        classify("(native-first-plan (quote (car (cons 2 3))))"),
        "(native-plan ((mov-r64-imm64 rax 2) (mov-mem-disp8-r64 rdi 0 rax) (mov-r64-imm64 rax 3) (mov-mem-disp8-r64 rdi 8 rax) (mov-r64-mem-disp8 rax rdi 0) (ret)) 16)"
    );
}

#[test]
fn unsupported_expressions_fall_back_unchanged() {
    for (source, expected) in [
        (
            "(native-first-plan (quote (+ 2 3)))",
            "(evaluator-fallback (+ 2 3))",
        ),
        (
            "(native-first-plan (quote (car (cons (+ 1 1) 3))))",
            "(evaluator-fallback (car (cons (+ 1 1) 3)))",
        ),
        (
            "(native-first-plan (quote (car (cons -1 3))))",
            "(evaluator-fallback (car (cons -1 3)))",
        ),
        (
            "(native-first-plan (quote radio))",
            "(evaluator-fallback radio)",
        ),
        (
            "(native-first-plan (quote (car . radio)))",
            "(evaluator-fallback (car . radio))",
        ),
    ] {
        assert_eq!(classify(source), expected, "unexpected route for {source}");
    }
}

#[test]
fn classification_does_not_execute_native_code() {
    let result = classify("(native-first-plan (quote (car (cons 7 9))))");
    assert!(result.starts_with("(native-plan "));
    assert!(
        result.contains("(mov-r64-imm64 rax 7)"),
        "classifier should return structured forms, not execute them"
    );
}

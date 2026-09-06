//! B1 boundary ordering for knowledge-module observations: validate the input
//! before deciding that a module is merely absent.

use my_lisp::{eval_program, Session};

fn observe_in(source: &str) -> String {
    let mut session = Session::default();
    for library in [
        include_str!("../../../lib/core.my"),
        include_str!("../../../lib/unify.my"),
        include_str!("../../../lib/reason.my"),
        include_str!("../../../lib/forward.my"),
        include_str!("../../../lib/knowledge.my"),
        include_str!("../../../lib/result-status.my"),
    ] {
        eval_program(library, &mut session).unwrap();
    }
    eval_program(source, &mut session)
        .unwrap_or_else(|e| panic!("evaluation failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

#[test]
fn malformed_module_name_is_invalid_not_unknown() {
    assert_eq!(
        observe_in(r#"(reason-in-observe 42 (quote (planet earth)))"#),
        "(invalid invalid-module 42)"
    );
}

#[test]
fn malformed_goal_is_invalid_even_when_module_is_missing() {
    assert_eq!(
        observe_in(r#"(reason-in-observe (quote missing) (quote (not)))"#),
        "(invalid invalid-goal (not))"
    );
}

#[test]
fn valid_question_about_missing_module_is_still_unknown() {
    assert_eq!(
        observe_in(r#"(reason-in-observe (quote missing) (quote (planet earth)))"#),
        "(unknown (module-not-found missing))"
    );
}

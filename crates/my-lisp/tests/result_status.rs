//! MYLISP-UNKNOWN-RESULT-SEMANTICS-DESIGN: lib/result-status.my's opt-in
//! tagged-result constructors (unknown/partial/blocked/disputed), kept
//! separate from lib/reason.my/lib/knowledge.my's tests since this
//! module isn't wired into either — see docs/adr/unknown-result-semantics.md.

use my_lisp::{eval_program, Session};

fn eval_result_status(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/result-status.my"), &mut session).unwrap();
    eval_program(source, &mut session)
        .unwrap_or_else(|e| panic!("evaluation failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

#[test]
fn make_unknown_tags_a_reason() {
    assert_eq!(
        eval_result_status(r#"(make-unknown (quote no-rules-for-predicate))"#),
        "(unknown no-rules-for-predicate)"
    );
}

#[test]
fn make_partial_tags_a_value_and_a_bound() {
    assert_eq!(
        eval_result_status(r#"(make-partial (quote ()) (quote (depth . 12)))"#),
        "(partial () (depth . 12))"
    );
}

#[test]
fn make_blocked_tags_a_reason() {
    assert_eq!(
        eval_result_status(r#"(make-blocked (quote (depends-on PANINI-BRIDGE-MY-LISP-SYNTAX-CONVERSION)))"#),
        "(blocked (depends-on PANINI-BRIDGE-MY-LISP-SYNTAX-CONVERSION))"
    );
}

#[test]
fn make_disputed_tags_a_proof_list() {
    assert_eq!(
        eval_result_status(r#"(make-disputed (list (quote proof-a) (quote proof-b)))"#),
        "(disputed (proof-a proof-b))"
    );
}

#[test]
fn result_status_extracts_the_tag() {
    assert_eq!(
        eval_result_status(r#"(result-status (make-unknown (quote x)))"#),
        "unknown"
    );
    assert_eq!(
        eval_result_status(r#"(result-status (make-blocked (quote x)))"#),
        "blocked"
    );
}

#[test]
fn result_payload_extracts_everything_after_the_tag() {
    assert_eq!(
        eval_result_status(r#"(result-payload (make-partial 1 2))"#),
        "(1 2)"
    );
}

#[test]
fn an_ordinary_value_is_not_a_tagged_result() {
    // A ordinary my-lisp value (including a plain list) must not be
    // mistaken for a tagged result just because it has a car — the
    // whole point of the four-tag gate in result-tagged? is precision.
    assert_eq!(eval_result_status(r#"(result-tagged? 42)"#), "()");
    assert_eq!(eval_result_status(r#"(result-tagged? (quote (a b c)))"#), "()");
    assert_eq!(
        eval_result_status(r#"(result-tagged? (make-disputed (quote ())))"#),
        "t"
    );
}

#[test]
fn result_status_of_an_untagged_value_is_nil() {
    assert_eq!(eval_result_status(r#"(result-status 42)"#), "()");
}

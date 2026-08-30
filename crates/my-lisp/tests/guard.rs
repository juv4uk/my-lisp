use my_lisp::{eval_program, Session};

fn eval_guard(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/guard.wsm"), &mut session).unwrap();
    eval_program(source, &mut session)
        .unwrap_or_else(|e| panic!("guard evaluation failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

#[test]
fn guard_keeps_decision_and_evidence_status_separate() {
    let value = eval_guard(
        r#"(make-guard-finding
              (quote warn) (quote confirmed) (quote swarm/tasks)
              (quote auto-sync-disabled) (quote tasks-materialized)
              (quote tasks-not-materialized) (quote stale-projection)
              (quote sync-tasks) (quote (systemd tasks.my journal)))"#,
    );
    assert!(value.contains("(decision warn)"));
    assert!(value.contains("(evidence-status confirmed)"));
    assert!(value.contains("(schema guard/1)"));
}

#[test]
fn missing_evidence_is_unknown_not_reject() {
    let value = eval_guard(
        r#"(guard-unknown (quote swarm/peer) (quote peer-journal-unreadable) (quote inspect-peer))"#,
    );
    assert!(value.contains("(decision unknown)"));
    assert!(value.contains("(evidence-status unresolved)"));
}

#[test]
fn oracle_and_observation_agreement_allows() {
    let value = eval_guard(
        r#"(guard-compare (quote arithmetic) (quote 3/10) (quote 3/10) (quote (oracle runtime)))"#,
    );
    assert!(value.contains("(decision allow)"));
    assert!(value.contains("(evidence-status confirmed)"));
}

#[test]
fn disagreement_warns_and_preserves_both_values() {
    let value = eval_guard(
        r#"(guard-compare (quote task-state) (quote ready) (quote missing) (quote (tasks.my journal)))"#,
    );
    assert!(value.contains("(decision warn)"));
    assert!(value.contains("(expected ready observed missing)"));
}

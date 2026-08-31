use my_lisp::{eval_program, Session};

fn eval_guard(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/guard.wsm"), &mut session).unwrap();
    eval_program(
        include_str!("../../../knowledge/guard-reference.wsm"),
        &mut session,
    )
    .unwrap();
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
fn guard_requires_freeze_sync_drift_record_before_reopening_commits() {
    let blocked = eval_guard(
        r#"(guard-sync-window (quote open) (quote completed) (quote recorded)
             (quote (git sync)))"#,
    );
    assert!(blocked.contains("freeze-commits-before-synchronization"));
    let ready = eval_guard(
        r#"(guard-sync-window (quote frozen) (quote completed) (quote recorded)
             (quote (git sync drift-log)))"#,
    );
    assert!(ready.contains("reopen-commit-window"));
}

#[test]
fn missing_evidence_is_unknown_not_reject() {
    let value = eval_guard(
        r#"(guard-unknown (quote swarm/peer) (quote peer-journal-unreadable) (quote inspect-peer))"#,
    );
    assert!(value.contains("(decision unknown)"));
    assert!(value.contains("(evidence-status unresolved)"));
    assert!(value.contains("(route ask-agent)"));
    assert!(value.contains("(route ask-owner)"));
    assert!(value.contains("(route research-web)"));
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

#[test]
fn reference_bureau_points_to_authority_workflow_and_evidence() {
    assert_eq!(
        eval_guard(r#"(guard-authority (quote semantic-oracle))"#),
        "(language-contract.my crates/my-lisp-cli/src/swarm.rs docs/guard-oracle-node-plan.md)"
    );
    assert_eq!(
        eval_guard(r#"(guard-how-to (quote task-materialization))"#),
        "(edit-tasks.my auto-sync-or-sync-tasks verify-projection verify-peer-convergence)"
    );
    assert_eq!(
        eval_guard(r#"(guard-verify (quote agent-messaging))"#),
        "(durable-inbox-entry wakeup-result session-id-not-unknown commit-sha)"
    );
}

#[test]
fn unknown_topic_is_an_honest_unresolved_reference() {
    let value = eval_guard(r#"(guard-reference (quote no-such-topic))"#);
    assert!(value.contains("reference-missing"));
    assert!(value.contains("(decision unknown)"));
    assert!(value.contains("(evidence-status unresolved)"));
    assert!(value.contains("(guidance choose-unknown-route)"));
    assert!(value.contains("(route ask-agent)"));
    assert!(value.contains("(route ask-owner)"));
    assert!(value.contains("(route research-web)"));
}

#[test]
fn agents_can_enumerate_the_reference_desk() {
    let topics = eval_guard(r#"(guard-topics *guard-reference-directory*)"#);
    assert!(topics.contains("language-semantics"));
    assert!(topics.contains("swarm-coordination"));
    assert!(topics.contains("swarm-node-startup"));
    assert!(topics.contains("guix"));
    assert!(topics.contains("documentation-map"));
    assert!(topics.contains("reference-learning"));
    assert!(topics.contains("licenses"));
    assert!(topics.contains("owner-context"));
    assert!(topics.contains("wsm-lisp-filesystem"));
}

#[test]
fn spawn_node_reference_requires_exclusive_state_ownership() {
    assert_eq!(
        eval_guard(r#"(guard-verify (quote swarm-node-startup))"#),
        "(exclusive-data-dir-lock unchanged-identity-on-failed-spawn explicit-task-sync-choice metrics)"
    );
}

#[test]
fn reference_learning_preserves_review_and_provenance_boundaries() {
    assert_eq!(
        eval_guard(r#"(guard-how-to (quote reference-learning))"#),
        "(search-directory choose-unknown-route collect-source-and-evidence append-pending-review review promote-or-reject)"
    );
    assert_eq!(
        eval_guard(r#"(guard-verify (quote reference-learning))"#),
        "(candidate-record provenance review-status curated-entry-or-rejection)"
    );
    assert_eq!(
        eval_guard(r#"(guard-authority (quote guix))"#),
        "(../ecosystem/docs/VIVEKA-FINDINGS-2026-08-24.md manifest.scm channels.scm guix.scm evidence/GUIX-WITNESS-01)"
    );
}

#[test]
fn agents_can_list_curated_frequently_used_tools() {
    let tools = eval_guard(r#"(guard-scripts *guard-script-directory*)"#);
    assert!(tools.contains("oracle-check"));
    assert!(tools.contains("agent-send"));
    assert!(tools.contains("resource-preflight"));
    assert!(tools.contains("bilingual-docs-check"));
    assert!(tools.contains("conformance-check"));
    assert!(!tools.contains("registry-audit"));
}

#[test]
fn script_reference_contains_path_invocation_risk_and_evidence() {
    let tool = eval_guard(r#"(guard-script (quote agent-send))"#);
    assert!(tool.contains("../ecosystem/scripts/agent-send"));
    assert!(tool.contains("/home/agents/ecosystem/scripts/agent-send send"));
    assert!(tool.contains("(risk writes-coordination-log)"));
    assert!(tool.contains("(verify (admitted inbox-id wakeup-result))"));
}

#[test]
fn unknown_script_routes_instead_of_guessing() {
    let tool = eval_guard(r#"(guard-script (quote no-such-tool))"#);
    assert!(tool.contains("tool-missing"));
    assert!(tool.contains("(decision unknown)"));
    assert!(tool.contains("(route ask-agent)"));
    assert!(tool.contains("(route ask-owner)"));
    assert!(tool.contains("(route research-web)"));
}

use my_lisp::{eval_program, Session};

const DEPRECATION: &str = include_str!("../../../knowledge/swarm-legacy-deprecation.wsm");
const NO_LIVE_CALLERS_AUDIT: &str =
    include_str!("../../../knowledge/swarm-no-live-callers-audit.wsm");
const MESH_DOC: &str = include_str!("../../../docs/swarm-mesh-v2.md");
const AGENT_GUIDE: &str = include_str!("../../../AGENTS.md");

#[test]
fn legacy_coordination_deprecation_is_machine_readable() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(DEPRECATION, &mut session).unwrap();

    let value = session
        .environment
        .get("*swarm-legacy-coordination*")
        .expect("deprecation marker should define machine-readable migration data")
        .to_string();

    assert!(value.contains("(status . deprecated)"), "{value}");
    assert!(
        value.contains("(coordination-authority . swarm-node)"),
        "{value}"
    );
    assert!(value.contains("hello"), "{value}");
    assert!(value.contains("claim"), "{value}");
    assert!(value.contains("subscribe"), "{value}");
    assert!(value.contains("notify"), "{value}");
    assert!(value.contains("preserve-eval-parse-diagnose"), "{value}");
}

#[test]
fn no_live_callers_removal_gate_fails_closed_while_blockers_exist() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(NO_LIVE_CALLERS_AUDIT, &mut session).unwrap();

    let value = session
        .environment
        .get("*swarm-no-live-callers-audit*")
        .expect("no-live-callers audit should be executable machine-readable data")
        .to_string();

    assert!(
        value.contains("(schema . swarm-no-live-callers-audit/1)"),
        "{value}"
    );
    assert!(value.contains("(scope . ecosystem)"), "{value}");
    assert!(value.contains("(status . partial)"), "{value}");
    assert!(
        NO_LIVE_CALLERS_AUDIT.contains("(safe-to-remove . ())"),
        "partial audit must fail closed instead of claiming removal safety"
    );
    assert!(
        value.contains("(my-lisp-production-operational . confirmed)"),
        "{value}"
    );
    assert!(
        value.contains("(my-lisp-production-operational-callers)"),
        "{value}"
    );
    assert!(value.contains("cross-repo-active-guidance"), "{value}");
    assert!(value.contains("cml/tasks.my"), "{value}");
    assert!(value.contains("legacy-cli-compatibility-test"), "{value}");
    assert!(value.contains("semantic-callers-allowed"), "{value}");
    assert!(
        value.contains("sibling-executable-legacy-caller-search"),
        "{value}"
    );
}

#[test]
fn current_agent_authority_forbids_new_legacy_coordination_callers() {
    assert!(AGENT_GUIDE.contains("Current coordination authority:"));
    assert!(AGENT_GUIDE.contains("`swarm-node`"));
    assert!(AGENT_GUIDE.contains("must not be used for new coordination workflows"));
    assert!(AGENT_GUIDE.contains("my-lisp :9999"));
    assert!(AGENT_GUIDE.contains("swarm-node :910x"));
}

#[test]
fn human_migration_doc_keeps_semantic_and_coordination_planes_separate() {
    assert!(MESH_DOC.contains("my-lisp :9999"));
    assert!(MESH_DOC.contains("swarm-node :910x"));
    assert!(MESH_DOC.contains("no longer the\ncoordination path going forward"));
    assert!(MESH_DOC.contains("semantic oracle"));
}

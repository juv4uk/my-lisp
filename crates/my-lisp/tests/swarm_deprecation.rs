use my_lisp::{eval_program, Session};

const DEPRECATION: &str = include_str!("../../../knowledge/swarm-legacy-deprecation.wsm");
const MESH_DOC: &str = include_str!("../../../docs/swarm-mesh-v2.md");

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
fn human_migration_doc_keeps_semantic_and_coordination_planes_separate() {
    assert!(MESH_DOC.contains("my-lisp :9999"));
    assert!(MESH_DOC.contains("swarm-node :910x"));
    assert!(MESH_DOC.contains("no longer the\ncoordination path going forward"));
    assert!(MESH_DOC.contains("semantic oracle"));
}

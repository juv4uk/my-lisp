use my_lisp::{semantic_registry_export, Environment, Value};
use std::collections::BTreeSet;
use std::rc::Rc;

const STRING_APPEND_SEMANTIC_ID: &str = "1043";
const UK_SURFACE: &str = include_str!("../../../lib/surface/uk.lisp");

#[test]
fn string_append_1043_uk_peer_is_direct_root_binding_without_surface_bridge() {
    let names: BTreeSet<_> = semantic_registry_export::admitted_surfaces_for_semantic_id(
        STRING_APPEND_SEMANTIC_ID,
    )
    .into_iter()
    .map(|row| row.name)
    .collect();

    assert!(names.contains("string-append"));
    assert!(names.contains("зчепити"));

    let environment = Environment::root();
    let english = environment
        .get("string-append")
        .expect("1043 English peer must exist in the root environment");
    let ukrainian = environment
        .get("зчепити")
        .expect("1043 Ukrainian peer must exist directly in the root environment");

    match (&english, &ukrainian) {
        // Mechanism-only invariant: one registry-driven peer installation
        // shares the initial runtime handle. Semantic identity itself remains
        // the numeric registry ID 1043, not the Rc allocation.
        (Value::Builtin(english), Value::Builtin(ukrainian)) => assert!(
            Rc::ptr_eq(english, ukrainian),
            "1043 peers must be installed from one runtime builtin value"
        ),
        other => panic!("1043 peers must both be builtin values, got {other:?}"),
    }

    assert!(
        !UK_SURFACE.contains("(define зчепити string-append)"),
        "Ukrainian 1043 must not be implemented as an alias through English"
    );
}

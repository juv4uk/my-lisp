//! Integration test for #719: preserve four distinct observable result domains.
//!
//! Uses real Lisp and Datalog engines. CLIPS and Prolog slots are typed
//! placeholders because their native islands (#712, #714) are not yet merged.

use my_lisp::{eval_program, load_core_library, Session};
use wsm_datalog_kernel::{
    Atom, Database, Evaluator, Program, Rule, Term, Value,
};
use wsm_native_result_types::{datalog_result, FourKernelObservation, NativeResult};

fn run_lisp_fact() -> (String, Vec<String>) {
    let mut session = Session::default();
    load_core_library(&mut session).unwrap();
    let result = eval_program("(+ 20 22)", &mut session).unwrap();
    (result.value.to_string(), result.output)
}

fn run_datalog_transitive_closure() -> Database {
    let mut db = Database::new();
    db.add_fact("edge", vec![Value::sym("a"), Value::sym("b")]);
    db.add_fact("edge", vec![Value::sym("b"), Value::sym("c")]);

    let mut program = Program::new();
    program.add_rule(Rule::with_id(
        "path-base",
        Atom::new("path", vec![Term::v("X"), Term::v("Y")]),
        vec![Atom::new("edge", vec![Term::v("X"), Term::v("Y")])],
    ));
    program.add_rule(Rule::with_id(
        "path-step",
        Atom::new("path", vec![Term::v("X"), Term::v("Z")]),
        vec![
            Atom::new("edge", vec![Term::v("X"), Term::v("Y")]),
            Atom::new("path", vec![Term::v("Y"), Term::v("Z")]),
        ],
    ));

    Evaluator::naive_fixpoint(&program, &mut db);
    db
}

#[test]
fn four_domain_artifact_stores_lisp_and_datalog_independently() {
    let (lisp_value, lisp_output) = run_lisp_fact();
    let datalog_db = run_datalog_transitive_closure();

    let obs = FourKernelObservation::from_lisp_and_datalog(
        lisp_value.clone(),
        lisp_output.clone(),
        &datalog_db,
    );

    // Each domain keeps its own native shape.
    assert!(matches!(obs.lisp, NativeResult::Lisp { .. }));
    assert!(matches!(obs.datalog, NativeResult::Datalog { .. }));
    assert!(matches!(obs.clips, NativeResult::Clips { .. }));
    assert!(matches!(obs.prolog, NativeResult::Prolog { .. }));

    // Lisp value is the numeric result, not derived from Datalog.
    assert_eq!(lisp_value, "42");

    // Datalog closure contains the path facts, including derived ones.
    if let NativeResult::Datalog { closure, .. } = &obs.datalog {
        let path = closure.get("path").expect("path relation exists");
        assert!(path.contains(&vec![Value::sym("a"), Value::sym("c")]));
    } else {
        panic!("expected Datalog result");
    }

    // The structural invariant: no domain was rewritten into another.
    assert!(obs.preserves_independent_domains());
}

#[test]
fn datalog_deltas_are_base_facts_not_closure() {
    let mut db = Database::new();
    db.add_fact("edge", vec![Value::sym("a"), Value::sym("b")]);
    db.add_fact("edge", vec![Value::sym("b"), Value::sym("c")]);

    let mut program = Program::new();
    program.add_rule(Rule::with_id(
        "path-base",
        Atom::new("path", vec![Term::v("X"), Term::v("Y")]),
        vec![Atom::new("edge", vec![Term::v("X"), Term::v("Y")])],
    ));
    program.add_rule(Rule::with_id(
        "path-step",
        Atom::new("path", vec![Term::v("X"), Term::v("Z")]),
        vec![
            Atom::new("edge", vec![Term::v("X"), Term::v("Y")]),
            Atom::new("path", vec![Term::v("Y"), Term::v("Z")]),
        ],
    ));

    Evaluator::naive_fixpoint(&program, &mut db);

    let result = datalog_result(&db);

    if let NativeResult::Datalog {
        relation_deltas,
        closure,
        ..
    } = result
    {
        let edge_deltas = relation_deltas.get("edge").unwrap();
        assert_eq!(edge_deltas.len(), 2);

        let path_deltas = relation_deltas.get("path").unwrap();
        assert!(path_deltas.is_empty());

        let path_closure = closure.get("path").unwrap();
        assert_eq!(path_closure.len(), 3);
    } else {
        panic!("expected Datalog result");
    }
}

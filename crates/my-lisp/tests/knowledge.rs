use my_lisp::{eval_program, Session};

fn eval_knowledge(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/unify.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/reason.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/knowledge.my"), &mut session).unwrap();
    let result = eval_program(source, &mut session);
    match result {
        Ok(res) => {
            for line in &res.output {
                println!("{}", line);
            }
            res.value.to_string()
        },
        Err(e) => {
            println!("Output before panic:");
            for line in &session.environment.output_snapshot() {
                println!("{}", line);
            }
            panic!("evaluation failed: {e}\nsource: {source}")
        }
    }
}

#[test]
fn test_defmodule_and_reason_in() {
    let source = r#"
        (load-knowledge "../../knowledge/physics.my")
        (let ((results (reason-in 'physics '(has-mass (var x)))))
             ;; We expect the first proof result to bind (x . apple)
             (car (car results)))
    "#;
    assert_eq!(eval_knowledge(source), "((x . apple))");
}

#[test]
fn test_reason_in_unknown_module() {
    let source = r#"
        (reason-in 'biology '(is-alive cell))
    "#;
    assert_eq!(eval_knowledge(source), "Module-not-found");
}

#[test]
fn test_astronomy_module() {
    let source = r#"
        (load-knowledge "../../knowledge/astronomy.my")
        (let ((results (reason-in 'astronomy '(orbits earth sun))))
             ;; The result contains the bindings used during the proof, including rule variables
             (car (car results)))
    "#;
    assert_eq!(eval_knowledge(source), "(((s . 0) . sun) ((p . 0) . earth))");
}

#[test]
fn test_describe_collects_every_fact_about_a_symbol() {
    let source = r#"
        (load-knowledge "../../knowledge/astronomy.my")
        (describe 'earth 'astronomy)
    "#;
    // `earth` appears in one fact (`(planet earth)`); the `orbits` rule is not
    // a fact, so it is excluded even though `earth` could satisfy it.
    assert_eq!(eval_knowledge(source), "((planet earth))");
}

#[test]
fn test_describe_unknown_module() {
    let source = r#"
        (describe 'earth 'biology)
    "#;
    assert_eq!(eval_knowledge(source), "Module-not-found");
}

#[test]
fn test_describe_symbol_with_no_facts() {
    let source = r#"
        (load-knowledge "../../knowledge/astronomy.my")
        (describe 'pluto 'astronomy)
    "#;
    assert_eq!(eval_knowledge(source), "()");
}

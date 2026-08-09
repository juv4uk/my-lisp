use my_lisp::{eval_program, Session};

fn eval_knowledge(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/unify.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/reason.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/forward.my"), &mut session).unwrap();
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
fn test_forward_in_materializes_every_derivable_fact_in_a_module() {
    let source = r#"
        (load-knowledge "../../knowledge/astronomy.my")
        (forward-in 'astronomy)
    "#;
    assert_eq!(
        eval_knowledge(source),
        "((orbits earth sun) (orbits mars sun) (star sun) (planet mars) (planet earth))"
    );
}

#[test]
fn test_forward_in_unknown_module() {
    let source = r#"
        (forward-in 'biology)
    "#;
    assert_eq!(eval_knowledge(source), "Module-not-found");
}

#[test]
fn test_forward_in_chains_multiple_rules_in_one_module() {
    let source = r#"
        (load-knowledge "../../knowledge/physics.my")
        (forward-in 'physics)
    "#;
    assert_eq!(
        eval_knowledge(source),
        "((attracted-by-gravity apple) (has-mass apple))"
    );
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

#[test]
fn test_record_usage_accumulates_across_separate_queries() {
    // `record-usage!` must run directly at the top level (the global frame),
    // not nested inside a `let` — `let` desugars to an immediately-invoked
    // lambda, and `def` only ever mutates the frame it runs in, so a
    // `record-usage!` wrapped in `let` would quietly define a throwaway local
    // instead of updating the global `*usage-counts*`.
    //
    // The renamed rule head `(orbits (var (p . 0)) (var (s . 0)))` is built
    // with `cons`/`list` rather than a quoted `'(p . 0)` literal: the reader
    // has no dotted-pair syntax (a literal `.` parses as an ordinary symbol),
    // even though the printer renders real dotted pairs that way — so a
    // quoted `(p . 0)` and an actual `(cons 'p 0)` are not `equal?`.
    let source = r#"
        (load-knowledge "../../knowledge/astronomy.my")
        (def rule-key (list 'orbits (list 'var (cons 'p 0)) (list 'var (cons 's 0))))
        (def results-1 (reason-in 'astronomy '(orbits earth sun)))
        (record-usage! (second (car results-1)))
        (def results-2 (reason-in 'astronomy '(orbits mars sun)))
        (record-usage! (second (car results-2)))
        (usage-of rule-key)
    "#;
    // The `orbits` rule fired once per query, on two separate top-level
    // `record-usage!` calls; usage-of reports the running total.
    assert_eq!(eval_knowledge(source), "2");
}

#[test]
fn test_usage_of_unrecorded_rule_is_zero() {
    let source = r#"
        (usage-of (list 'orbits (list 'var (cons 'p 0)) (list 'var (cons 's 0))))
    "#;
    assert_eq!(eval_knowledge(source), "0");
}

// --- append-only fact journal --------------------------------------------
// `*knowledge-base*` (a single snapshot per module, replaced outright on
// every write) is gone; `*knowledge-journal*` is the source of truth now —
// a flat, ever-growing list of `tell`/`retract` events, and a module's
// clause list is a projection folded over it on demand.

#[test]
fn retract_knowledge_removes_a_fact_the_module_can_no_longer_prove() {
    let source = r#"
        (defmodule zoo '(((has-fur cat)) ((has-fur dog))))
        (retract-knowledge zoo '((has-fur cat)))
        (reason-in 'zoo '(has-fur cat))
    "#;
    assert_eq!(eval_knowledge(source), "()");
}

#[test]
fn retract_knowledge_leaves_the_rest_of_the_module_intact() {
    let source = r#"
        (defmodule zoo '(((has-fur cat)) ((has-fur dog))))
        (retract-knowledge zoo '((has-fur cat)))
        (car (car (reason-in 'zoo '(has-fur dog))))
    "#;
    assert_eq!(eval_knowledge(source), "()");
}

#[test]
fn a_module_retracted_down_to_nothing_is_still_a_known_module() {
    // This is exactly the distinction `module-known?` exists to preserve:
    // "no `defmodule`/`tell-knowledge` ever named this module" must read
    // differently from "this module existed, but everything it was told
    // has since been retracted" — the second case still isn't
    // `Module-not-found`, it's a known module with an empty clause list.
    let source = r#"
        (defmodule zoo '(((has-fur cat))))
        (retract-knowledge zoo '((has-fur cat)))
        (reason-in 'zoo '(has-fur cat))
    "#;
    assert_eq!(eval_knowledge(source), "()");
    let describe_source = r#"
        (defmodule zoo '(((has-fur cat))))
        (retract-knowledge zoo '((has-fur cat)))
        (describe 'cat 'zoo)
    "#;
    // `describe` returning `()` (an empty fact list, not the symbol
    // `Module-not-found`) is the proof the module is still known.
    assert_eq!(eval_knowledge(describe_source), "()");
}

#[test]
fn defmodule_called_twice_for_the_same_name_accumulates_instead_of_replacing() {
    // A deliberate behavior change from the old snapshot model, where a
    // second `defmodule` for the same name silently shadowed the first:
    // an append-only journal never discards what an earlier call told it,
    // so both calls' clauses are visible together.
    let source = r#"
        (defmodule zoo '(((has-fur cat))))
        (defmodule zoo '(((has-fur dog))))
        (list (car (car (reason-in 'zoo '(has-fur cat))))
              (car (car (reason-in 'zoo '(has-fur dog)))))
    "#;
    assert_eq!(eval_knowledge(source), "(() ())");
}

#[test]
fn tell_knowledge_and_defmodule_contributions_to_the_same_module_both_survive() {
    let source = r#"
        (defmodule zoo '(((has-fur cat))))
        (tell-knowledge zoo '(((has-fur dog))))
        (list (car (car (reason-in 'zoo '(has-fur cat))))
              (car (car (reason-in 'zoo '(has-fur dog)))))
    "#;
    assert_eq!(eval_knowledge(source), "(() ())");
}

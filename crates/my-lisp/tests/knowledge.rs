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
        (load-knowledge "../../knowledge/family.my")
        (let ((results (reason-in 'family '(parent tom (var x)))))
             ;; We expect the first proof result to bind (x . bob)
             (car (car results)))
    "#;
    assert_eq!(eval_knowledge(source), "((x . bob))");
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
        (load-knowledge "../../knowledge/family.my")
        (forward-in 'family)
    "#;
    // family.my's `ancestor` is recursive (base case: direct parent; recursive
    // case: parent of an ancestor) — this list includes transitive facts like
    // (ancestor tom jim), three hops from a fact never stated directly,
    // proving run-multi's fixpoint loop actually re-fires a rule against its
    // own prior output, not just each rule once.
    assert_eq!(
        eval_knowledge(source),
        "((ancestor tom jim) (ancestor tom pat) (ancestor tom ann) (ancestor bob jim) (ancestor tom bob) (ancestor tom liz) (ancestor bob ann) (ancestor bob pat) (ancestor pat jim) (grandparent tom ann) (grandparent tom pat) (grandparent bob jim) (parent pat jim) (parent bob pat) (parent bob ann) (parent tom liz) (parent tom bob))"
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
fn test_forward_in_chains_a_recursive_rule_through_its_own_prior_output() {
    let source = r#"
        (load-knowledge "../../knowledge/family.my")
        (reason-in 'family '(ancestor tom jim))
    "#;
    // grandparent alone (a fixed one-hop rule) cannot reach `jim` from `tom`
    // (three parent-hops away: tom -> bob -> pat -> jim); only the
    // recursive `ancestor` rule, firing against its own previously derived
    // output, can. A non-empty proof list is direct evidence the chain
    // actually recursed, not just evaluated each rule once.
    assert_ne!(eval_knowledge(source), "()");
}

#[test]
fn test_family_module() {
    let source = r#"
        (load-knowledge "../../knowledge/family.my")
        (let ((results (reason-in 'family '(grandparent tom ann))))
             ;; The result contains the bindings used during the proof, including rule variables
             (car (car results)))
    "#;
    assert_eq!(eval_knowledge(source), "(((z . 0) . bob) ((y . 0) . ann) ((x . 0) . tom))");
}

#[test]
fn test_describe_collects_every_fact_about_a_symbol() {
    let source = r#"
        (load-knowledge "../../knowledge/family.my")
        (describe 'jim 'family)
    "#;
    // `jim` appears in one fact (`(parent pat jim)`); the `grandparent`/
    // `ancestor` rules are not facts, so they're excluded even though `jim`
    // could satisfy them.
    assert_eq!(eval_knowledge(source), "((parent pat jim))");
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
        (load-knowledge "../../knowledge/family.my")
        (describe 'ringo 'family)
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
    // The renamed rule head `(grandparent (var (x . 0)) (var (y . 0)))` is
    // built with `cons`/`list` rather than a quoted `'(x . 0)` literal: the
    // reader has no dotted-pair syntax (a literal `.` parses as an ordinary
    // symbol), even though the printer renders real dotted pairs that way —
    // so a quoted `(x . 0)` and an actual `(cons 'x 0)` are not `equal?`.
    let source = r#"
        (load-knowledge "../../knowledge/family.my")
        (def rule-key (list 'grandparent (list 'var (cons 'x 0)) (list 'var (cons 'y 0))))
        (def results-1 (reason-in 'family '(grandparent tom ann)))
        (record-usage! (second (car results-1)))
        (def results-2 (reason-in 'family '(grandparent tom pat)))
        (record-usage! (second (car results-2)))
        (usage-of rule-key)
    "#;
    // The `grandparent` rule fired once per query, on two separate top-level
    // `record-usage!` calls; usage-of reports the running total.
    assert_eq!(eval_knowledge(source), "2");
}

#[test]
fn test_usage_of_unrecorded_rule_is_zero() {
    let source = r#"
        (usage-of (list 'grandparent (list 'var (cons 'x 0)) (list 'var (cons 'y 0))))
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

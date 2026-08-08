//! Exercises lib/forward.my — Step 1 of a CLIPS-style forward-chaining rule
//! engine: one working-memory list, one rule fired against one fact.
//! Перевіряє lib/forward.my — Крок 1 forward-chaining рушія в стилі CLIPS:
//! один список working memory, одне правило проти одного факту.
//! Prüft lib/forward.my — Schritt 1 einer CLIPS-artigen
//! Forward-Chaining-Regel-Engine: eine Working-Memory-Liste, eine Regel
//! gegen einen Fakt angewendet.

use my_lisp::{eval_program, Session};

fn eval_forward(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/unify.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/forward.my"), &mut session).unwrap();
    eval_program(source, &mut session)
        .unwrap_or_else(|e| panic!("evaluation failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

#[test]
fn fire_rule_produces_a_new_fact_when_the_pattern_matches() {
    let source = r#"
        (fire-rule (list (list 'planet (logic-var 'x)) (list 'has-mass (logic-var 'x)))
                   '(planet earth))
    "#;
    assert_eq!(eval_forward(source), "(has-mass earth)");
}

#[test]
fn fire_rule_returns_no_match_when_the_pattern_fails() {
    let source = r#"
        (fire-rule (list (list 'planet (logic-var 'x)) (list 'has-mass (logic-var 'x)))
                   '(star sun))
    "#;
    assert_eq!(eval_forward(source), "no-match");
}

#[test]
fn fire_rule_on_facts_collects_new_facts_and_drops_non_matches() {
    let source = r#"
        (fire-rule-on-facts (list (list 'planet (logic-var 'x)) (list 'has-mass (logic-var 'x)))
                             (list '(planet earth) '(star sun) '(planet mars)))
    "#;
    assert_eq!(eval_forward(source), "((has-mass earth) (has-mass mars))");
}

#[test]
fn fire_rule_on_facts_returns_empty_list_when_nothing_matches() {
    let source = r#"
        (fire-rule-on-facts (list (list 'planet (logic-var 'x)) (list 'has-mass (logic-var 'x)))
                             (list '(star sun) '(moon luna)))
    "#;
    assert_eq!(eval_forward(source), "()");
}

#[test]
fn fire_rule_on_working_memory_reads_the_global_fact_list() {
    let source = r#"
        (assert-fact! '(planet earth))
        (assert-fact! '(star sun))
        (assert-fact! '(planet mars))
        (fire-rule-on-working-memory (list (list 'planet (logic-var 'x)) (list 'has-mass (logic-var 'x))))
    "#;
    assert_eq!(eval_forward(source), "((has-mass mars) (has-mass earth))");
}

#[test]
fn fire_rules_on_facts_applies_every_rule_and_collects_all_results() {
    let source = r#"
        (fire-rules-on-facts
          (list (list (list 'planet (logic-var 'x)) (list 'has-mass (logic-var 'x)))
                (list (list 'star (logic-var 'x)) (list 'has-mass (logic-var 'x))))
          (list '(planet earth) '(star sun) '(moon luna)))
    "#;
    assert_eq!(eval_forward(source), "((has-mass earth) (has-mass sun))");
}

#[test]
fn fire_rules_on_working_memory_reads_the_global_fact_list() {
    let source = r#"
        (assert-fact! '(planet earth))
        (assert-fact! '(star sun))
        (fire-rules-on-working-memory
          (list (list (list 'planet (logic-var 'x)) (list 'has-mass (logic-var 'x)))
                (list (list 'star (logic-var 'x)) (list 'has-mass (logic-var 'x)))))
    "#;
    assert_eq!(eval_forward(source), "((has-mass earth) (has-mass sun))");
}

#[test]
fn assert_fact_adds_to_the_global_working_memory() {
    let source = r#"
        (assert-fact! '(planet earth))
        (assert-fact! '(planet mars))
        *working-memory*
    "#;
    assert_eq!(eval_forward(source), "((planet mars) (planet earth))");
}

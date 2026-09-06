//! B3: a small adversarial Advice Taker corpus through one semantic path:
//! candidate knowledge -> advise/advise-all -> reason-in-observe -> narrate-outcome.
//! The cases are chosen for distinct failure/derivation modes, not fixture count.

use my_lisp::{eval_program, Session};

fn eval_advice_corpus(source: &str) -> String {
    let mut session = Session::default();
    for library in [
        include_str!("../../../lib/core.my"),
        include_str!("../../../lib/unify.my"),
        include_str!("../../../lib/reason.my"),
        include_str!("../../../lib/forward.my"),
        include_str!("../../../lib/knowledge.my"),
        include_str!("../../../lib/understand.my"),
        include_str!("../../../lib/result-status.my"),
        include_str!("../../../lib/narrate.my"),
    ] {
        eval_program(library, &mut session).unwrap();
    }
    eval_program(source, &mut session)
        .unwrap_or_else(|e| panic!("evaluation failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

#[test]
fn direct_controlled_fact_reaches_proved_narration() {
    let source = r#"
        (advise astronomy (understand (quote (earth is a planet))))
        (narrate-outcome
          (reason-in-observe (quote astronomy) (quote (planet earth))))
    "#;
    assert_eq!(eval_advice_corpus(source), "(proved earth is a planet)");
}

#[test]
fn multistep_rules_survive_admission_reasoning_and_presentation() {
    let source = r#"
        (advise-all science
          (quote (
            ((planet earth))
            ((has (var x) mass) (planet (var x)))
            ((valuable (var x)) (has (var x) mass))
          )))
        (let* ((outcome
                 (reason-in-observe (quote science) (quote (valuable earth))))
               (words (narrate-outcome outcome)))
          (list (result-status outcome) (car words)))
    "#;
    assert_eq!(eval_advice_corpus(source), "(proved proved)");
}

#[test]
fn recursive_rule_survives_the_same_end_to_end_path() {
    let source = r#"
        (advise-all family
          (quote (
            ((parent alice bob))
            ((parent bob charlie))
            ((ancestor (var x) (var y)) (parent (var x) (var y)))
            ((ancestor (var x) (var y))
              (parent (var x) (var z))
              (ancestor (var z) (var y)))
          )))
        (let* ((outcome
                 (reason-in-observe
                   (quote family)
                   (quote (ancestor alice charlie))))
               (words (narrate-outcome outcome)))
          (list (result-status outcome) (car words)))
    "#;
    assert_eq!(eval_advice_corpus(source), "(proved proved)");
}

#[test]
fn known_module_unknown_query_stays_unknown_through_narration() {
    let source = r#"
        (advise astronomy (understand (quote (earth is a planet))))
        (narrate-outcome
          (reason-in-observe (quote astronomy) (quote (planet mars))))
    "#;
    assert_eq!(
        eval_advice_corpus(source),
        "(unknown because no-proof-found-for (planet mars))"
    );
}

#[test]
fn explicit_conflict_is_rejected_and_cannot_replace_the_existing_fact() {
    let source = r#"
        (advise ethics (quote ((mortal socrates))))
        (let* ((decision
                 (advise ethics (quote ((not (mortal socrates))))))
               (outcome
                 (reason-in-observe (quote ethics) (quote (mortal socrates)))))
          (list (car decision) (result-status outcome)))
    "#;
    assert_eq!(eval_advice_corpus(source), "(conflict proved)");
}

#[test]
fn malformed_advice_is_rejected_and_does_not_become_unknown_knowledge() {
    let source = r#"
        (advise geometry (quote ((point origin))))
        (let* ((decision (advise geometry (quote (point broken))))
               (outcome
                 (reason-in-observe (quote geometry) (quote (point broken)))))
          (list (car decision) (result-status outcome)))
    "#;
    assert_eq!(eval_advice_corpus(source), "(rejected unknown)");
}

#[test]
fn knowledge_package_round_trip_precedes_reasoning_without_eval_of_package_data() {
    let source = r#"
        (def package
          (make-knowledge-package
            (quote archive)
            (quote (((planet mars))))))
        (def *knowledge-journal* (quote ()))
        (import-knowledge-package package)
        (narrate-outcome
          (reason-in-observe (quote archive) (quote (planet mars))))
    "#;
    assert_eq!(eval_advice_corpus(source), "(proved mars is a planet)");
}

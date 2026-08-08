//! Exercises lib/clips-import.my — Step 1 of a universal-ish importer for
//! old symbolic-AI systems: CLIPS's `deffacts` (facts only, no variables),
//! per PLAN.md's forward-chaining/CLIPS thread and NASA's Johnson Space
//! Center origin of CLIPS itself. Real CLIPS source parses cleanly as
//! ordinary my-lisp data via `read`/`quote` — no dedicated tokenizer or
//! string primitives needed.
//! Перевіряє lib/clips-import.my — Крок 1 універсального(-уватого)
//! імпортера зі старих символьних AI-систем: CLIPS `deffacts` (факти без
//! змінних). Справжній CLIPS-код чисто парситься як звичайні my-lisp дані
//! через `read`/`quote` — без окремого токенізатора чи рядкових
//! примітивів.
//! Prüft lib/clips-import.my — Schritt 1 eines universell(-ähnlichen)
//! Importers für alte symbolische KI-Systeme: CLIPS' `deffacts` (Fakten
//! ohne Variablen). Echter CLIPS-Quellcode parst sauber als gewöhnliche
//! my-lisp-Daten über `read`/`quote` — kein eigener Tokenizer oder
//! String-Primitive nötig.

use my_lisp::{eval_program, Session};

fn eval_import(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/unify.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/reason.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/forward.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/knowledge.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/clips-import.my"), &mut session).unwrap();
    eval_program(source, &mut session)
        .unwrap_or_else(|e| panic!("evaluation failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

#[test]
fn clips_deffacts_becomes_zero_condition_clauses() {
    // A real CLIPS `deffacts` block, read as data rather than evaluated.
    let source = r#"
        (clips-import '((deffacts initial-facts (planet earth) (planet mars) (star sun))))
    "#;
    assert_eq!(
        eval_import(source),
        "(((planet earth)) ((planet mars)) ((star sun)))"
    );
}

#[test]
fn clips_import_skips_unsupported_forms_without_erroring() {
    // deffunction isn't supported (no step covers it) — a mixed file
    // still imports whatever it can, rather than failing the whole import.
    let source = r#"
        (clips-import '(
            (deffacts initial-facts (planet earth))
            (deffunction square (?x) (* ?x ?x))
        ))
    "#;
    assert_eq!(eval_import(source), "(((planet earth)))");
}

#[test]
fn clips_defrule_converts_question_mark_variables_to_var_terms() {
    let source = r#"
        (clips-import '((defrule mass-rule (planet ?x) => (assert (has-mass ?x)))))
    "#;
    assert_eq!(
        eval_import(source),
        "(((has-mass (var x)) (planet (var x))))"
    );
}

#[test]
fn clips_defrule_leaves_non_variable_arguments_untouched() {
    // A CLIPS fact argument can be a plain symbol or a number, not just a
    // `?`-prefixed variable — clips-var? must not choke on either.
    let source = r#"
        (clips-import '((defrule hot (temperature 98) => (assert (alert critical)))))
    "#;
    assert_eq!(eval_import(source), "(((alert critical) (temperature 98)))");
}

#[test]
fn clips_defrule_with_multiple_conditions_converts_all_of_them() {
    let source = r#"
        (clips-import '((defrule grandparent-rule
                           (parent ?x ?z) (parent ?z ?y)
                           => (assert (grandparent ?x ?y)))))
    "#;
    assert_eq!(
        eval_import(source),
        "(((grandparent (var x) (var y)) (parent (var x) (var z)) (parent (var z) (var y))))"
    );
}

#[test]
fn clips_defrule_with_no_asserts_imports_as_no_clauses() {
    let source = r#"
        (clips-import '((defrule broken (planet ?x) => (printout t "hello"))))
    "#;
    assert_eq!(eval_import(source), "()");
}

#[test]
fn clips_defrule_with_multiple_asserts_produces_one_clause_per_assert() {
    // N assertions sharing one LHS become N clauses, each with the same
    // (converted) condition list — logically equivalent to CLIPS firing
    // all N assertions together whenever the shared conditions hold.
    let source = r#"
        (clips-import '((defrule two-asserts (planet ?x) => (assert (a ?x)) (assert (b ?x)))))
    "#;
    assert_eq!(
        eval_import(source),
        "(((a (var x)) (planet (var x))) ((b (var x)) (planet (var x))))"
    );
}

#[test]
fn clips_import_mixes_deffacts_and_defrule_into_one_usable_module() {
    let source = r#"
        (def imported (clips-import '(
            (deffacts init (planet earth))
            (defrule mass-rule (planet ?x) => (assert (has-mass ?x)))
        )))
        (defmodule imported-astro imported)
        (forward-in 'imported-astro)
    "#;
    assert_eq!(
        eval_import(source),
        "((has-mass earth) (planet earth))"
    );
}

#[test]
fn clips_import_file_reads_and_imports_a_real_clp_file() {
    // tests/fixtures/astronomy.clp is a genuine CLIPS source file, not a
    // caller-supplied quoted literal — the actual "connect to old
    // symbolic-AI systems" tool this whole file exists to build.
    let source = r#"
        (def imported (clips-import-file "../../tests/fixtures/astronomy.clp"))
        (defmodule imported-astro imported)
        (forward-in 'imported-astro)
    "#;
    assert_eq!(
        eval_import(source),
        "((orbits earth sun) (orbits mars sun) (star sun) (planet mars) (planet earth))"
    );
}

#[test]
fn clips_defrule_with_a_not_condition_imports_and_runs_correctly() {
    // Was skipped entirely as of clips-import.my Step 5, back when
    // lib/forward.my's match-conditions had no negation-as-failure
    // handling. Step 6 added it (match-one-condition/
    // match-negated-condition), so this now imports normally and derives
    // exactly the right fact: tweety (animal, not penguin) becomes a
    // bird; pingu (a penguin) does not.
    let source = r#"
        (def imported (clips-import '(
            (deffacts init (animal tweety) (animal pingu) (penguin pingu))
            (defrule bird-rule (animal ?x) (not (penguin ?x)) => (assert (bird ?x)))
        )))
        (defmodule zoo imported)
        (forward-in 'zoo)
    "#;
    assert_eq!(
        eval_import(source),
        "((bird tweety) (penguin pingu) (animal pingu) (animal tweety))"
    );
}

#[test]
fn clips_deftemplate_converts_named_slots_to_positional_order() {
    let source = r#"
        (clips-import '(
            (deftemplate reading (slot sensor) (slot value))
            (deffacts init (reading (value 98) (sensor probe1)))
        ))
    "#;
    assert_eq!(eval_import(source), "(((reading probe1 98)))");
}

#[test]
fn clips_deftemplate_slot_order_holds_regardless_of_slot_order_in_the_fact() {
    let source = r#"
        (clips-import '(
            (deftemplate reading (slot sensor) (slot value))
            (deffacts init (reading (sensor probe1) (value 98)))
        ))
    "#;
    assert_eq!(eval_import(source), "(((reading probe1 98)))");
}

#[test]
fn clips_deftemplate_applies_inside_defrule_conditions_and_conclusions() {
    let source = r#"
        (clips-import '(
            (deftemplate reading (slot sensor) (slot value))
            (defrule hot-rule (reading (sensor ?s) (value 98))
                => (assert (alert (sensor ?s))))
            (deftemplate alert (slot sensor))
        ))
    "#;
    assert_eq!(
        eval_import(source),
        "(((alert (var s)) (reading (var s) 98)))"
    );
}

#[test]
fn clips_deftemplate_works_end_to_end_through_forward_in() {
    let source = r#"
        (def imported (clips-import '(
            (deftemplate reading (slot sensor) (slot value))
            (deffacts init (reading (sensor probe1) (value 98)))
            (defrule hot-rule (reading (sensor ?s) (value 98)) => (assert (alert ?s)))
        )))
        (defmodule sensors imported)
        (forward-in 'sensors)
    "#;
    assert_eq!(
        eval_import(source),
        "((alert probe1) (reading probe1 98))"
    );
}

#[test]
fn clips_import_result_feeds_straight_into_defmodule() {
    // The whole point: no hand-editing step between import and use.
    let source = r#"
        (def imported-clauses (clips-import '((deffacts initial-facts (planet earth) (star sun)))))
        (defmodule imported imported-clauses)
        (reason-in 'imported '(planet earth))
    "#;
    assert_eq!(
        eval_import(source),
        "((() (proved (planet earth) (planet earth) ())))"
    );
}

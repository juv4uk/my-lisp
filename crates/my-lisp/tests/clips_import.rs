//! Exercises lib/clips-import.my — Step 1 of a universal-ish importer for
//! old symbolic-AI systems: CLIPS's `deffacts` (facts only, no variables),
//! per PLAN.md's forward-chaining/CLIPS thread and NASA's Johnson Space
//! Center origin of CLIPS itself. Real CLIPS source parses cleanly as
//! ordinary my-lisp data via `read`/`quote` — no dedicated tokenizer or
//! string primitives needed.
//! Pereviriaie lib/clips-import.my — Krok 1 universalnoho(-uvatoho)
//! importera zi starykh symvolnykh AI-system: CLIPS `deffacts` (fakty bez
//! zminnykh). Spravzhnii CLIPS-kod chysto parsytsia yak zvychaini my-lisp dani
//! cherez `read`/`quote` — bez okremoho tokenizatora chy riadkovykh
//! prymityviv.
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
fn clips_defrule_strips_a_docstring_before_its_conditions() {
    // Regression: an empty docstring "" right after the rule name used to
    // land in the condition list as a stray, never-matching pattern,
    // silently killing the whole rule.
    let source = r#"
        (clips-import '((defrule mass-rule "explains why planets have mass"
                           (planet ?x) => (assert (has-mass ?x)))))
    "#;
    assert_eq!(
        eval_import(source),
        "(((has-mass (var x)) (planet (var x))))"
    );
}

#[test]
fn clips_defrule_strips_a_declare_salience_before_its_conditions() {
    let source = r#"
        (clips-import '((defrule mass-rule (declare (salience 10))
                           (planet ?x) => (assert (has-mass ?x)))))
    "#;
    assert_eq!(
        eval_import(source),
        "(((has-mass (var x)) (planet (var x))))"
    );
}

#[test]
fn clips_defrule_strips_both_docstring_and_declare_together() {
    let source = r#"
        (clips-import '((defrule mass-rule "docstring" (declare (salience 10))
                           (planet ?x) => (assert (has-mass ?x)))))
    "#;
    assert_eq!(
        eval_import(source),
        "(((has-mass (var x)) (planet (var x))))"
    );
}

#[test]
fn clips_import_file_imports_a_second_real_external_clp_file_correctly() {
    // tests/fixtures/auto-external.clp is CLIPS's own "Automotive Expert
    // System" example — every one of its 21 rules has an empty docstring,
    // several also a (declare (salience N)); before Step 16 every single
    // one silently never fired. Checking a handful of representative
    // rules imported with clean condition lists (no stray "" or declare)
    // is enough to prove the fix, without pinning all 21 verbatim.
    let source = r#"
        (clips-import-file "../../tests/fixtures/auto-external.clp")
    "#;
    let imported = eval_import(source);
    assert!(
        !imported.contains("declare"),
        "a stray (declare ...) leaked into a condition list: {imported}"
    );
    // The very first clause: no leading "" docstring before its conditions.
    assert!(
        imported.starts_with("(((engine-starts"),
        "expected the first clause to start cleanly with the engine-starts fact, got: {imported}"
    );
}

#[test]
fn clips_defrule_with_a_multi_fact_assert_produces_one_clause_per_fact() {
    // Real CLIPS assert can take multiple facts in one call:
    // (assert (number 0) (number 1) (number 2)) is one assert, three
    // facts — found on a genuine external .clp file, not guessed.
    let source = r#"
        (clips-import '((defrule init (foo) => (assert (number 0) (number 1) (number 2)))))
    "#;
    assert_eq!(
        eval_import(source),
        "(((number 0) (foo)) ((number 1) (foo)) ((number 2) (foo)))"
    );
}

#[test]
fn clips_import_file_imports_a_real_external_clp_file_from_the_official_clips_examples() {
    // tests/fixtures/wordgame-external.clp is CLIPS's own GERALD+DONALD=
    // ROBERT word-puzzle example, downloaded verbatim — the actual
    // interoperability test: does our importer handle a file we didn't
    // write, not tailored to our importer's coverage?
    let source = r#"
        (clips-import-file "../../tests/fixtures/wordgame-external.clp")
    "#;
    // All twenty facts from startup's single multi-fact assert import
    // correctly, plus generate-combinations' single-fact rule; find-solution
    // has no assert at all (printout only) so contributes nothing — none
    // of that is a bug, that's this rule genuinely having nothing to
    // assert.
    assert_eq!(
        eval_import(source),
        "(((number 0)) ((number 1)) ((number 2)) ((number 3)) ((number 4)) ((number 5)) ((number 6)) ((number 7)) ((number 8)) ((number 9)) ((letter G)) ((letter E)) ((letter R)) ((letter A)) ((letter L)) ((letter D)) ((letter O)) ((letter N)) ((letter B)) ((letter T)) ((combination (var a) (var x)) (number (var x)) (letter (var a))))"
    );
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
fn clips_defrule_with_printout_alongside_assert_still_imports_the_assert() {
    // Regression: a rule mixing printout (debug output) with assert used
    // to import as no clauses at all, silently dropping a perfectly
    // representable assertion just because printout sat next to it.
    let source = r#"
        (clips-import '((defrule mass-rule (planet ?x)
            => (printout t "found planet " ?x crlf) (assert (has-mass ?x)))))
    "#;
    assert_eq!(
        eval_import(source),
        "(((has-mass (var x)) (planet (var x))))"
    );
}

#[test]
fn clips_defrule_with_retract_still_imports_as_no_clauses() {
    // retract refers to a fact by CLIPS fact-address, a concept this
    // project's set-of-facts model has no equivalent for — still
    // disqualifying, unlike the harmless printout above.
    let source = r#"
        (clips-import '((defrule broken (planet ?x) => (retract ?x) (assert (has-mass ?x)))))
    "#;
    assert_eq!(eval_import(source), "()");
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
fn clips_deftemplate_converts_facts_nested_inside_or() {
    // Regression: clips-convert-template used to recurse through a `not`
    // wrapper only, so a template fact inside `or`/`and` never got its
    // named slots converted to positional form, silently never matching
    // anything.
    let source = r#"
        (def imported (clips-import '(
            (deftemplate cat (slot name))
            (deftemplate dog (slot name))
            (deffacts init (cat (name tom)) (dog (name rex)))
            (defrule pet-rule (or (cat (name ?x)) (dog (name ?x))) => (assert (pet ?x)))
        )))
        (defmodule zoo imported)
        (forward-in 'zoo)
    "#;
    assert_eq!(
        eval_import(source),
        "((pet rex) (pet tom) (dog rex) (cat tom))"
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

#[test]
fn clips_deftemplate_name_is_matched_regardless_of_a_defmodule_qualifier() {
    // Regression: real CLIPS files namespace deftemplate names with a
    // `defmodule` prefix (`QUESTIONS::question`), then assert plain
    // `(question ...)` facts elsewhere — CLIPS resolves that against
    // whichever module is current. This importer models no modules at
    // all, so both the template's own registered name and any fact head
    // referring to it must be compared with the `defmodule::` prefix
    // stripped, or the named-slot conversion silently never fires.
    let source = r#"
        (clips-import '(
            (deftemplate QUESTIONS::question (slot attribute) (slot text))
            (deffacts init (question (attribute color) (text "what color?")))
        ))
    "#;
    assert_eq!(
        eval_import(source),
        "(((question color \"what color?\")))"
    );
}

#[test]
fn clips_condition_slot_with_no_value_does_not_crash_the_importer() {
    // Regression: a CLIPS condition can name a multislot with no value at
    // all, e.g. `(precursors)`, meaning "match regardless of what's in
    // this slot". clips-slot-value-of used to assume every slot form has
    // a value part and called `second` unconditionally, crashing with
    // "car expects a non-empty list" the first time a real file used this
    // shorthand.
    let source = r#"
        (clips-import '(
            (deftemplate question (slot attribute) (multislot precursors))
            (defrule ask (question (attribute ?a) (precursors)) => (assert (asked ?a)))
        ))
    "#;
    assert_eq!(
        eval_import(source),
        "(((asked (var a)) (question (var a) ())))"
    );
}

#[test]
fn clips_import_file_imports_a_third_real_external_clp_file_with_module_qualified_templates() {
    // wine.clp (CLIPS's own "Wine Expert Sample Problem") is the first
    // real external example to combine defmodule-qualified deftemplate
    // names with deffacts entries that reference them by bare name, and
    // to use CLIPS's valueless-multislot condition shorthand — both bugs
    // above were found through this exact file.
    let source = r#"
        (clips-import-file "../../tests/fixtures/wine-external.clp")
    "#;
    let imported = eval_import(source);
    assert!(
        imported.contains(
            "(question main-component \"Is the main component of the meal meat, fish, or poultry? \" meat () ())"
        ),
        "expected the main-component question fact to be converted to positional form, got: {imported}"
    );
    assert!(
        imported.contains("(wine Gamay red medium medium)"),
        "expected a wine fact to be converted to positional form, got: {imported}"
    );
}

#[test]
fn clips_import_stays_stack_safe_on_a_deffacts_block_with_many_facts() {
    // Regression: clips-facts->clauses/clips-import-forms used to build
    // their result with `(cons ... (recurse ...))`/`(append ... (recurse
    // ...))`, neither a tail call, so the Rust stack grew one frame per
    // fact/form. A synthetic 5,000-fact deffacts reproduces the same
    // shape as the real external file that first hit this (a 128-fact
    // block in animal-external.clp) without depending on that fixture or
    // network access — mirrors how tests/stack_safety.rs regression-tests
    // lib/core.my's own tail-recursive list utilities on a 100,000-element
    // list.
    let mut facts = String::new();
    for i in 0..5000 {
        facts.push_str(&format!("(number {i})"));
    }
    let source = format!(
        r#"(clips-import '((deffacts many (goal x)) (deffacts nums {facts})))"#
    );
    let imported = eval_import(&source);
    assert!(
        imported.contains("(number 0)") && imported.contains("(number 4999)"),
        "expected every fact to survive the import, got a result of length {}",
        imported.len()
    );
}

#[test]
fn clips_import_file_imports_a_fourth_real_external_clp_file_with_a_large_deffacts_block() {
    // animal.clp ("Animal Identification Expert System") is the real file
    // that first crashed the process outright with a Rust stack overflow
    // (not a graceful LanguageError) — its knowledge-base deffacts has 128
    // facts, enough non-tail recursion depth (compounded across all 37 of
    // the file's top-level forms) to blow the stack before the fix above.
    let source = r#"
        (clips-import-file "../../tests/fixtures/animal-external.clp")
    "#;
    let imported = eval_import(source);
    assert!(
        imported.contains("(goal type.animal)"),
        "expected the initial goal fact to survive the import, got: {imported}"
    );
    assert!(
        imported.contains("(legalanswers yes)"),
        "expected a converted deftemplate fact to survive the import, got: {imported}"
    );
}

#[test]
fn clips_import_file_imports_a_fifth_real_external_clp_file_with_mostly_disqualified_rules() {
    // manners.clp (the classic OPS5-derived "Manners Benchmark", also
    // distributed with CLIPS) didn't surface a new bug — every one of its
    // 8 rules already exercises policy this importer already has right:
    // `defglobal` is ignored (not a deffacts/defrule, produces no
    // clauses), 6 of the 8 rules use `modify`/`retract`/`(halt)` on their
    // right-hand side and correctly import as no clauses, and one
    // condition even uses CLIPS's `~?var` connective-constraint syntax
    // (documented as unsupported — read as an oddly-named ordinary
    // variable) inside a rule that's disqualified anyway. Only
    // `make_path`, whose RHS is a single plain `assert`, survives. Worth
    // locking in as a regression precisely because it's a differently-
    // shaped real file (no deffacts at all — this benchmark loads its
    // guest data separately) that exercises almost every disqualification
    // rule at once without crashing or importing anything incorrect.
    let source = r#"
        (clips-import-file "../../tests/fixtures/manners-external.clp")
    "#;
    assert_eq!(
        eval_import(source),
        "(((path (var id) (var n1) (var s)) (context make_path) (seating () () () () (var id) (var pid) no) (path (var pid) (var n1) (var s)) (not (path (var id) (var n1) ()))))"
    );
}

#[test]
fn clips_import_file_imports_a_sixth_real_external_clp_file_with_duplicate_and_embedded_constraints()
{
    // dilemma1.clp ("Farmer's Dilemma", CLIPS's own cannibals-and-goat
    // search example) is another clean confirmation, not a new bug.
    // Every one of its defrules uses `duplicate` (a CLIPS action this
    // importer has never seen before — an unrecognized non-assert action,
    // same "not an assert form" bucket as `halt`), `retract`, or `modify`
    // on its right-hand side, so all of them correctly import as no
    // clauses. One condition even writes an embedded test constraint
    // directly against a symbol with no separating whitespace —
    // `(search-depth ?sd2&:(< ?sd1 ?sd2))` — which the reader splits into
    // a stray extra list item rather than one token; harmless here since
    // that whole rule (`circular-path`) is disqualified by its own
    // `retract` anyway. Only the two `deffacts` blocks (module-qualified
    // `MAIN::status`/`MAIN::opposites`, exercising the Step 17 prefix fix
    // again) survive.
    let source = r#"
        (clips-import-file "../../tests/fixtures/dilemma1-external.clp")
    "#;
    assert_eq!(
        eval_import(source),
        "(((status 1 no-parent shore-1 shore-1 shore-1 shore-1 no-move)) ((opposite-of shore-1 shore-2)) ((opposite-of shore-2 shore-1)))"
    );
}

#[test]
fn clips_import_file_imports_a_seventh_real_external_clp_file_at_scale() {
    // zebra.clp (the classic "Who owns the Zebra?" logic puzzle) is
    // another clean confirmation, not a new bug — but it's the first real
    // file to exercise the importer at real scale and hit two genuinely
    // new-in-combination edge cases without crashing:
    //
    // - `find-solution` has 25 conditions, all against the same
    //   `(deftemplate avh (field a) (field v) (field h))` template (note
    //   `field`, not `slot` — irrelevant to this importer, which only
    //   ever reads the slot *name*, never the keyword in front of it),
    //   most wrapped in CLIPS connective-constraint syntax (`~?n1`,
    //   `&=(+ ?c2 1)`, `&:(or ...)`) already known to be read as odd
    //   variable names or stray extra list elements rather than
    //   evaluated — confirmed harmless at 25-conditions-on-one-template
    //   scale, not just in isolation.
    // - `startup` is the first *surviving* rule (RHS is `printout` +
    //   `assert`, no retract/modify) whose left-hand side has zero
    //   conditions at all — straight from the rule name to `=>`. Earlier
    //   zero-condition rules (wine.clp's `start`, dilemma2.clp's
    //   `start-it`) were always disqualified for an unrelated reason
    //   before this path mattered.
    //
    // `print-solution` and `generate-combinations` both use `retract` and
    // correctly contribute no clauses.
    let source = r#"
        (clips-import-file "../../tests/fixtures/zebra-external.clp")
    "#;
    let imported = eval_import(source);
    assert!(
        imported.contains("(value color red)"),
        "expected startup's zero-condition rule to survive and produce a clause, got: {imported}"
    );
    assert!(
        imported.contains("(solution pet zebra (var p5)) (avh nationality englishman (var n1))"),
        "expected find-solution's 25-condition body to convert correctly, got: {imported}"
    );
    let source_count = format!("(length {source})");
    assert_eq!(
        eval_import(&source_count),
        "50",
        "expected 25 clauses from find-solution's 25 asserts plus 25 from startup's 25 asserts"
    );
}

#[test]
fn clips_import_file_imports_an_eighth_real_external_clp_file_with_bare_tilde_constraints() {
    // mab.clp ("Monkees and Bananas", CLIPS's own planning-problem
    // example) is another clean confirmation, not a new bug, but the
    // first real file to use a *bare* `~var`/`~symbol` connective
    // constraint as a whole slot value on its own — e.g. `(monkey
    // (holding ~?chest))` and `(thing (on-top-of ~floor) ...)` — rather
    // than only ever appearing as the tail of an `&`-chain like
    // `?x&~?y`. Same known limitation (read as one oddly-shaped symbol,
    // not evaluated), confirmed harmless in this new shape too. It's
    // also the first file with 33 defrules at once, most combining
    // fact-address bindings (`?monkey <-`) with both `not` conditions and
    // `modify` on the right-hand side — every `modify`-bearing rule
    // correctly disqualifies, every pure-assert rule (all with an empty
    // `""` docstring, per Step 16) correctly survives.
    let source = r#"
        (clips-import-file "../../tests/fixtures/mab-external.clp")
    "#;
    let imported = eval_import(source);
    assert!(
        imported.contains("(monkey () () ~?chest)"),
        "expected a bare ~?var connective constraint to survive as an odd symbol, got: {imported}"
    );
    assert!(
        imported.contains("(thing (var chest) () ~floor light)"),
        "expected a bare ~symbol connective constraint to survive as an odd symbol, got: {imported}"
    );
    assert!(
        imported.contains("(monkey t5-7 green-couch blank)"),
        "expected startup's zero-condition rule to survive and produce a clause, got: {imported}"
    );
    let source_count = format!("(length {source})");
    assert_eq!(
        eval_import(&source_count),
        "30",
        "expected 17 goal-directed rules plus startup's 13 asserted facts to survive"
    );
}

#[test]
fn clips_defrule_with_an_exists_condition_imports_and_converts_its_template() {
    // Once lib/forward.my Step 15 gave `exists`/`forall` real dispatcher
    // support, the Step 19 skip-guard in this file was removed (same as
    // Step 5's `not` guard was removed after lib/forward.my Step 7).
    // `clips-convert-template` also learned to recurse into `exists`
    // (same lesson Step 13 taught for `not`/`or`/`and`), so a named-slot
    // fact nested inside it gets converted to positional form too.
    let source = r#"
        (clips-import '(
            (deftemplate unsolved (slot row) (slot column))
            (defrule wait-for-more
                (rank (value ?last))
                (exists (unsolved (row ?r) (column ?c)))
                =>
                (assert (rank (value done))))))
    "#;
    assert_eq!(
        eval_import(source),
        "(((rank (value done)) (rank (value (var last))) (exists (unsolved (var r) (var c)))))"
    );
}

#[test]
fn clips_defrule_with_a_forall_condition_imports_correctly() {
    let source = r#"
        (clips-import '((defrule all-checked
            (forall (item ?x) (checked ?x))
            =>
            (assert (done)))))
    "#;
    assert_eq!(
        eval_import(source),
        "(((done) (forall (item (var x)) (checked (var x)))))"
    );
}

#[test]
fn clips_import_file_imports_a_ninth_real_external_clp_file_with_a_real_exists_condition() {
    // sudoku.clp (CLIPS's own Sudoku solver) is the real file that first
    // used `exists` — `(exists (unsolved))`, "true if at least one
    // unsolved cell remains" — inside a rank-selection rule. No deffacts
    // in this file (puzzle data loads separately); 12 defrules, most
    // disqualified by `not`/`test`-heavy conditions converting correctly
    // (already-known limitations, not bugs) or surviving as genuine
    // working clauses, including the one with `exists` itself.
    let source = r#"
        (clips-import-file "../../tests/fixtures/sudoku-external.clp")
    "#;
    let imported = eval_import(source);
    assert!(
        imported.contains("(exists (unsolved () ()))"),
        "expected the exists-bearing rule to import with its template converted, got: {imported}"
    );
    assert!(
        imported.contains("(size-value 1 1)"),
        "expected a converted deftemplate fact to survive the import, got: {imported}"
    );
}

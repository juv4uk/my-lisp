//! Exercises lib/understand.my — the controlled-natural-language bridge from
//! a fixed-shape word list to a knowledge clause, per private/lisp-to-knowledge.md
//! §6. No string primitives involved: input is already a word list, not text.
//! Pereviriaie lib/understand.my — mistok kontrolovanoi pryrodnoi movy vid
//! spysku sliv fiksovanoi formy do znannievoho clause, za
//! private/lisp-to-knowledge.md §6. Bez riadkovykh prymityviv: vkhid — uzhe
//! spysok sliv, ne tekst.
//! Prüft lib/understand.my — die Brücke kontrollierter natürlicher Sprache
//! von einer Wortliste fester Form zu einem Wissens-Clause, gemäß
//! private/lisp-to-knowledge.md §6. Ohne String-Primitive: die Eingabe ist
//! bereits eine Wortliste, kein Text.

use my_lisp::{eval_program, Session};

fn eval_understand(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/unify.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/reason.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/understand.my"), &mut session).unwrap();
    eval_program(source, &mut session)
        .unwrap_or_else(|e| panic!("evaluation failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

#[test]
fn understand_is_a_produces_a_class_membership_fact() {
    assert_eq!(
        eval_understand("(understand '(earth is a planet))"),
        "((planet earth))"
    );
}

#[test]
fn understand_is_without_article_produces_the_same_fact() {
    assert_eq!(
        eval_understand("(understand '(earth is round))"),
        "((round earth))"
    );
}

#[test]
fn understand_subject_verb_object_produces_a_relation_fact() {
    assert_eq!(
        eval_understand("(understand '(earth orbits sun))"),
        "((orbits earth sun))"
    );
}

#[test]
fn understand_all_have_produces_a_universal_rule() {
    assert_eq!(
        eval_understand("(understand '(all planet have mass))"),
        "((has (var w) mass) (planet (var w)))"
    );
}

#[test]
fn understand_output_is_usable_directly_as_a_reason_rule() {
    // The whole point of `understand` is to feed straight into the existing
    // reasoning engine without hand-editing — no separate translation step.
    let source = r#"
        (let ((fact1 (understand '(earth is a planet)))
              (rule1 (understand '(all planet have mass))))
             (let ((rules (list rule1 fact1)))
                  (length (reason '(has earth mass) rules))))
    "#;
    assert_eq!(eval_understand(source), "1");
}

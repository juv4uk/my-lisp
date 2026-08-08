//! Exercises lib/reason.my — the symbolic inference engine written in my-lisp
//! itself, fulfilling the Advice Taker vision of deriving new facts from rules.
//! Loads core, unify, and reason into one session.
//! Перевіряє lib/reason.my — рушій символьного висновку, написаний самою
//! my-lisp, що втілює бачення Advice Taker щодо виведення фактів з правил.
//! Завантажує core, unify та reason в одну сесію.
//! Prüft lib/reason.my — die symbolische Inferenz-Engine, geschrieben in
//! my-lisp selbst, die die Advice-Taker-Vision erfüllt. Lädt core, unify
//! und reason in eine Sitzung.

use my_lisp::{eval_program, Session};

fn eval_reason(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/unify.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/reason.my"), &mut session).unwrap();
    eval_program(source, &mut session)
        .unwrap_or_else(|e| panic!("evaluation failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

#[test]
fn simple_fact_retrieval() {
    let source = r#"
        (let ((rules '(((parent alice bob)))))
             (reason '(parent alice bob) rules))
    "#;
    // Returns a list containing one empty substitution (meaning it succeeded with no variables bound)
    assert_eq!(eval_reason(source), "(())");
}

#[test]
fn variable_binding_from_fact() {
    let source = r#"
        (let ((rules '(((parent alice bob)))))
             (reason (list 'parent (logic-var 'x) 'bob) rules))
    "#;
    // Returns a list of substitutions. One successful path binding x to alice.
    assert_eq!(eval_reason(source), "(((x . alice)))");
}

#[test]
fn rule_with_condition_backward_chaining() {
    let source = r#"
        (let ((rules '(
                 ((grandparent (var x) (var y)) (parent (var x) (var z)) (parent (var z) (var y)))
                 ((parent alice bob))
                 ((parent bob charlie))
               )))
             (length (reason (list 'grandparent (logic-var 'a) (logic-var 'b)) rules)))
    "#;
    // Should find exactly one valid path
    assert_eq!(eval_reason(source), "1");
}

#[test]
fn multiple_valid_paths() {
    let source = r#"
        (let ((rules '(
                 ((sibling (var x) (var y)) (parent (var z) (var x)) (parent (var z) (var y)))
                 ((parent alice bob))
                 ((parent alice charlie))
               )))
             (length (reason (list 'sibling 'bob 'charlie) rules)))
    "#;
    // Should find a path via parent 'alice'.
    assert_eq!(eval_reason(source), "1");
}

#[test]
fn recursive_rule_standardizing_apart() {
    // Tests that variable names don't collide across recursive rule invocations.
    // Without standardizing apart, the `z` in the first invocation of the recursive rule
    // would collide with the `x`, `y`, or `z` in the inner invocations.
    let source = r#"
        (let ((rules '(
                 ((ancestor (var x) (var y)) (parent (var x) (var y)))
                 ((ancestor (var x) (var y)) (parent (var x) (var z)) (ancestor (var z) (var y)))
                 
                 ((parent alice bob))
                 ((parent bob charlie))
                 ((parent charlie dave))
               )))
             (length (reason (list 'ancestor 'alice 'dave) rules)))
    "#;
    // alice -> bob -> charlie -> dave = 1 valid path
    assert_eq!(eval_reason(source), "1");
}

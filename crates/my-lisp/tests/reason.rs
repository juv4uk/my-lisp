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

fn eval_reason_with_output(source: &str) -> Vec<String> {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/unify.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/reason.my"), &mut session).unwrap();
    eval_program(source, &mut session)
        .unwrap_or_else(|e| panic!("evaluation failed: {e}\nsource: {source}"))
        .output
}

#[test]
fn simple_fact_retrieval() {
    let source = r#"
        (let ((rules '(((parent alice bob)))))
             (reason '(parent alice bob) rules))
    "#;
    // Returns a list containing one empty substitution and its proof tree
    assert_eq!(eval_reason(source), "((() (proved (parent alice bob) (parent alice bob) ())))");
}

#[test]
fn variable_binding_from_fact() {
    let source = r#"
        (let ((rules '(((parent alice bob)))))
             (reason (list 'parent (logic-var 'x) 'bob) rules))
    "#;
    // Returns a list of substitutions and proof trees. One successful path binding x to alice.
    assert_eq!(eval_reason(source), "((((x . alice)) (proved (parent (var x) bob) (parent alice bob) ())))");
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

#[test]
fn negation_as_failure() {
    let source = r#"
        (let ((rules '(
                 ((bird (var x)) (animal (var x)) (not (penguin (var x))))
                 ((animal tweety))
                 ((animal pingu))
                 ((penguin pingu))
               )))
             (reason (list 'bird (logic-var 'x)) rules))
    "#;
    // Only tweety is a bird because pingu is a penguin, so the 'not' fails for pingu.
    assert_eq!(eval_reason(source), "(((((x . 0) . tweety) (x var (x . 0))) (proved (bird (var x)) (bird (var (x . 0))) ((proved (animal (var (x . 0))) (animal tweety) ()) (proved-not (penguin (var (x . 0))))))))");
}

#[test]
fn test_explain_proof() {
    let source = r#"
        (let ((rules '(
                 ((bird (var x)) (animal (var x)) (not (penguin (var x))))
                 ((animal tweety))
                 ((animal pingu))
                 ((penguin pingu))
               )))
             (let* ((results (reason (list 'bird (logic-var 'x)) rules))
                    (proof (second (car results))))
               (explain-proof proof)))
    "#;
    let output = eval_reason_with_output(source);
    assert_eq!(
        output,
        vec![
            "Proved:", "(bird (var x))", "using", "rule:", "(bird (var (x . 0)))",
            "..", "|-", "Proved:", "(animal (var (x . 0)))", "using", "rule:", "(animal tweety)",
            "..", "|-", "Proved", "by", "failure:", "not", "(penguin (var (x . 0)))"
        ]
    );
}

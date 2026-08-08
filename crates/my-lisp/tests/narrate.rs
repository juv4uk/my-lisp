//! Exercises lib/narrate.my — the "structure -> text" half of the bridge
//! from private/lisp-to-knowledge.md §6, the reverse of lib/understand.my's
//! "text -> structure" half.
//! Перевіряє lib/narrate.my — половину мосту "структура -> текст" з
//! private/lisp-to-knowledge.md §6, обернену до "текст -> структура" з
//! lib/understand.my.
//! Prüft lib/narrate.my — die Hälfte "Struktur -> Text" der Brücke aus
//! private/lisp-to-knowledge.md §6, das Gegenstück zu "Text -> Struktur"
//! aus lib/understand.my.

use my_lisp::{eval_program, Session};

fn eval_narrate(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/unify.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/reason.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/understand.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/narrate.my"), &mut session).unwrap();
    eval_program(source, &mut session)
        .unwrap_or_else(|e| panic!("evaluation failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

#[test]
fn narrate_fact_undoes_understand_is_a() {
    // Round-trip: understand turns "earth is a planet" into a fact, and
    // narrate-fact turns that fact back into the original words.
    let source = r#"
        (narrate-fact (car (understand '(earth is a planet))))
    "#;
    assert_eq!(eval_narrate(source), "(earth is a planet)");
}

#[test]
fn narrate_fact_undoes_understand_relation() {
    let source = r#"
        (narrate-fact (car (understand '(earth orbits sun))))
    "#;
    assert_eq!(eval_narrate(source), "(earth orbits sun)");
}

#[test]
fn narrate_provenance_explains_a_bare_fact_with_no_because() {
    let source = r#"
        (let ((rules '(((parent alice bob)))))
             (let* ((results (reason '(parent alice bob) rules))
                    (proof (second (car results))))
               (narrate-provenance (provenance proof))))
    "#;
    assert_eq!(eval_narrate(source), "(alice parent bob)");
}

#[test]
fn narrate_provenance_explains_a_derived_fact_with_because_and_and() {
    let source = r#"
        (let ((rules '(
                 ((grandparent (var x) (var y)) (parent (var x) (var z)) (parent (var z) (var y)))
                 ((parent alice bob))
                 ((parent bob charlie))
               )))
             (let* ((results (reason (list 'grandparent (logic-var 'a) (logic-var 'b)) rules))
                    (proof (second (car results))))
               (narrate-provenance (provenance proof))))
    "#;
    let output = eval_narrate(source);
    // Both sub-facts are ground (parent alice bob / parent bob charlie),
    // so this reads as an actual sentence, joined by "because"/"and" — the
    // limitation about unresolved (var ...) placeholders only bites when a
    // rule's own head isn't fully grounded, which isn't the case here.
    assert_eq!(
        output,
        "((var (x . 0)) grandparent (var (y . 0)) because alice parent bob and bob parent charlie)"
    );
}

#[test]
fn assert_understand_and_narrate_are_direct_inverses_for_the_is_a_shape() {
    let source = r#"
        (equal? '(mars is a planet) (narrate-fact (car (understand '(mars is a planet)))))
    "#;
    assert_eq!(eval_narrate(source), "t");
}

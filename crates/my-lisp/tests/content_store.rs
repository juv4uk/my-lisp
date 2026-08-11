use my_lisp::{eval_program, Session};

fn eval_store(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/unify.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/reason.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/forward.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/knowledge.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/persistent-map.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/world.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/content-store.my"), &mut session).unwrap();
    eval_program(source, &mut session)
        .unwrap()
        .value
        .to_string()
}

#[test]
fn stored_knowledge_is_retrievable_by_its_canonical_address() {
    assert_eq!(
        eval_store(
            r#"
            (let ((knowledge '((planet earth))))
              (let ((store (content-store-put (empty-content-store) knowledge)))
                (content-store-get store (knowledge-content-address knowledge))))
            "#
        ),
        "(((planet earth)))"
    );
}

#[test]
fn inserting_equal_content_twice_does_not_grow_the_store() {
    assert_eq!(
        eval_store(
            r#"
            (let ((knowledge '((planet earth))))
              (let ((once (content-store-put (empty-content-store) knowledge)))
                (let ((twice (content-store-put once knowledge)))
                  (list (content-store-size once)
                        (content-store-size twice)))))
            "#
        ),
        "(1 1)"
    );
}

#[test]
fn different_content_occupies_different_addresses() {
    assert_eq!(
        eval_store(
            r#"
            (let ((earth '((planet earth)))
                  (mars '((planet mars))))
              (let ((store (content-store-put
                             (content-store-put (empty-content-store) earth)
                             mars)))
                (list (content-store-size store)
                      (content-store-contains?
                        store (knowledge-content-address earth))
                      (content-store-contains?
                        store (knowledge-content-address mars)))))
            "#
        ),
        "(2 t t)"
    );
}

#[test]
fn reconstructed_equal_worlds_deduplicate_in_the_store() {
    assert_eq!(
        eval_store(
            r#"
            (let ((source
                    (world-tell (empty-world) 'zoo '((has-fur cat)))))
              (let ((copy
                      (second
                        (import-knowledge-package-world
                          (empty-world)
                          (make-world-knowledge-package source 'zoo)))))
                (let ((store
                        (content-store-put-world
                          (content-store-put-world (empty-content-store) source)
                          copy)))
                  (content-store-size store))))
            "#
        ),
        "1"
    );
}

#[test]
fn worlds_with_equal_projection_but_different_history_remain_distinct() {
    assert_eq!(
        eval_store(
            r#"
            (let ((direct
                    (world-tell (empty-world) 'zoo '((has-fur cat)))))
              (let ((retold
                      (world-tell
                        (world-retract
                          (world-tell (empty-world) 'zoo '((has-fur cat)))
                          'zoo '((has-fur cat)))
                        'zoo '((has-fur cat)))))
                (let ((store
                        (content-store-put-world
                          (content-store-put-world (empty-content-store) direct)
                          retold)))
                  (list (equal? (world-clauses direct 'zoo)
                                (world-clauses retold 'zoo))
                        (content-store-size store)))))
            "#
        ),
        "(t 2)"
    );
}

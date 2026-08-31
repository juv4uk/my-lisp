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
    eval_program(include_str!("../../../lib/lisp-fs.my"), &mut session).unwrap();
    eval_program(source, &mut session)
        .unwrap()
        .value
        .to_string()
}

#[test]
fn lisp_fs_keeps_old_snapshot_and_reads_new_bindings() {
    assert_eq!(
        eval_store(
            r#"
            (let ((empty (fs-empty)))
              (let ((first-write (fs-write empty "notes/today" (quote (hello world)))))
                (let ((old (car first-write))
                      (second-write (fs-write (car first-write)
                                              "notes/tomorrow"
                                              (quote (hello again)))))
                  (let ((new (car second-write)))
                    (list
                      (fs-read old "notes/today")
                      (fs-read old "notes/tomorrow")
                      (fs-read new "notes/today")
                      (fs-read new "notes/tomorrow")
                      (fs-revision old)
                      (fs-revision new))))))
            "#
        ),
        "((found (hello world) \"(hello world)\") (not-found \"notes/tomorrow\") (found (hello world) \"(hello world)\") (found (hello again) \"(hello again)\") 1 2)"
    );
}

#[test]
fn lisp_fs_deduplicates_equal_objects_but_keeps_explicit_missing_status() {
    assert_eq!(
        eval_store(
            r#"
            (let ((empty (fs-empty)))
              (let ((a (fs-write empty "a" (quote ()))))
                (let ((b (fs-write (car a) "b" (quote ()))))
                  (list
                    (content-store-size (fs-objects (car b)))
                    (fs-contains? (car b) "a")
                    (fs-read (car b) "missing")))))
            "#
        ),
        "(1 t (not-found \"missing\"))"
    );
}

#[test]
fn lisp_fs_conformance_fixture_is_deterministic() {
    assert_eq!(
        eval_store(include_str!("../../../tests/fixtures/lisp-fs-conformance.my")),
        "((found (hello world) \"(hello world)\") (not-found \"notes/empty\") (found () \"()\") (not-found \"missing\") 2 1 2)"
    );
}

#[test]
fn lisp_fs_overwrite_is_a_new_revision_and_old_value_remains_in_old_root() {
    assert_eq!(
        eval_store(
            r#"
            (let ((empty (fs-empty)))
              (let ((a (fs-write empty "same" (quote old))))
                (let ((b (fs-write (car a) "same" (quote new))))
                  (list
                    (fs-read (car a) "same")
                    (fs-read (car b) "same")
                    (fs-revision (car a))
                    (fs-revision (car b))))))
            "#
        ),
        "((found old \"old\") (found new \"new\") 1 2)"
    );
}

#[test]
fn lisp_fs_empty_name_is_literal_and_unknown_object_address_is_not_found() {
    assert_eq!(
        eval_store(
            r#"
            (let ((empty (fs-empty)))
              (let ((written (fs-write empty "" (quote value))))
                (let ((malformed
                        (list (fs-objects (car written))
                              (map-insert "ghost" "missing-address" map-empty)
                              9)))
                  (list
                    (fs-read (car written) "")
                    (fs-read malformed "ghost")
                    (fs-read malformed "absent")))))
            "#
        ),
        "((found value \"value\") (not-found \"ghost\") (not-found \"absent\"))"
    );
}

#[test]
fn lisp_fs_object_envelope_round_trips_and_rejects_bad_metadata() {
    assert_eq!(
        eval_store(
            r#"
            (let ((encoded (fs-serialize-object (quote (hello world)))))
              (list
                encoded
                (fs-deserialize-object encoded)
                (fs-object-package-decision
                  (list
                    (cons (quote format) (quote wsm-fs-object))
                    (cons (quote version) (quote (9 9)))
                    (cons (quote address) "wrong")
                    (cons (quote value) (quote (hello world)))))
                (fs-object-package-decision
                  (list
                    (cons (quote format) (quote wsm-fs-object))
                    (cons (quote version) (quote (0 1)))
                    (cons (quote address) "wrong")
                    (cons (quote value) (quote (hello world)))))))
            "#
        ),
        "(\"((format . wsm-fs-object) (version 0 1) (address . \\\"(hello world)\\\") (value hello world))\" (accepted (hello world)) (rejected unsupported-version) (rejected address-mismatch))"
    );
}

#[test]
fn lisp_fs_root_envelope_round_trips_bindings_revision_and_addresses() {
    assert_eq!(
        eval_store(
            r#"
            (let* ((fs (car (fs-write (fs-empty) "notes/today" (quote (hello world)))))
                   (encoded (fs-serialize-root fs))
                   (bad
                     (list
                       (cons (quote format) (quote wsm-fs-root))
                       (cons (quote version) (quote (9 9)))
                       (cons (quote revision) 1)
                       (cons (quote bindings) (quote ()))
                       (cons (quote objects) (quote ())))))
              (list encoded
                    (car (fs-root-package-decision (read encoded)))
                    (car (fs-root-package-decision bad))))
            "#
        ),
        "(\"((format . wsm-fs-root) (version 0 1) (revision . 1) (bindings (\\\"notes/today\\\" . \\\"(hello world)\\\")) (objects \\\"(hello world)\\\"))\" accepted rejected)"
    );
}

#[test]
fn stored_knowledge_is_retrievable_by_its_canonical_address() {
    assert_eq!(
        eval_store(
            r#"
            (let ((knowledge (quote ((planet earth)))))
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
            (let ((knowledge (quote ((planet earth)))))
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
            (let ((earth (quote ((planet earth))))
                  (mars (quote ((planet mars)))))
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
                    (world-tell (empty-world) (quote zoo) (quote ((has-fur cat))))))
              (let ((copy
                      (second
                        (import-knowledge-package-world
                          (empty-world)
                          (make-world-knowledge-package source (quote zoo))))))
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
                    (world-tell (empty-world) (quote zoo) (quote ((has-fur cat))))))
              (let ((retold
                      (world-tell
                        (world-retract
                          (world-tell (empty-world) (quote zoo) (quote ((has-fur cat))))
                          (quote zoo) (quote ((has-fur cat))))
                        (quote zoo) (quote ((has-fur cat))))))
                (let ((store
                        (content-store-put-world
                          (content-store-put-world (empty-content-store) direct)
                          retold)))
                  (list (equal? (world-clauses direct (quote zoo))
                                (world-clauses retold (quote zoo)))
                        (content-store-size store)))))
            "#
        ),
        "(t 2)"
    );
}

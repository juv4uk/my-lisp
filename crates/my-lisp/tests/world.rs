use my_lisp::{eval_program, Session};

fn eval_world(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/unify.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/reason.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/forward.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/knowledge.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/world.my"), &mut session).unwrap();
    eval_program(source, &mut session)
        .unwrap()
        .value
        .to_string()
}

#[test]
fn empty_world_is_an_ordinary_first_class_value() {
    assert_eq!(eval_world("(world? (empty-world))"), "t");
    assert_eq!(eval_world("(world? '(not-a-world))"), "()");
}

#[test]
fn tell_returns_a_new_world_without_changing_the_old_one() {
    assert_eq!(
        eval_world(
            r#"
            (let ((before (empty-world)))
              (let ((after (world-tell before 'zoo '((has-fur cat)))))
                (list (world-clauses before 'zoo)
                      (world-clauses after 'zoo))))
            "#
        ),
        "(() (((has-fur cat))))"
    );
}

#[test]
fn each_world_keeps_its_immediate_parent() {
    assert_eq!(
        eval_world(
            r#"
            (let ((before (empty-world)))
              (let ((after (world-tell before 'zoo '((has-fur cat)))))
                (equal? before (world-parent after))))
            "#
        ),
        "t"
    );
}

#[test]
fn later_versions_preserve_every_earlier_snapshot() {
    assert_eq!(
        eval_world(
            r#"
            (let ((w0 (empty-world)))
              (let ((w1 (world-tell w0 'zoo '((has-fur cat)))))
                (let ((w2 (world-tell w1 'zoo '((has-fur dog)))))
                  (list (world-clauses w0 'zoo)
                        (world-clauses w1 'zoo)
                        (world-clauses w2 'zoo)))))
            "#
        ),
        "(() (((has-fur cat))) (((has-fur dog)) ((has-fur cat))))"
    );
}

#[test]
fn defmodule_compatibility_wrapper_uses_the_world_transition() {
    assert_eq!(
        eval_world(
            r#"
            (let ((clauses '(((planet earth)) ((star sun)))))
              (let ((expected
                      (world-journal
                        (world-tell-all
                          (make-world '() *knowledge-journal* '())
                          'space
                          clauses))))
                ((lambda ()
                   (defmodule space clauses)
                   (equal? *knowledge-journal* expected)))))
            "#
        ),
        "t"
    );
}

#[test]
fn defmodule_after_world_load_keeps_legacy_reason_in_behavior() {
    assert_eq!(
        eval_world(
            r#"
            (defmodule space '(((planet earth))))
            (reason-in 'space '(planet earth))
            "#
        ),
        "((() (proved (planet earth) (planet earth) ())))"
    );
}

#[test]
fn repeated_compatible_defmodule_calls_still_accumulate() {
    assert_eq!(
        eval_world(
            r#"
            (defmodule space '(((planet earth))))
            (defmodule space '(((planet mars))))
            (module-clauses-now 'space)
            "#
        ),
        "(((planet mars)) ((planet earth)))"
    );
}

#[test]
fn tell_knowledge_compatibility_wrapper_uses_the_world_transition() {
    assert_eq!(
        eval_world(
            r#"
            (defmodule space '(((planet earth))))
            (def clauses '(((planet mars))))
            (def expected-journal
              (world-journal
                (world-tell-all
                  (make-world '() *knowledge-journal* '())
                  'space
                  clauses)))
            (tell-knowledge space clauses)
            (equal? *knowledge-journal* expected-journal)
            "#
        ),
        "t"
    );
}

#[test]
fn conflicting_tell_knowledge_keeps_the_legacy_journal_unchanged() {
    assert_eq!(
        eval_world(
            r#"
            (defmodule space '(((not (planet earth)))))
            (let ((before *knowledge-journal*))
              (list (tell-knowledge space '(((planet earth))))
                    (equal? before *knowledge-journal*)))
            "#
        ),
        "(Conflict-detected t)"
    );
}

#[test]
fn retract_knowledge_compatibility_wrapper_uses_the_world_transition() {
    assert_eq!(
        eval_world(
            r#"
            (defmodule space '(((planet earth))))
            (def expected-journal
              (world-journal
                (world-retract
                  (make-world '() *knowledge-journal* '())
                  'space
                  '((planet earth)))))
            (retract-knowledge space '((planet earth)))
            (list (equal? *knowledge-journal* expected-journal)
                  (reason-in 'space '(planet earth)))
            "#
        ),
        "(t ())"
    );
}

#[test]
fn advise_compatibility_wrapper_commits_only_the_accepted_world() {
    assert_eq!(
        eval_world(
            r#"
            (list (advise space '((planet earth)))
                  (reason-in 'space '(planet earth)))
            "#
        ),
        "((accepted (module space) (knowledge ((planet earth)))) ((() (proved (planet earth) (planet earth) ()))))"
    );
}

#[test]
fn advise_compatibility_wrapper_preserves_journal_on_conflict() {
    assert_eq!(
        eval_world(
            r#"
            (defmodule space '(((not (planet earth)))))
            (def before *knowledge-journal*)
            (def decision (advise space '((planet earth))))
            (list (car decision) (equal? before *knowledge-journal*))
            "#
        ),
        "(conflict t)"
    );
}

#[test]
fn advise_compatibility_argument_is_evaluated_once() {
    assert_eq!(
        eval_world(
            r#"
            (def *evaluation-count* 0)
            (def decision
              (advise space
                (second
                  (list (def *evaluation-count* (+ *evaluation-count* 1))
                        '((planet earth))))))
            (list *evaluation-count* (car decision))
            "#
        ),
        "(1 accepted)"
    );
}

#[test]
fn advise_all_compatibility_wrapper_keeps_atomic_world_transition() {
    assert_eq!(
        eval_world(
            r#"
            (def decision
              (advise-all space
                '(((star sun))
                  ((planet earth) (star sun)))))
            (list (car decision)
                  (length (module-clauses-now 'space)))
            "#
        ),
        "(accepted 2)"
    );
}

#[test]
fn advise_all_compatibility_wrapper_rolls_back_invalid_batch() {
    assert_eq!(
        eval_world(
            r#"
            (def before *knowledge-journal*)
            (def decision (advise-all space '(((planet earth)) malformed)))
            (list (car decision) (equal? before *knowledge-journal*))
            "#
        ),
        "(rejected t)"
    );
}

#[test]
fn advise_all_compatibility_argument_is_evaluated_once() {
    assert_eq!(
        eval_world(
            r#"
            (def *evaluation-count* 0)
            (def decision
              (advise-all space
                (second
                  (list (def *evaluation-count* (+ *evaluation-count* 1))
                        '(((planet earth)))))))
            (list *evaluation-count* (car decision))
            "#
        ),
        "(1 accepted)"
    );
}

#[test]
fn package_import_compatibility_wrapper_commits_the_accepted_world() {
    assert_eq!(
        eval_world(
            r#"
            (def package
              (make-knowledge-package 'space '(((planet earth)))))
            (list (car (import-knowledge-package package))
                  (car (reason-in 'space '(planet earth))))
            "#
        ),
        "(accepted (() (proved (planet earth) (planet earth) ())))"
    );
}

#[test]
fn package_import_compatibility_wrapper_preserves_journal_on_rejection() {
    assert_eq!(
        eval_world(
            r#"
            (def before *knowledge-journal*)
            (def package
              '((format . my-lisp-knowledge)
                (version 99 0)
                (module . space)
                (clauses . (((planet earth))))))
            (def decision (import-knowledge-package package))
            (list (car decision) (equal? before *knowledge-journal*))
            "#
        ),
        "(rejected t)"
    );
}

#[test]
fn package_import_compatibility_wrapper_preserves_journal_on_conflict() {
    assert_eq!(
        eval_world(
            r#"
            (defmodule space '(((not (planet earth)))))
            (def before *knowledge-journal*)
            (def package
              (make-knowledge-package 'space '(((planet earth)))))
            (def decision (import-knowledge-package package))
            (list (car decision) (equal? before *knowledge-journal*))
            "#
        ),
        "(conflict t)"
    );
}

#[test]
fn package_import_compatibility_argument_is_evaluated_once() {
    assert_eq!(
        eval_world(
            r#"
            (def *evaluation-count* 0)
            (def decision
              (import-knowledge-package
                (second
                  (list (def *evaluation-count* (+ *evaluation-count* 1))
                        (make-knowledge-package
                          'space '(((planet earth))))))))
            (list *evaluation-count* (car decision))
            "#
        ),
        "(1 accepted)"
    );
}

#[test]
fn retract_creates_history_instead_of_erasing_it() {
    assert_eq!(
        eval_world(
            r#"
            (let ((w0 (empty-world)))
              (let ((w1 (world-tell w0 'zoo '((has-fur cat)))))
                (let ((w2 (world-retract w1 'zoo '((has-fur cat)))))
                  (list (world-clauses w1 'zoo)
                        (world-clauses w2 'zoo)
                        (world-module-known? w2 'zoo)))))
            "#
        ),
        "((((has-fur cat))) () t)"
    );
}

#[test]
fn independent_branches_can_grow_from_the_same_world() {
    assert_eq!(
        eval_world(
            r#"
            (let ((root (empty-world)))
              (let ((cats (world-tell root 'zoo '((has-fur cat))))
                    (dogs (world-tell root 'zoo '((has-fur dog)))))
                (list (world-clauses cats 'zoo)
                      (world-clauses dogs 'zoo)
                      (world-clauses root 'zoo))))
            "#
        ),
        "((((has-fur cat))) (((has-fur dog))) ())"
    );
}

#[test]
fn backward_reasoning_reads_the_selected_world_snapshot() {
    assert_eq!(
        eval_world(
            r#"
            (let ((w0 (empty-world)))
              (let ((w1 (world-tell w0 'family '((parent tom bob)))))
                (let ((w2 (world-retract w1 'family '((parent tom bob)))))
                  (list (cond
                          ((atom (reason-in-world w1 'family '(parent tom bob))) 'no)
                          (t 'yes))
                        (cond
                          ((atom (reason-in-world w2 'family '(parent tom bob))) 'no)
                          (t 'yes))))))
            "#
        ),
        "(yes no)"
    );
}

#[test]
fn backward_reasoning_keeps_independent_branches_isolated() {
    assert_eq!(
        eval_world(
            r#"
            (let ((root (empty-world)))
              (let ((cats (world-tell root 'zoo '((likes alice cats))))
                    (dogs (world-tell root 'zoo '((likes alice dogs)))))
                (list (cond
                        ((atom (reason-in-world cats 'zoo '(likes alice cats))) 'no)
                        (t 'yes))
                      (cond
                        ((atom (reason-in-world cats 'zoo '(likes alice dogs))) 'no)
                        (t 'yes))
                      (cond
                        ((atom (reason-in-world dogs 'zoo '(likes alice dogs))) 'no)
                        (t 'yes))
                      (cond
                        ((atom (reason-in-world dogs 'zoo '(likes alice cats))) 'no)
                        (t 'yes)))))
            "#
        ),
        "(yes no yes no)"
    );
}

#[test]
fn forward_reasoning_materializes_only_the_selected_world() {
    assert_eq!(
        eval_world(
            r#"
            (let ((w0 (empty-world)))
              (let ((w1 (world-tell w0 'physics '((has-mass apple)))))
                (let ((w2 (world-tell w1 'physics
                                      '((attracted-by-gravity (var x))
                                        (has-mass (var x))))))
                  (list (forward-in-world w1 'physics)
                        (forward-in-world w2 'physics)))))
            "#
        ),
        "(((has-mass apple)) ((attracted-by-gravity apple) (has-mass apple)))"
    );
}

#[test]
fn world_reasoning_reports_an_unknown_module_without_global_fallback() {
    assert_eq!(
        eval_world("(reason-in-world (empty-world) 'missing '(fact x))"),
        "Module-not-found"
    );
    assert_eq!(
        eval_world("(forward-in-world (empty-world) 'missing)"),
        "Module-not-found"
    );
}

#[test]
fn advise_world_accepts_into_a_new_queryable_world() {
    assert_eq!(
        eval_world(
            r#"
            (let ((before (empty-world)))
              (let ((result (advise-world before 'astronomy '((planet venus)))))
                (let ((after (second result)))
                  (list (car (car result))
                        (world-clauses before 'astronomy)
                        (cond
                          ((atom (reason-in-world after 'astronomy '(planet venus))) 'no)
                          (t 'yes))))))
            "#
        ),
        "(accepted () yes)"
    );
}

#[test]
fn advise_world_rejection_returns_the_unchanged_world() {
    assert_eq!(
        eval_world(
            r#"
            (let ((before (empty-world)))
              (let ((result (advise-world before 'astronomy '(planet venus))))
                (list (car (car result))
                      (equal? before (second result))
                      (world-module-known? (second result) 'astronomy))))
            "#
        ),
        "(rejected t ())"
    );
}

#[test]
fn advise_world_conflict_preserves_the_existing_snapshot() {
    assert_eq!(
        eval_world(
            r#"
            (let ((w1 (world-tell (empty-world)
                                  'astronomy
                                  '((not (planet pluto))))))
              (let ((result (advise-world w1 'astronomy '((planet pluto)))))
                (list (car (car result))
                      (equal? w1 (second result))
                      (world-clauses (second result) 'astronomy))))
            "#
        ),
        "(conflict t (((not (planet pluto)))))"
    );
}

#[test]
fn advise_world_does_not_read_the_global_knowledge_journal() {
    assert_eq!(
        eval_world(
            r#"
            (advise astronomy '((not (planet mars))))
            (let ((result (advise-world (empty-world)
                                        'astronomy
                                        '((planet mars)))))
              (list (car (car result))
                    (world-clauses (second result) 'astronomy)))
            "#
        ),
        "(accepted (((planet mars))))"
    );
}

#[test]
fn advise_all_world_accepts_one_atomic_dependent_batch() {
    assert_eq!(
        eval_world(
            r#"
            (let ((before (empty-world)))
              (let ((result
                      (advise-all-world
                        before
                        'astronomy
                        '(((planet earth))
                          ((has-mass (var x)) (planet (var x)))))))
                (let ((after (second result)))
                  (list (car (car result))
                        (world-clauses before 'astronomy)
                        (cond
                          ((atom (reason-in-world after 'astronomy
                                                 '(has-mass earth))) 'no)
                          (t 'yes))
                        (equal? before (world-parent after))))))
            "#
        ),
        "(accepted () yes t)"
    );
}

#[test]
fn advise_all_world_rejects_the_whole_malformed_batch() {
    assert_eq!(
        eval_world(
            r#"
            (let ((before (empty-world)))
              (let ((result
                      (advise-all-world before 'astronomy
                                        '(((planet earth)) (planet mars)))))
                (list (car (car result))
                      (equal? before (second result))
                      (world-module-known? (second result) 'astronomy))))
            "#
        ),
        "(rejected t ())"
    );
}

#[test]
fn advise_all_world_rejects_an_empty_batch_without_a_new_world() {
    assert_eq!(
        eval_world(
            r#"
            (let ((before (empty-world)))
              (let ((result (advise-all-world before 'astronomy '())))
                (list (car (car result))
                      (second (second (car result)))
                      (equal? before (second result)))))
            "#
        ),
        "(rejected invalid-batch t)"
    );
}

#[test]
fn advise_all_world_detects_internal_conflict_without_partial_writes() {
    assert_eq!(
        eval_world(
            r#"
            (let ((before (empty-world)))
              (let ((result
                      (advise-all-world
                        before
                        'astronomy
                        '(((planet pluto)) ((not (planet pluto)))))))
                (list (car (car result))
                      (equal? before (second result))
                      (world-module-known? (second result) 'astronomy))))
            "#
        ),
        "(conflict t ())"
    );
}

#[test]
fn advise_all_world_ignores_conflicts_in_the_global_journal() {
    assert_eq!(
        eval_world(
            r#"
            (advise astronomy '((not (planet mars))))
            (let ((result
                    (advise-all-world (empty-world)
                                      'astronomy
                                      '(((planet mars))))))
              (list (car (car result))
                    (world-clauses (second result) 'astronomy)))
            "#
        ),
        "(accepted (((planet mars))))"
    );
}

#[test]
fn world_package_export_reads_the_selected_snapshot_only() {
    assert_eq!(
        eval_world(
            r#"
            (let ((w1 (world-tell (empty-world) 'astronomy '((planet earth)))))
              (let ((w2 (world-tell w1 'astronomy '((planet mars)))))
                (list (knowledge-package-field
                        'clauses (make-world-knowledge-package w1 'astronomy))
                      (knowledge-package-field
                        'clauses (make-world-knowledge-package w2 'astronomy)))))
            "#
        ),
        "((((planet earth))) (((planet mars)) ((planet earth))))"
    );
}

#[test]
fn world_package_import_atomically_creates_a_queryable_child() {
    assert_eq!(
        eval_world(
            r#"
            (let ((before (empty-world)))
              (let ((package
                      (make-knowledge-package
                        'astronomy
                        '(((planet earth))
                          ((has-mass (var x)) (planet (var x)))))))
                (let ((result (import-knowledge-package-world before package)))
                  (let ((after (second result)))
                    (list (car (car result))
                          (equal? before (world-parent after))
                          (cond
                            ((atom (reason-in-world after 'astronomy
                                                   '(has-mass earth))) 'no)
                            (t 'yes)))))))
            "#
        ),
        "(accepted t yes)"
    );
}

#[test]
fn world_package_import_rejects_unsupported_versions_without_transition() {
    assert_eq!(
        eval_world(
            r#"
            (def before (empty-world))
            (def result
              (import-knowledge-package-world
                before
                '((format . my-lisp-knowledge)
                  (version 1 0)
                  (module . astronomy)
                  (clauses . (((planet earth)))))))
            (list (car (car result))
                  (second (second (car result)))
                  (equal? before (second result)))
            "#
        ),
        "(rejected unsupported-version t)"
    );
}

#[test]
fn world_package_import_conflict_preserves_the_target_snapshot() {
    assert_eq!(
        eval_world(
            r#"
            (let ((before
                    (world-tell (empty-world)
                                'astronomy
                                '((not (planet pluto))))))
              (let ((package
                      (make-knowledge-package 'astronomy
                                              '(((planet pluto))))))
                (let ((result (import-knowledge-package-world before package)))
                  (list (car (car result))
                        (equal? before (second result))
                        (world-clauses (second result) 'astronomy)))))
            "#
        ),
        "(conflict t (((not (planet pluto)))))"
    );
}

#[test]
fn exported_snapshot_can_seed_an_independent_world_branch() {
    assert_eq!(
        eval_world(
            r#"
            (let ((source
                    (world-tell (empty-world) 'zoo '((has-fur cat)))))
              (let ((package (make-world-knowledge-package source 'zoo)))
                (let ((target (second
                                (import-knowledge-package-world
                                  (empty-world) package))))
                  (let ((target-grown
                          (world-tell target 'zoo '((has-fur dog)))))
                    (list (world-clauses source 'zoo)
                          (world-clauses target 'zoo)
                          (world-clauses target-grown 'zoo))))))
            "#
        ),
        "((((has-fur cat))) (((has-fur cat))) (((has-fur dog)) ((has-fur cat))))"
    );
}

#[test]
fn world_depth_counts_transitions_from_the_root() {
    assert_eq!(
        eval_world(
            r#"
            (let ((w0 (empty-world)))
              (let ((w1 (world-tell w0 'zoo '((has-fur cat)))))
                (let ((w2 (world-retract w1 'zoo '((has-fur cat)))))
                  (list (world-depth w0)
                        (world-depth w1)
                        (world-depth w2)))))
            "#
        ),
        "(0 1 2)"
    );
}

#[test]
fn world_at_depth_recovers_an_exact_historical_snapshot() {
    assert_eq!(
        eval_world(
            r#"
            (let ((w0 (empty-world)))
              (let ((w1 (world-tell w0 'zoo '((has-fur cat)))))
                (let ((w2 (world-tell w1 'zoo '((has-fur dog)))))
                  (list (equal? w0 (world-at-depth w2 0))
                        (equal? w1 (world-at-depth w2 1))
                        (equal? w2 (world-at-depth w2 2))))))
            "#
        ),
        "(t t t)"
    );
}

#[test]
fn world_at_depth_rejects_depths_outside_the_history() {
    assert_eq!(
        eval_world("(list (world-at-depth (empty-world) -1) (world-at-depth (empty-world) 1))"),
        "(World-not-found World-not-found)"
    );
}

#[test]
fn world_diff_returns_chronological_events_across_atomic_transitions() {
    assert_eq!(
        eval_world(
            r#"
            (let ((w0 (empty-world)))
              (let ((w1
                      (world-tell-all
                        w0 'zoo
                        '(((has-fur cat)) ((has-fur dog))))))
                (let ((w2 (world-retract w1 'zoo '((has-fur cat)))))
                  (world-diff w0 w2))))
            "#
        ),
        "((tell zoo ((has-fur cat))) (tell zoo ((has-fur dog))) (retract zoo ((has-fur cat))))"
    );
}

#[test]
fn world_diff_refuses_to_invent_a_path_between_sibling_branches() {
    assert_eq!(
        eval_world(
            r#"
            (let ((root (empty-world)))
              (let ((cats (world-tell root 'zoo '((has-fur cat))))
                    (dogs (world-tell root 'zoo '((has-fur dog)))))
                (world-diff cats dogs)))
            "#
        ),
        "World-not-ancestor"
    );
}

#[test]
fn world_common_ancestor_finds_the_branch_point() {
    assert_eq!(
        eval_world(
            r#"
            (let ((root (empty-world)))
              (let ((base (world-tell root 'zoo '((animal cat)))))
                (let ((left (world-tell base 'zoo '((has-fur cat))))
                      (right (world-tell base 'zoo '((has-tail cat)))))
                  (equal? base (world-common-ancestor left right)))))
            "#
        ),
        "t"
    );
}

#[test]
fn world_common_ancestor_aligns_unequal_branch_depths() {
    assert_eq!(
        eval_world(
            r#"
            (let ((root (empty-world)))
              (let ((base (world-tell root 'zoo '((animal cat)))))
                (let ((left1 (world-tell base 'zoo '((has-fur cat))))
                      (right (world-tell base 'zoo '((has-tail cat)))))
                  (let ((left2 (world-tell left1 'zoo '((likes cat milk)))))
                    (equal? base (world-common-ancestor left2 right))))))
            "#
        ),
        "t"
    );
}

#[test]
fn world_branch_diff_reports_both_chronological_deltas() {
    assert_eq!(
        eval_world(
            r#"
            (let ((base (world-tell (empty-world) 'zoo '((animal cat)))))
              (let ((left (world-tell base 'zoo '((has-fur cat))))
                    (right (world-tell base 'zoo '((has-tail cat)))))
                (let ((comparison (world-branch-diff left right)))
                  (list (second (second comparison))
                        (second (third comparison))))))
            "#
        ),
        "(((tell zoo ((has-fur cat)))) ((tell zoo ((has-tail cat)))))"
    );
}

#[test]
fn reconstructed_equal_worlds_have_no_branch_delta() {
    assert_eq!(
        eval_world(
            r#"
            (let ((source
                    (world-tell (empty-world) 'zoo '((has-fur cat)))))
              (let ((copy
                      (second
                        (import-knowledge-package-world
                          (empty-world)
                          (make-world-knowledge-package source 'zoo)))))
                (let ((comparison (world-branch-diff source copy)))
                  (list (second (second comparison))
                        (second (third comparison))))))
            "#
        ),
        "(() ())"
    );
}

#[test]
fn equal_knowledge_has_the_same_canonical_content_address() {
    assert_eq!(
        eval_world(
            r#"
            (eq (knowledge-content-address '((planet earth)))
                (knowledge-content-address '((planet earth))))
            "#
        ),
        "t"
    );
}

#[test]
fn different_knowledge_has_a_different_content_address() {
    assert_eq!(
        eval_world(
            r#"
            (eq (knowledge-content-address '((planet earth)))
                (knowledge-content-address '((planet mars))))
            "#
        ),
        "()"
    );
}

#[test]
fn knowledge_content_addresses_round_trip_to_the_same_structure() {
    assert_eq!(
        eval_world(
            r#"
            (let ((knowledge
                    '((has-mass (var x)) (planet (var x)))))
              (equal? knowledge
                      (read (knowledge-content-address knowledge))))
            "#
        ),
        "t"
    );
}

#[test]
fn independently_reconstructed_worlds_have_the_same_content_address() {
    assert_eq!(
        eval_world(
            r#"
            (let ((source
                    (world-tell (empty-world) 'zoo '((has-fur cat)))))
              (let ((copy
                      (second
                        (import-knowledge-package-world
                          (empty-world)
                          (make-world-knowledge-package source 'zoo)))))
                (eq (world-content-address source)
                    (world-content-address copy))))
            "#
        ),
        "t"
    );
}

#[test]
fn equal_current_clauses_do_not_erase_distinct_world_histories() {
    assert_eq!(
        eval_world(
            r#"
            (let ((direct
                    (world-tell (empty-world) 'zoo '((has-fur cat)))))
              (let ((told
                      (world-tell (empty-world) 'zoo '((has-fur cat)))))
                (let ((retracted
                        (world-retract told 'zoo '((has-fur cat)))))
                  (let ((retold
                          (world-tell retracted 'zoo '((has-fur cat)))))
                    (list (equal? (world-clauses direct 'zoo)
                                  (world-clauses retold 'zoo))
                          (eq (world-content-address direct)
                              (world-content-address retold)))))))
            "#
        ),
        "(t ())"
    );
}

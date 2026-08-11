use my_lisp::{eval_program, Session};

fn eval_world(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/unify.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/reason.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/forward.my"), &mut session).unwrap();
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

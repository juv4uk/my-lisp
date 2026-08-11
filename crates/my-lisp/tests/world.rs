use my_lisp::{eval_program, Session};

fn eval_world(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
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

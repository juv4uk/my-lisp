use my_lisp::{eval_program, Session};

fn session() -> Session {
    let mut session = Session::default();
    for library in [
        include_str!("../../../lib/core.my"),
        include_str!("../../../lib/unify.my"),
        include_str!("../../../lib/reason.my"),
    ] {
        eval_program(library, &mut session).expect("reason index fixture should load");
    }
    session
}

fn eval(session: &mut Session, source: &str) -> String {
    eval_program(source, session)
        .unwrap_or_else(|e| panic!("evaluation failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

#[test]
fn indexed_bucket_preserves_source_rule_order_exactly() {
    let mut s = session();
    let source = r#"
      (def rules
        (quote
          (((p first))
           ((q irrelevant))
           ((p second))
           ((r irrelevant))
           ((p third)))))
      (reason-index-candidates (quote (p target)) (reason-make-index rules))
    "#;
    assert_eq!(
        eval(&mut s, source),
        "(((p first)) ((p second)) ((p third)))"
    );
}

#[test]
fn indexed_reason_is_structurally_identical_to_forced_linear_reason() {
    let mut s = session();
    let source = r#"
      (def rules
        (quote
          (((seed a))
           ((noise one))
           ((path (var x) left) (seed (var x)))
           ((noise two))
           ((path (var x) right) (seed (var x)))
           ((reachable (var x)) (path (var x) (var side))))))
      (def indexed (reason (quote (reachable a)) rules))
      (def linear
        (prove-goal
          (quote (reachable a))
          rules
          (quote ())
          (reason-index-linear rules)
          0))
      (equal? indexed linear)
    "#;
    assert_eq!(eval(&mut s, source), "t");
}

#[test]
fn public_reason_accepts_prebuilt_index_as_an_immutable_snapshot() {
    let mut s = session();
    let source = r#"
      (def old-rules
        (quote
          (((seed a))
           ((reachable (var x)) (seed (var x))))))
      (def prepared (reason-make-index old-rules))
      (def newer-rules
        (append old-rules (quote (((later yes))))))
      (list
        (equal?
          (reason (quote (reachable a)) old-rules)
          (reason (quote (reachable a)) prepared))
        (length (reason (quote (later yes)) prepared))
        (length (reason (quote (later yes)) newer-rules)))
    "#;
    assert_eq!(eval(&mut s, source), "(t 0 1)");
}

#[test]
fn variable_goal_falls_back_to_every_rule() {
    let mut s = session();
    let source = r#"
      (def rules (quote (((p one)) ((q two)) ((r three)))))
      (length
        (reason-index-candidates
          (quote (var x))
          (reason-make-index rules)))
    "#;
    assert_eq!(eval(&mut s, source), "3");
}

#[test]
fn variable_rule_head_disables_index_instead_of_changing_unification() {
    let mut s = session();
    let source = r#"
      (def rules
        (quote
          (((p one))
           ((var whole-head))
           ((q two)))))
      (reason-index-mode (reason-make-index rules))
    "#;
    assert_eq!(eval(&mut s, source), "linear");
}

#[test]
fn too_many_distinct_predicates_fall_back_to_linear_mode() {
    let mut s = session();
    let mut source = String::from("(reason-index-mode (reason-make-index (quote (");
    for i in 0..65 {
        source.push_str(&format!("((pred{i} value))"));
    }
    source.push_str("))))");
    assert_eq!(eval(&mut s, &source), "linear");
}

#[test]
fn indexed_negation_and_recursion_match_forced_linear_results() {
    let mut s = session();
    let source = r#"
      (def rules
        (quote
          (((parent alice bob))
           ((parent bob carol))
           ((ancestor (var x) (var y)) (parent (var x) (var y)))
           ((ancestor (var x) (var y))
             (parent (var x) (var z))
             (ancestor (var z) (var y)))
           ((safe (var x)) (not (blocked (var x))))
           ((noise irrelevant)))))
      (def goal (quote (ancestor alice carol)))
      (def indexed (reason goal rules))
      (def linear
        (prove-goal goal rules (quote ()) (reason-index-linear rules) 0))
      (list
        (equal? indexed linear)
        (equal?
          (reason (quote (safe alice)) rules)
          (prove-goal
            (quote (safe alice))
            rules
            (quote ())
            (reason-index-linear rules)
            0)))
    "#;
    assert_eq!(eval(&mut s, source), "(t t)");
}

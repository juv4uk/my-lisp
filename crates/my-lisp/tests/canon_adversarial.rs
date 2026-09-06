use my_lisp::{eval_program, ErrorKind, Session};

fn eval(source: &str) -> String {
    let mut session = Session::default();
    eval_program(source, &mut session)
        .expect("adversarial canonical program should evaluate")
        .value
        .to_string()
}

#[test]
fn canon_zero_is_atomic_but_not_a_pair() {
    assert_eq!(eval("(атом? ())"), "t");
    assert_eq!(eval("(aṇu ())"), "t");
}

#[test]
fn quote_surface_suppresses_evaluation_of_unknown_code() {
    assert_eq!(
        eval("(як-є (цієї-функції-не-існує 1 2))"),
        "(цієї-функції-не-існує 1 2)"
    );
    assert_eq!(
        eval("(svarūpa (ayam-na-vidyate 1 2))"),
        "(ayam-na-vidyate 1 2)"
    );
}

#[test]
fn cond_surface_stops_at_the_first_true_clause() {
    assert_eq!(
        eval("(за-умовою (t (як-є перша)) ((цієї-функції-не-існує) (як-є друга)))"),
        "перша"
    );
    assert_eq!(
        eval("(anukrama (t (svarūpa prathama)) ((ayam-na-vidyate) (svarūpa dvitīya)))"),
        "prathama"
    );
}

#[test]
fn rest_passes_the_triple_cons_structure_test() {
    assert_eq!(eval("(решта (як-є (кіт . 42)))"), "42");
    assert_eq!(eval("(решта (як-є (1 2 3)))"), "(2 3)");
    assert_eq!(eval("(решта (як-є (1 2 . 3)))"), "(2 . 3)");

    assert_eq!(eval("(śeṣa (svarūpa (phalam . 42)))"), "42");
    assert_eq!(eval("(śeṣa (svarūpa (1 2 3)))"), "(2 3)");
    assert_eq!(eval("(śeṣa (svarūpa (1 2 . 3)))"), "(2 . 3)");
}

#[test]
fn car_and_cdr_on_canon_zero_fail_named_not_panic() {
    for source in ["(перше ())", "(решта ())", "(ādi ())", "(śeṣa ())"] {
        let mut session = Session::default();
        let error = eval_program(source, &mut session).expect_err("projection on () must fail");
        assert_eq!(error.kind, ErrorKind::Type, "source: {source}");
    }
}

#[test]
fn historical_shadowing_must_not_retarget_canonical_surface() {
    // Canonical spellings denote the same primitive meaning, not a late-bound
    // alias to whatever the historical spelling happens to name now.
    assert_eq!(
        eval("(def car (lambda (x) (як-є зламано))) (перше (сполучити 1 2))"),
        "1"
    );
    assert_eq!(
        eval("(def cdr (lambda (x) (svarūpa broken))) (śeṣa (saṃyuj 1 2))"),
        "2"
    );
}

#[test]
fn canonical_builtin_shadowing_is_local_to_that_surface_name() {
    assert_eq!(
        eval("(def перше (lambda (x) (як-є локально))) (перше 42)"),
        "локально"
    );
    assert_eq!(
        eval("(def ādi (lambda (x) (svarūpa sthānika))) (ādi 42)"),
        "sthānika"
    );
}

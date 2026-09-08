use my_lisp::{eval_program, ErrorKind, Session};

fn eval(source: &str) -> String {
    let mut session = Session::default();
    eval_program(source, &mut session)
        .expect("canonical surface program should evaluate")
        .value
        .to_string()
}

#[test]
fn canon_zero_is_the_empty_list_itself() {
    assert_eq!(eval("()"), "()");
}

#[test]
fn ukrainian_surface_executes_all_seven_canonical_operations() {
    assert_eq!(eval("(як-є кіт)"), "кіт");
    assert_eq!(eval("(атом? (як-є кіт))"), "t");
    assert_eq!(eval("(тотожне? (як-є кіт) (як-є кіт))"), "t");
    assert_eq!(eval("(сполучити (як-є кіт) 42)"), "(кіт . 42)");
    assert_eq!(eval("(перше (сполучити 10 20))"), "10");
    assert_eq!(eval("(решта (як-є (1 2 3)))"), "(2 3)");
    assert_eq!(
        eval("(за-умовою (() (як-є ні)) (t (як-є так)))"),
        "так"
    );
}

#[test]
fn sanskrit_surface_executes_all_seven_canonical_operations() {
    assert_eq!(eval("(svarūpa phalam)"), "phalam");
    assert_eq!(eval("(aṇu (svarūpa phalam))"), "t");
    assert_eq!(eval("(abheda (svarūpa phalam) (svarūpa phalam))"), "t");
    assert_eq!(eval("(saṃyuj (svarūpa phalam) 42)"), "(phalam . 42)");
    assert_eq!(eval("(ādi (saṃyuj 10 20))"), "10");
    assert_eq!(eval("(śeṣa (svarūpa (1 2 3)))"), "(2 3)");
    assert_eq!(
        eval("(anukrama (() (svarūpa na)) (t (svarūpa satyam)))"),
        "satyam"
    );
}

#[test]
fn historical_ukrainian_and_sanskrit_surfaces_are_observationally_equal() {
    let cases = [
        ("(quote (1 2 3))", "(як-є (1 2 3))", "(svarūpa (1 2 3))"),
        ("(atom (quote кіт))", "(атом? (як-є кіт))", "(aṇu (svarūpa кіт))"),
        ("(eq (quote кіт) (quote кіт))", "(тотожне? (як-є кіт) (як-є кіт))", "(abheda (svarūpa кіт) (svarūpa кіт))"),
        ("(cons 1 2)", "(сполучити 1 2)", "(saṃyuj 1 2)"),
        ("(car (cons 1 2))", "(перше (сполучити 1 2))", "(ādi (saṃyuj 1 2))"),
        ("(cdr (quote (1 2 3)))", "(решта (як-є (1 2 3)))", "(śeṣa (svarūpa (1 2 3)))"),
        ("(cond (() (quote ні)) (t (quote так)))", "(за-умовою (() (як-є ні)) (t (як-є так)))", "(anukrama (() (svarūpa na)) (t (svarūpa так)))"),
    ];

    for (historical, ukrainian, sanskrit) in cases {
        let historical_value = eval(historical);
        let ukrainian_value = eval(ukrainian);
        let sanskrit_value = eval(sanskrit);
        assert_eq!(historical_value, ukrainian_value);
        assert_eq!(historical_value, sanskrit_value);
    }
}

#[test]
fn ukrainian_canon_surface_is_reserved_but_noncanon_builtin_is_shadowable() {
    let mut session = Session::default();
    let error = eval_program("(def перше (lambda (x) (як-є затінено)))", &mut session)
        .expect_err("Contract 6.0 must reject shadowing a Ukrainian Canon spelling");
    assert_eq!(error.kind, ErrorKind::InvalidForm);

    let ordinary = eval_program("(def + (lambda (x y) (як-є локально))) (+ 1 2)", &mut session)
        .expect("non-Canon builtins must remain ordinary lexical values");
    assert_eq!(ordinary.value.to_string(), "локально");
}

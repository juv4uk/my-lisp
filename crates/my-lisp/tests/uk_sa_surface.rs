use my_lisp::{eval_program, load_core_library, Session};

/// Load core library + Ukrainian surface, return a fresh session.
fn uk_session() -> Session {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core bootstrap");
    eval_program(include_str!("../../../lib/surface/uk.my"), &mut session)
        .expect("Ukrainian surface should load");
    session
}

/// Load core library + Sanskrit surface, return a fresh session.
fn sa_session() -> Session {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core bootstrap");
    eval_program(include_str!("../../../lib/surface/sa.my"), &mut session)
        .expect("Sanskrit surface should load");
    session
}

// ── Canon 0+7: Ukrainian surface ──────────────────────────────

#[test]
fn uk_quote_returns_form_unevaluated() {
    let mut s = uk_session();
    let r = eval_program("(як-є (+ 1 2))", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "(+ 1 2)");
}

#[test]
fn uk_atom_predicates_correctly() {
    let mut s = uk_session();
    let r = eval_program("(атом? 42)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "t");
    let r2 = eval_program("(атом? (сполучити 1 2))", &mut s).expect("eval");
    assert_eq!(r2.value.to_string(), "()");
}

#[test]
fn uk_eq_compares_identity() {
    let mut s = uk_session();
    let r = eval_program("(тотожне? 'а 'а)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "t");
}

#[test]
fn uk_cons_car_cdr_roundtrip() {
    let mut s = uk_session();
    let r = eval_program("(перше (сполучити 10 20))", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "10");
    let r2 = eval_program("(решта (сполучити 10 20))", &mut s).expect("eval");
    assert_eq!(r2.value.to_string(), "20");
}

#[test]
fn uk_cond_branches_correctly() {
    let mut s = uk_session();
    let r = eval_program(
        r#"(за-умовою
             ((тотожне? 1 1) 'так)
             (інакше 'ні))"#,
        &mut s,
    ).expect("eval");
    assert_eq!(r.value.to_string(), "так");
}

// ── Arithmetic: Ukrainian surface ──────────────────────────────

#[test]
fn uk_addition_works() {
    let mut s = uk_session();
    let r = eval_program("(додати 1 2 3)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "6");
}

#[test]
fn uk_subtraction_works() {
    let mut s = uk_session();
    let r = eval_program("(відняти 10 3)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "7");
}

#[test]
fn uk_multiplication_works() {
    let mut s = uk_session();
    let r = eval_program("(помножити 2 3 4)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "24");
}

#[test]
fn uk_division_works() {
    let mut s = uk_session();
    let r = eval_program("(поділити 6 2)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "3");
}

#[test]
fn uk_abs_works() {
    let mut s = uk_session();
    let r = eval_program("(модуль -5)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "5");
}

#[test]
fn uk_min_max_work() {
    let mut s = uk_session();
    let r = eval_program("(найменше 3 1 2)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "1");
    let r2 = eval_program("(найбільше 3 1 2)", &mut s).expect("eval");
    assert_eq!(r2.value.to_string(), "3");
}

#[test]
fn uk_mod_quotient_work() {
    let mut s = uk_session();
    let r = eval_program("(остача 7 3)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "1");
    let r2 = eval_program("(частка 7 3)", &mut s).expect("eval");
    assert_eq!(r2.value.to_string(), "2");
}

// ── Comparisons: Ukrainian surface ─────────────────────────────

#[test]
fn uk_comparisons_work() {
    let mut s = uk_session();
    assert_eq!(eval_program("(менше? 1 2)", &mut s).unwrap().value.to_string(), "t");
    assert_eq!(eval_program("(більше? 2 1)", &mut s).unwrap().value.to_string(), "t");
    assert_eq!(eval_program("(рівне? 3 3)", &mut s).unwrap().value.to_string(), "t");
    assert_eq!(eval_program("(не-більше? 2 3)", &mut s).unwrap().value.to_string(), "t");
    assert_eq!(eval_program("(не-менше? 3 3)", &mut s).unwrap().value.to_string(), "t");
}

// ── Predicates: Ukrainian surface ──────────────────────────────

#[test]
fn uk_not_works() {
    let mut s = uk_session();
    assert_eq!(eval_program("(не 't)", &mut s).unwrap().value.to_string(), "()");
    assert_eq!(eval_program("(не '())", &mut s).unwrap().value.to_string(), "t");
}

#[test]
fn uk_equal_works() {
    let mut s = uk_session();
    let r = eval_program("(однакові? (список 1 2) (список 1 2))", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "t");
}

#[test]
fn uk_symbol_predicate_works() {
    let mut s = uk_session();
    let r = eval_program("(символ? 'кіт)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "t");
}

#[test]
fn uk_string_predicate_works() {
    let mut s = uk_session();
    let r = eval_program(r#"(текст? "привіт")"#, &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "t");
}

// ── Lists: Ukrainian surface ───────────────────────────────────

#[test]
fn uk_list_length_append_reverse() {
    let mut s = uk_session();
    let r = eval_program("(список 1 2 3)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "(1 2 3)");
    let r2 = eval_program("(довжина (список 1 2 3))", &mut s).expect("eval");
    assert_eq!(r2.value.to_string(), "3");
    let r3 = eval_program("(приєднати (список 1 2) (список 3 4))", &mut s).expect("eval");
    assert_eq!(r3.value.to_string(), "(1 2 3 4)");
    let r4 = eval_program("(зворот (список 1 2 3))", &mut s).expect("eval");
    assert_eq!(r4.value.to_string(), "(3 2 1)");
}

#[test]
fn uk_nth_member_assoc() {
    let mut s = uk_session();
    let r = eval_program("(за-номером (список 10 20 30) 1)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "20");
    let r2 = eval_program("(містить? 2 (список 1 2 3))", &mut s).expect("eval");
    assert_eq!(r2.value.to_string(), "t");
}

#[test]
fn uk_ordinals_work() {
    let mut s = uk_session();
    let r = eval_program("(друге (список 1 2 3 4 5))", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "2");
    let r2 = eval_program("(третє (список 1 2 3 4 5))", &mut s).expect("eval");
    assert_eq!(r2.value.to_string(), "3");
    let r3 = eval_program("(п'яте (список 1 2 3 4 5))", &mut s).expect("eval");
    assert_eq!(r3.value.to_string(), "5");
}

// ── Higher-order: Ukrainian surface ────────────────────────────

#[test]
fn uk_map_works() {
    let mut s = uk_session();
    let r = eval_program(
        "(відобразити (функція (x) (помножити x 2)) (список 1 2 3))",
        &mut s,
    ).expect("eval");
    assert_eq!(r.value.to_string(), "(2 4 6)");
}

#[test]
fn uk_filter_works() {
    let mut s = uk_session();
    let r = eval_program(
        "(відсіяти (функція (x) (менше? x 3)) (список 1 2 3 4))",
        &mut s,
    ).expect("eval");
    assert_eq!(r.value.to_string(), "(1 2)");
}

#[test]
fn uk_reduce_works() {
    let mut s = uk_session();
    let r = eval_program(
        "(згорнути додати (список 1 2 3 4))",
        &mut s,
    ).expect("eval");
    assert_eq!(r.value.to_string(), "10");
}

// ── Strings: Ukrainian surface ─────────────────────────────────

#[test]
fn uk_string_operations() {
    let mut s = uk_session();
    let r = eval_program(r#"(довжина-тексту "привіт")"#, &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "6");
    let r2 = eval_program(r#"(порожній? "")"#, &mut s).expect("eval");
    assert_eq!(r2.value.to_string(), "t");
    let r3 = eval_program(r#"(зчепити "abc" "def")"#, &mut s).expect("eval");
    assert_eq!(r3.value.to_string(), "\"abcdef\"");
}

// ── Ukrainian + English coexistence ────────────────────────────

#[test]
fn uk_and_en_produce_same_result() {
    let mut s = uk_session();
    let uk_r = eval_program("(додати (помножити 3 4) (відняти 10 3))", &mut s).expect("eval");
    let en_r = eval_program("(+ (* 3 4) (- 10 3))", &mut s).expect("eval");
    assert_eq!(uk_r.value.to_string(), en_r.value.to_string());
    assert_eq!(uk_r.value.to_string(), "19");
}

// ── Sanskrit surface tests ─────────────────────────────────────

#[test]
fn sa_canon_works() {
    let mut s = sa_session();
    let r = eval_program("(svarūpa (+ 1 2))", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "(+ 1 2)");
    let r2 = eval_program("(ādi (saṃyuj 10 20))", &mut s).expect("eval");
    assert_eq!(r2.value.to_string(), "10");
    let r3 = eval_program("(śeṣa (saṃyuj 10 20))", &mut s).expect("eval");
    assert_eq!(r3.value.to_string(), "20");
}

#[test]
fn sa_arithmetic_works() {
    let mut s = sa_session();
    assert_eq!(eval_program("(yoga 1 2 3)", &mut s).unwrap().value.to_string(), "6");
    assert_eq!(eval_program("(viyoga 10 3)", &mut s).unwrap().value.to_string(), "7");
    assert_eq!(eval_program("(guṇana 2 3 4)", &mut s).unwrap().value.to_string(), "24");
    assert_eq!(eval_program("(haraṇa 6 2)", &mut s).unwrap().value.to_string(), "3");
}

#[test]
fn sa_comparisons_work() {
    let mut s = sa_session();
    assert_eq!(eval_program("(hīna? 1 2)", &mut s).unwrap().value.to_string(), "t");
    assert_eq!(eval_program("(adhika? 2 1)", &mut s).unwrap().value.to_string(), "t");
    assert_eq!(eval_program("(sama? 3 3)", &mut s).unwrap().value.to_string(), "t");
}

#[test]
fn sa_lists_higher_order_work() {
    let mut s = sa_session();
    let r = eval_program("(śreṇī 1 2 3)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "(1 2 3)");
    let r2 = eval_program("(pramāṇa (śreṇī 1 2 3))", &mut s).expect("eval");
    assert_eq!(r2.value.to_string(), "3");
    let r3 = eval_program(
        "(āvartana (lambda (x) (guṇana x 2)) (śreṇī 1 2 3))",
        &mut s,
    ).expect("eval");
    assert_eq!(r3.value.to_string(), "(2 4 6)");
    let r4 = eval_program(
        "(kalpana (lambda (x) (hīna? x 3)) (śreṇī 1 2 3 4))",
        &mut s,
    ).expect("eval");
    assert_eq!(r4.value.to_string(), "(1 2)");
}

#[test]
fn sa_and_en_produce_same_result() {
    let mut s = sa_session();
    let sa_r = eval_program("(yoga (guṇana 3 4) (viyoga 10 3))", &mut s).expect("eval");
    let en_r = eval_program("(+ (* 3 4) (- 10 3))", &mut s).expect("eval");
    assert_eq!(sa_r.value.to_string(), en_r.value.to_string());
    assert_eq!(sa_r.value.to_string(), "19");
}

// ── Cross-surface equivalence ──────────────────────────────────

#[test]
fn uk_and_sa_produce_same_result() {
    // Load both surfaces
    let mut s = Session::default();
    load_core_library(&mut s).expect("core");
    eval_program(include_str!("../../../lib/surface/uk.my"), &mut s).expect("uk");
    eval_program(include_str!("../../../lib/surface/sa.my"), &mut s).expect("sa");

    let uk_r = eval_program("(додати 1 2)", &mut s).expect("eval");
    let sa_r = eval_program("(yoga 1 2)", &mut s).expect("eval");
    let en_r = eval_program("(+ 1 2)", &mut s).expect("eval");
    assert_eq!(uk_r.value.to_string(), sa_r.value.to_string());
    assert_eq!(uk_r.value.to_string(), en_r.value.to_string());
    assert_eq!(uk_r.value.to_string(), "3");
}

// ── Ukrainian acceptance program ───────────────────────────────

#[test]
fn uk_acceptance_program_passes() {
    let mut s = uk_session();
    let r = eval_program(
        include_str!("../../../lib/surface/uk-acceptance.my"),
        &mut s,
    ).expect("Ukrainian acceptance program should evaluate");
    assert_eq!(
        r.value.to_string(), "успіх",
        "Ukrainian acceptance program must return 'успіх (success)"
    );
}

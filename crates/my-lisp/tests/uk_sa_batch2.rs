use my_lisp::{eval_program, load_core_library, load_time_library, Session};

/// Load core + time + Ukrainian surface, return a fresh session.
fn uk_session_full() -> Session {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core bootstrap");
    load_time_library(&mut session).expect("time library");
    eval_program(include_str!("../../../lib/surface/uk.my"), &mut session)
        .expect("Ukrainian surface should load");
    session
}

/// Load core + time + Sanskrit surface, return a fresh session.
fn sa_session_full() -> Session {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core bootstrap");
    load_time_library(&mut session).expect("time library");
    eval_program(include_str!("../../../lib/surface/sa.my"), &mut session)
        .expect("Sanskrit surface should load");
    session
}

// ── Persistent map: Ukrainian surface ──────────────────────────

#[test]
fn uk_persistent_map_basic() {
    let mut s = uk_session_full();
    let r = eval_program("(карта-порожня)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "()");
    let r2 = eval_program(
        "(карта-містить? (карта-вставити (карта-порожня) 'ключ 42) 'ключ)",
        &mut s,
    ).expect("eval");
    assert_eq!(r2.value.to_string(), "t");
    let r3 = eval_program(
        "(карта-отримати (карта-вставити (карта-порожня) 'ключ 42) 'ключ)",
        &mut s,
    ).expect("eval");
    assert_eq!(r3.value.to_string(), "42");
}

// ── Persistent vector: Ukrainian surface ───────────────────────

#[test]
fn uk_persistent_vector_basic() {
    let mut s = uk_session_full();
    let r = eval_program("(вектор-розмір (вектор-порожній))", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "0");
    let r2 = eval_program(
        "(вектор-розмір (вектор-додати (вектор-додати (вектор-порожній) 10) 20))",
        &mut s,
    ).expect("eval");
    assert_eq!(r2.value.to_string(), "2");
    let r3 = eval_program(
        "(вектор-за-номером (вектор-додати (вектор-порожній) 99) 0)",
        &mut s,
    ).expect("eval");
    assert_eq!(r3.value.to_string(), "99");
}

// ── Time library: Ukrainian surface ────────────────────────────

#[test]
fn uk_time_utc_now_works() {
    let mut s = uk_session_full();
    let r = eval_program("(поточний-utc)", &mut s).expect("eval");
    // utc-now returns (utc-time seconds nanoseconds) list
    assert!(r.value.to_string().contains("utc-time"));
}

#[test]
fn uk_time_mono_ms_works() {
    let mut s = uk_session_full();
    let r = eval_program("(монотонний-мс)", &mut s).expect("eval");
    // should be a number
    let val = r.value.to_string();
    assert!(val.parse::<f64>().is_ok() || val.parse::<i64>().is_ok(),
        "monotонний-мс should return a number, got: {val}");
}

// ── Knowledge: Ukrainian surface ───────────────────────────────

#[test]
fn uk_describe_works() {
    let mut s = uk_session_full();
    // describe should work on a simple value
    let r = eval_program("(описати 42)", &mut s).expect("eval");
    // describe returns a string description
    assert!(!r.value.to_string().is_empty());
}

// ── Unification: Ukrainian surface ─────────────────────────────

#[test]
fn uk_unify_basic() {
    let mut s = uk_session_full();
    // unify two identical atoms → empty substitution (success)
    let r = eval_program("(уніфікувати 'x 'x '())", &mut s).expect("eval");
    // result should be a non-failed substitution
    assert_ne!(r.value.to_string(), "()");
}

// ── Epistemic: Ukrainian surface ────────────────────────────────

#[test]
fn uk_claim_predicate() {
    let mut s = uk_session_full();
    // claim? on a non-claim should return falsy
    let r = eval_program("(твердження? 42)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "()");
}

// ── Sanskrit Batch 2: I/O ──────────────────────────────────────

#[test]
fn sa_io_eval_works() {
    let mut s = sa_session_full();
    let r = eval_program("(vicāraṇa '(+ 1 2))", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "3");
}

#[test]
fn sa_env_works() {
    let mut s = sa_session_full();
    let r = eval_program("(āśraya)", &mut s).expect("eval");
    // env returns a list — just check it doesn't error
    assert!(!r.value.to_string().is_empty());
}

// ── Sanskrit Batch 2: Conversions ──────────────────────────────

#[test]
fn sa_conversions_work() {
    let mut s = sa_session_full();
    let r = eval_program("(saṅkhyā-śabda 42)", &mut s).expect("eval");
    assert_eq!(r.value.to_string(), "\"42\"");
    let r2 = eval_program("(nāman-śabda 'кіт)", &mut s).expect("eval");
    assert_eq!(r2.value.to_string(), "\"кіт\"");
}

// ── Sanskrit Batch 2: Vectors ──────────────────────────────────

#[test]
fn sa_vector_works() {
    let mut s = sa_session_full();
    let r = eval_program("(samūha 1 2 3)", &mut s).expect("eval");
    // vector internal repr — just check no error
    assert!(!r.value.to_string().is_empty());
}

// ── Sanskrit Batch 2: Time ─────────────────────────────────────

#[test]
fn sa_time_unix_works() {
    let mut s = sa_session_full();
    let r = eval_program("(kāla-unix)", &mut s).expect("eval");
    assert!(r.value.to_string().contains("unix-time"));
}

// ── Cross-surface: UK Batch 2 = EN ─────────────────────────────

#[test]
fn uk_batch2_en_equivalence() {
    let mut s = uk_session_full();
    // map-get = карта-отримати
    let uk_r = eval_program(
        "(карта-отримати (карта-вставити (карта-порожня) 'x 99) 'x)",
        &mut s,
    ).expect("eval");
    let en_r = eval_program(
        "(map-get (map-insert (map-empty) 'x 99) 'x)",
        &mut s,
    ).expect("eval");
    assert_eq!(uk_r.value.to_string(), en_r.value.to_string());
    assert_eq!(uk_r.value.to_string(), "99");
}

// ── Cross-surface: SA Batch 2 = EN ─────────────────────────────

#[test]
fn sa_batch2_en_equivalence() {
    let mut s = sa_session_full();
    let sa_r = eval_program("(vicāraṇa '(* 3 4))", &mut s).expect("eval");
    let en_r = eval_program("(eval '(* 3 4))", &mut s).expect("eval");
    assert_eq!(sa_r.value.to_string(), en_r.value.to_string());
    assert_eq!(sa_r.value.to_string(), "12");
}

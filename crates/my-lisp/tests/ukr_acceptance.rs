use my_lisp::{eval_program, load_core_library, Session};

const UKR_ACCEPTANCE: &str = include_str!("../../../lib/surface/ukr-acceptance.lisp");

#[test]
fn ukr_acceptance_program_needs_no_latin_keyboard_layout() {
    assert!(
        !UKR_ACCEPTANCE
            .chars()
            .any(|character| character.is_ascii_alphabetic()),
        "ukr acceptance source must contain no ASCII Latin letters"
    );
}

#[test]
fn ukr_acceptance_program_executes_through_real_runtime() {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core bootstrap must install stable surface peers");
    let result = eval_program(UKR_ACCEPTANCE, &mut session)
        .expect("ukr acceptance program must evaluate through the real runtime");
    assert_eq!(result.value.to_string(), "успіх");
}

#[test]
fn shadowing_ukr_peer_does_not_retarget_english_peer() {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core bootstrap must install stable surface peers");

    let result = eval_program(
        "(визначити порожній-текст? (функція (значення) (як-є затінено)))\n(string-empty? \"\")",
        &mut session,
    )
    .expect("ordinary ukr peer must remain independently shadowable");

    // Expected shape queried live from Lisp, not hardcoded (#114/#220):
    // `string-empty?` is `eq`-based, so its "true" shape is whatever #218's
    // contract currently says `eq` returns for a matching comparison.
    let mut expected_session = Session::default();
    load_core_library(&mut expected_session).expect("core bootstrap must install stable surface peers");
    let eq_same_shape = eval_program("(eq 1 1)", &mut expected_session)
        .expect("eq must evaluate")
        .value
        .to_string();

    assert_eq!(result.value.to_string(), eq_same_shape);
}

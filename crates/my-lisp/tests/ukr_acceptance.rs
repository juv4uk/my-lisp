use my_lisp::{eval_program, Session};

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
    let result = eval_program(UKR_ACCEPTANCE, &mut session)
        .expect("ukr acceptance program must evaluate through the real runtime");
    assert_eq!(result.value.to_string(), "успіх");
}

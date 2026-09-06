use my_lisp::{eval_program, Session};

fn session_with_language_canon() -> Session {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/canon.my"), &mut session)
        .expect("lib/canon.my must bootstrap as executable semantics");
    session
}

#[test]
fn language_level_canon_is_the_conformance_authority() {
    let mut session = session_with_language_canon();
    let verdict = eval_program("(canon-conforms?)", &mut session)
        .expect("runtime must be able to execute the language-owned semantic laws");
    assert_eq!(verdict.value.to_string(), "t");
}

#[test]
fn surface_binding_captures_primitive_value_not_historical_name_lookup() {
    let mut session = session_with_language_canon();
    let result = eval_program(
        "(def car (lambda (x) (quote зламано))) (перше (сполучити 1 2))",
        &mut session,
    )
    .expect("rebinding historical car must not mutate the captured canonical surface");
    assert_eq!(result.value.to_string(), "1");
}

#[test]
fn ukrainian_and_sanskrit_surfaces_are_independent_bindings_to_one_semantics() {
    let mut session = session_with_language_canon();
    let result = eval_program(
        "(def перше (lambda (x) (quote локально))) (ādi (saṃyuj 1 2))",
        &mut session,
    )
    .expect("shadowing one language surface must not mutate another surface binding");
    assert_eq!(result.value.to_string(), "1");
}

#[test]
fn canon_zero_is_stated_by_the_language_as_the_empty_list_itself() {
    let mut session = session_with_language_canon();
    let result = eval_program("canon-empty-list", &mut session)
        .expect("language canon should expose its Canon 0 witness");
    assert_eq!(result.value.to_string(), "()");
}

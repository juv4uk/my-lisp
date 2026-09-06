use my_lisp::{eval_program, load_core_library, Session};

#[test]
fn ukrainian_surface_defines_functions_without_rust_knowing_ukrainian_form_names() {
    let mut session = Session::default();
    load_core_library(&mut session).expect("canonical macro + core bootstrap should preload");
    eval_program(include_str!("../../../lib/surface/uk.my"), &mut session)
        .expect("Ukrainian surface should preload");

    let result = eval_program(
        r#"
            (визначити квадрат
              (функція (х)
                (* х х)))
            (квадрат 7)
        "#,
        &mut session,
    )
    .expect("Ukrainian DEFINE/LAMBDA surface should execute");

    assert_eq!(result.value.to_string(), "49");
}

#[test]
fn historical_def_remains_compatible_but_is_not_the_canonical_form_identity() {
    let mut session = Session::default();
    let result = eval_program("(def x 5) (define y 7) (+ x y)", &mut session)
        .expect("def compatibility and canonical define should coexist");
    assert_eq!(result.value.to_string(), "12");
}

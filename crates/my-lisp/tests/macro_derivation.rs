use my_lisp::{eval_program, Session};

fn eval_with_derived_macros(source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/macro.my"), &mut session)
        .expect("derived macro layer should load");
    eval_program(source, &mut session)
        .expect("program using derived macros should evaluate")
        .value
        .to_string()
}

#[test]
fn language_owned_defmacro_introduces_a_working_macro() {
    let value = eval_with_derived_macros(
        r#"
        (defmacro identity (x) x)
        (identity 42)
        "#,
    );
    assert_eq!(value, "42");
}

#[test]
fn language_owned_defmacro_preserves_unevaluated_arguments() {
    let value = eval_with_derived_macros(
        r#"
        (defmacro first-form (a b) a)
        (first-form (quote ok) never-defined)
        "#,
    );
    assert_eq!(value, "ok");
}

#[test]
fn language_owned_defmacro_can_build_control_flow() {
    let value = eval_with_derived_macros(
        r#"
        (defmacro unless (condition body)
          (cons (quote cond)
            (cons
              (cons condition
                (cons (quote ()) (quote ())))
              (cons
                (cons (quote t)
                  (cons body (quote ())))
                (quote ())))))
        (unless () (quote success))
        "#,
    );
    assert_eq!(value, "success");
}

#[test]
fn transitional_defmacro_derived_name_still_works() {
    let value = eval_with_derived_macros(
        r#"
        (defmacro-derived identity-old (x) x)
        (identity-old 7)
        "#,
    );
    assert_eq!(value, "7");
}

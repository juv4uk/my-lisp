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
fn defmacro_derived_introduces_a_working_macro() {
    let value = eval_with_derived_macros(
        r#"
        (defmacro-derived identity (x) x)
        (identity 42)
        "#,
    );
    assert_eq!(value, "42");
}

#[test]
fn defmacro_derived_preserves_unevaluated_arguments() {
    let value = eval_with_derived_macros(
        r#"
        (defmacro-derived first-form (a b) a)
        (first-form (quote ok) never-defined)
        "#,
    );
    assert_eq!(value, "ok");
}

#[test]
fn defmacro_derived_can_build_control_flow() {
    let value = eval_with_derived_macros(
        r#"
        (defmacro-derived unless (condition body)
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

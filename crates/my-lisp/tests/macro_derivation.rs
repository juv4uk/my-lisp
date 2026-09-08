use my_lisp::{eval_program, load_macro_library, Environment, Session, Value};
use std::rc::Rc;

fn eval_with_derived_macros(source: &str) -> String {
    let mut session = Session::default();
    eval_program(source, &mut session)
        .expect("program using derived macros should evaluate")
        .value
        .to_string()
}

fn assert_same_macro_value(values: [Value; 3]) {
    match (&values[0], &values[1], &values[2]) {
        (Value::Macro(defmacro), Value::Macro(uk), Value::Macro(compat)) => {
            assert!(
                Rc::ptr_eq(defmacro, uk),
                "defmacro and визначити-макрос must share one Macro value"
            );
            assert!(
                Rc::ptr_eq(defmacro, compat),
                "defmacro-derived must point to the same compatibility value"
            );
        }
        other => panic!("expected three Macro bindings, got {other:?}"),
    }
}

#[test]
fn default_session_binds_all_defmacro_peers_to_one_value() {
    let session = Session::default();
    assert_same_macro_value([
        session
            .environment
            .get("defmacro")
            .expect("default session must bind defmacro"),
        session
            .environment
            .get("визначити-макрос")
            .expect("default session must bind визначити-макрос"),
        session
            .environment
            .get("defmacro-derived")
            .expect("default session must retain defmacro-derived"),
    ]);
}

#[test]
fn bare_root_gains_peer_bindings_only_through_macro_loader() {
    let environment = Environment::root();
    assert!(environment.get("defmacro").is_none());
    assert!(environment.get("визначити-макрос").is_none());
    assert!(environment.get("defmacro-derived").is_none());

    let mut session = Session { environment };
    let loaded = load_macro_library(&mut session).expect("macro library should bootstrap");
    let loaded_macro = match &loaded.value {
        Value::Macro(value) => value,
        other => panic!("macro library must return one Macro value, got {other:?}"),
    };

    let defmacro = session
        .environment
        .get("defmacro")
        .expect("loader must bind defmacro");
    let uk = session
        .environment
        .get("визначити-макрос")
        .expect("loader must bind визначити-макрос");
    let compat = session
        .environment
        .get("defmacro-derived")
        .expect("loader must bind defmacro-derived");

    assert_same_macro_value([defmacro.clone(), uk, compat]);
    match &defmacro {
        Value::Macro(bound) => assert!(
            Rc::ptr_eq(loaded_macro, bound),
            "loader must bind the exact Macro value returned by lib/macro.my"
        ),
        other => panic!("defmacro binding must be a Macro value, got {other:?}"),
    }
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
fn ukrainian_defmacro_peer_works_without_loading_uk_surface_bridge() {
    let value = eval_with_derived_macros(
        r#"
        (визначити-макрос identity-uk (x) x)
        (identity-uk 11)
        "#,
    );
    assert_eq!(value, "11");
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

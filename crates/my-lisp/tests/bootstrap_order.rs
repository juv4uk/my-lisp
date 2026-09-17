use my_lisp::{
    eval_program, load_core_library, load_text_boundary_libraries, Session, Value,
};

#[test]
fn canonical_core_loader_installs_language_owned_defmacro_before_core() {
    let mut session = Session::default();
    load_core_library(&mut session).expect("macro.my then core.my should bootstrap cleanly");

    assert!(
        matches!(session.environment.get("defmacro"), Some(Value::Macro(_))),
        "defmacro must be a language-owned macro binding after canonical bootstrap"
    );

    let value = eval_program(
        r#"
        (defmacro choose-first (left right) left)
        (choose-first (quote ok) never-defined)
        "#,
        &mut session,
    )
    .expect("language-owned defmacro should preserve unevaluated macro arguments")
    .value;
    assert_eq!(value.to_string(), "ok");
}

#[test]
fn core_macros_work_after_canonical_macro_first_bootstrap() {
    let mut session = Session::default();
    load_core_library(&mut session).expect("macro.my then core.my should bootstrap cleanly");

    let value = eval_program("(and t 42)", &mut session)
        .expect("core.my macros should be available after canonical bootstrap")
        .value;
    assert_eq!(value.to_string(), "42");

    let value = eval_program("(or () (quote fallback))", &mut session)
        .expect("core.my OR macro should be available after canonical bootstrap")
        .value;
    assert_eq!(value.to_string(), "fallback");
}

#[test]
fn shared_text_boundary_loader_installs_process_tcp_and_fs_adapters() {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core bootstrap");
    load_text_boundary_libraries(&mut session)
        .expect("shared text bootstrap should install all text boundary adapters");

    assert!(matches!(
        session.environment.get("process-run"),
        Some(Value::Closure(_))
    ));
    assert!(matches!(
        session.environment.get("tcp-read"),
        Some(Value::Closure(_))
    ));
    assert!(matches!(
        session.environment.get("read-file"),
        Some(Value::Closure(_))
    ));
    assert!(matches!(
        session.environment.get("write-file"),
        Some(Value::Closure(_))
    ));
}

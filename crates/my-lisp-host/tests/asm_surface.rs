use my_lisp::{
    capability_installed, eval_program, load_core_library, load_fs_library, load_process_library,
    Session, Value,
};

fn asm_session() -> Session {
    my_lisp_host::install();
    let mut session = Session::default();
    load_core_library(&mut session).expect("core library must load");
    load_process_library(&mut session).expect("process library must load");
    load_fs_library(&mut session).expect("filesystem library must load");

    let source = std::fs::read_to_string("lib/asm-x86.lisp")
        .expect("Lisp-owned x86 assembler library must exist");
    eval_program(&source, &mut session).expect("Lisp-owned x86 assembler library must load");
    session
}

#[test]
fn x86_assembler_surface_is_owned_by_lisp_not_host_capabilities() {
    my_lisp_host::install();
    assert!(!capability_installed("x86-ret"));
    assert!(!capability_installed("x86-assemble"));

    let session = asm_session();
    assert!(matches!(
        session.environment.get("x86-ret"),
        Some(Value::Closure(_))
    ));
    assert!(matches!(
        session.environment.get("x86-assemble"),
        Some(Value::Closure(_))
    ));
}

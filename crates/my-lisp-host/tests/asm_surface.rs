use my_lisp::{capability_installed, eval_program, Session};

/// Асемблерна межа наслідує process/fs-патерн: host дає лише сирий
/// механізм кодування, а публічна Lisp-поверхня не реєструється як host form.
#[test]
fn host_installs_only_raw_x86_assembler_capability() {
    my_lisp_host::install();

    assert!(
        capability_installed("x86-assemble-raw"),
        "host must expose the raw assembler mechanism"
    );
    assert!(
        !capability_installed("x86-assemble"),
        "public assembler semantics must remain Lisp-owned"
    );
    assert!(
        !capability_installed("x86-ret"),
        "instruction constructors must remain Lisp-owned"
    );
}

#[test]
fn raw_x86_assembler_encodes_lisp_instruction_data() {
    my_lisp_host::install();
    let mut session = Session::default();

    let result = eval_program(
        "(x86-assemble-raw (quote ((ret))))",
        &mut session,
    )
    .expect("raw assembler capability should encode quoted Lisp instruction data");

    assert_eq!(result.value.to_string(), "(195)");
}

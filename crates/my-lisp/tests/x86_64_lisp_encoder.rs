use my_lisp::{eval_program, load_core_library, Session};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn encoder_session() -> Session {
    let path = repo_root().join("lib/machine/encoding/x86-64.lisp");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", path.display()));

    let mut session = Session::default();
    load_core_library(&mut session).expect("core must bootstrap before machine encoder");
    eval_program(&source, &mut session).expect("x86-64 encoder must load as ordinary my-lisp");
    session
}

fn eval_bytes(source: &str, session: &mut Session) -> String {
    eval_program(source, session)
        .unwrap_or_else(|error| panic!("encoder expression failed: {source}: {error}"))
        .value
        .to_string()
}

#[test]
fn lisp_encodes_ret_to_exact_machine_byte() {
    let mut session = encoder_session();
    assert_eq!(eval_bytes("(x86-encode-ret)", &mut session), "(195)");
}

#[test]
fn lisp_encodes_mov_eax_imm32_little_endian() {
    let mut session = encoder_session();
    assert_eq!(
        eval_bytes("(x86-encode-mov-eax-imm32 42)", &mut session),
        "(184 42 0 0 0)"
    );
}

#[test]
fn lisp_encodes_add_rax_rbx_without_external_assembler() {
    let mut session = encoder_session();
    assert_eq!(
        eval_bytes(
            "(x86-encode-add-r64-r64 (quote rax) (quote rbx))",
            &mut session
        ),
        "(72 1 216)"
    );
}

#[test]
fn encoder_source_contains_no_process_or_assembler_escape_hatch() {
    let path = repo_root().join("lib/machine/encoding/x86-64.lisp");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", path.display()));
    let lower = source.to_ascii_lowercase();

    for forbidden in ["process-run", "nasm", "as ", "objcopy", "keystone", "iced-x86"] {
        assert!(
            !lower.contains(forbidden),
            "proof encoder must construct bytes in Lisp, not escape through {forbidden}"
        );
    }
}

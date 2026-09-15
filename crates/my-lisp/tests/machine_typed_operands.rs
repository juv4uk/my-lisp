use my_lisp::{eval_program, load_core_library, Session};
use std::fs;
use std::path::PathBuf;

const REGISTRY: &str = include_str!("../../../lib/surface/semantic-registry.lisp");

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_lisp_file(path: &str, session: &mut Session) {
    let path = repo_root().join(path);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", path.display()));
    eval_program(&source, session)
        .unwrap_or_else(|error| panic!("{} must load as ordinary my-lisp: {error}", path.display()));
}

fn machine_session() -> Session {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core must bootstrap before machine operands");
    load_lisp_file("lib/machine/encoding/x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/operands/x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/block.lisp", &mut session);
    load_lisp_file("lib/machine/admission/x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/atoms/x86-64.lisp", &mut session);
    session
}

fn eval_value(source: &str, session: &mut Session) -> String {
    eval_program(source, session)
        .unwrap_or_else(|error| panic!("machine expression failed: {source}: {error}"))
        .value
        .to_string()
}

#[test]
fn typed_operands_are_explicit_lisp_machine_data() {
    let mut session = machine_session();

    for (source, expected) in [
        ("(x86-gpr64 (quote rax))", "(gpr64 rax)"),
        ("(x86-u64-imm 42)", "(u64-imm 42)"),
        (
            "(x86-u64-imm 18446744073709551615)",
            "(u64-imm 18446744073709551615)",
        ),
        ("(x86-disp8 -8)", "(disp8 -8)"),
        (
            "(x86-mem64-disp8 (x86-gpr64 (quote rdi)) (x86-disp8 8))",
            "(mem64-disp8 (gpr64 rdi) (disp8 8))",
        ),
    ] {
        assert_eq!(eval_value(source, &mut session), expected, "{source}");
    }
}

#[test]
fn invalid_operand_values_fail_closed_before_machine_form_creation() {
    let mut session = machine_session();

    for (source, expected) in [
        (
            "(x86-gpr64 (quote xmm0))",
            "(rejected machine-operand gpr64 xmm0)",
        ),
        (
            "(x86-u64-imm -1)",
            "(rejected machine-operand u64-imm -1)",
        ),
        (
            "(x86-u64-imm 18446744073709551616)",
            "(rejected machine-operand u64-imm 18446744073709551616)",
        ),
        (
            "(x86-u64-imm 1/2)",
            "(rejected machine-operand u64-imm 1/2)",
        ),
        (
            "(x86-disp8 128)",
            "(rejected machine-operand disp8 128)",
        ),
        (
            "(x86-disp8 -129)",
            "(rejected machine-operand disp8 -129)",
        ),
    ] {
        assert_eq!(eval_value(source, &mut session), expected, "{source}");
    }
}

#[test]
fn typed_registers_and_immediates_project_to_existing_176_forms() {
    let mut session = machine_session();

    assert_eq!(
        eval_value(
            "(x86-mov-r64-imm64 (x86-gpr64 (quote rax)) (x86-u64-imm 42))",
            &mut session,
        ),
        "(mov-r64-imm64 rax 42)"
    );
    assert_eq!(
        eval_value(
            "(x86-add-r64-r64 (x86-gpr64 (quote rax)) (x86-gpr64 (quote rcx)))",
            &mut session,
        ),
        "(add-r64-r64 rax rcx)"
    );
}

#[test]
fn typed_memory_operands_project_to_existing_disp8_forms() {
    let mut session = machine_session();

    let memory = "(x86-mem64-disp8 (x86-gpr64 (quote rdi)) (x86-disp8 8))";
    assert_eq!(
        eval_value(
            &format!("(x86-mov-mem64-r64 {memory} (x86-gpr64 (quote rax)))"),
            &mut session,
        ),
        "(mov-mem-disp8-r64 rdi 8 rax)"
    );
    assert_eq!(
        eval_value(
            &format!("(x86-mov-r64-mem64 (x86-gpr64 (quote rax)) {memory})"),
            &mut session,
        ),
        "(mov-r64-mem-disp8 rax rdi 8)"
    );
}

#[test]
fn operand_type_success_does_not_bypass_instruction_admission() {
    let mut session = machine_session();

    assert_eq!(
        eval_value(
            "(x86-encode-machine-block (machine-block-one (x86-add-r64-r64 (x86-gpr64 (quote rax)) (x86-gpr64 (quote rbx)))))",
            &mut session,
        ),
        "(rejected unadmitted-machine-form (add-r64-r64 rax rbx))"
    );
}

#[test]
fn invalid_typed_operands_propagate_instead_of_becoming_instruction_forms() {
    let mut session = machine_session();

    assert_eq!(
        eval_value(
            "(x86-add-r64-r64 (x86-gpr64 (quote xmm0)) (x86-gpr64 (quote rcx)))",
            &mut session,
        ),
        "(rejected machine-operand gpr64 xmm0)"
    );
}

#[test]
fn machine_operand_type_names_do_not_mint_semantic_surfaces() {
    let lower = REGISTRY.to_ascii_lowercase();
    for machine_only in [
        "x86-gpr64",
        "x86-u64-imm",
        "x86-disp8",
        "x86-mem64-disp8",
        "mem64-disp8",
    ] {
        assert!(
            !lower.contains(machine_only),
            "machine-only operand type {machine_only} must not become a semantic registry surface"
        );
    }
}

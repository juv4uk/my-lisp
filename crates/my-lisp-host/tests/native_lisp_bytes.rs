#![cfg(all(target_os = "linux", target_arch = "x86_64"))]

use my_lisp::{eval_program, load_core_library, Session};
use my_lisp_host::install;
use std::fs;
use std::path::PathBuf;

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

#[test]
fn x86_pair_layout_is_one_lisp_owned_machine_readable_authority() {
    let path = repo_root().join("lib/machine/layout/pair-x86-64.lisp");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", path.display()));

    my_lisp::parse(&source).expect("x86 pair layout authority must be valid my-lisp");
    for required in [
        "(def x86-pair-cell-bytes 16)",
        "(def x86-pair-car-offset 0)",
        "(def x86-pair-cdr-offset 8)",
        "(target x86-64)",
        "(arena-argument-register rdi)",
        "(lifetime native-call)",
        "(escape forbidden)",
    ] {
        assert!(source.contains(required), "pair layout authority missing {required}");
    }

    assert!(
        !source.contains("semantic-id"),
        "machine representation layout must not allocate language semantic identities"
    );

    let mut session = Session::default();
    load_core_library(&mut session).expect("core must bootstrap before layout authority");
    eval_program(&source, &mut session).expect("pair layout authority must evaluate as ordinary my-lisp");

    for (name, expected) in [
        ("x86-pair-cell-bytes", "16"),
        ("x86-pair-car-offset", "0"),
        ("x86-pair-cdr-offset", "8"),
    ] {
        let actual = eval_program(name, &mut session)
            .unwrap_or_else(|error| panic!("{name} must be queryable: {error}"))
            .value
            .to_string();
        assert_eq!(actual, expected);
    }
}

#[test]
fn interpreter_pair_reference_witnesses_remain_two_and_three() {
    install();
    let mut session = Session::default();
    load_core_library(&mut session).expect("core must bootstrap before pair reference witness");

    let car = eval_program("(перше (сполучити 2 3))", &mut session)
        .expect("interpreter CAR witness must remain valid");
    let cdr = eval_program("(решта (сполучити 2 3))", &mut session)
        .expect("interpreter CDR witness must remain valid");

    assert_eq!(car.value.to_string(), "2");
    assert_eq!(cdr.value.to_string(), "3");
}

#[test]
fn semantics_blind_raw_executor_accepts_optional_arena_bytes() {
    install();
    let mut session = Session::default();
    load_core_library(&mut session).expect("core must bootstrap before native witness");
    load_lisp_file("lib/machine/encoding/x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/lowering/semantic-x86-64.lisp", &mut session);

    let result = eval_program(
        "(native-call-u64-raw (x86-lower-add-u64 2 3) 16)",
        &mut session,
    )
    .expect("semantics-blind host must optionally provide a raw arena pointer to Lisp-owned bytes");

    assert_eq!(result.value.to_string(), "5");
}

#[test]
fn lisp_owned_add_bytes_execute_natively_through_semantics_blind_host() {
    install();
    let mut session = Session::default();
    load_core_library(&mut session).expect("core must bootstrap before native witness");
    load_lisp_file("lib/machine/encoding/x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/lowering/semantic-x86-64.lisp", &mut session);

    let result = eval_program(
        "(native-call-u64-raw (x86-lower-add-u64 2 3))",
        &mut session,
    )
    .expect("host must execute exactly the bytes produced by Lisp lowering");

    assert_eq!(result.value.to_string(), "5");
}

#[test]
fn native_execution_mechanism_is_not_a_language_semantic_identity() {
    let registry = fs::read_to_string(repo_root().join("lib/surface/semantic-registry.lisp"))
        .expect("semantic registry must be readable");
    assert!(
        !registry.contains("native-call-u64-raw"),
        "raw native invocation is host mechanism, never a language semantic identity"
    );
}

#[test]
fn rust_native_executor_contains_no_lisp_or_x86_lowering_decision() {
    let source = fs::read_to_string(repo_root().join("crates/my-lisp-host/src/native_exec.rs"))
        .expect("native execution mechanism source must be readable");

    for forbidden in [
        "0104",
        "0004",
        "0005",
        "0006",
        "x86-lower",
        "x86-encode",
        "ADD",
        "ADDSD",
        "CAR",
        "CDR",
        "CONS",
        "semantic-registry",
    ] {
        assert!(
            !source.contains(forbidden),
            "host executor must remain semantics-blind; found forbidden lowering token {forbidden}"
        );
    }

    for required in ["mmap", "mprotect", "munmap", "PROT_WRITE", "PROT_EXEC"] {
        assert!(
            source.contains(required),
            "host executor must expose only native memory/call mechanism; missing {required}"
        );
    }
}

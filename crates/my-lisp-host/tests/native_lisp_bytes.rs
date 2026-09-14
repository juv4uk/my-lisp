#![cfg(all(target_os = "linux", target_arch = "x86_64"))]

use my_lisp::{
    eval_program, load_core_library, register_capability, Environment, Exactness, Expr,
    LanguageError, Session, Span, Value,
};
use my_lisp_host::install;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};

static EXECUTOR_CALLS: AtomicUsize = AtomicUsize::new(0);
static TEST_LOCK: Mutex<()> = Mutex::new(());

struct RestoreHostCapabilities;

impl Drop for RestoreHostCapabilities {
    fn drop(&mut self) {
        install();
    }
}

fn test_lock() -> MutexGuard<'static, ()> {
    TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

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

fn spy_executor(
    _arguments: &[Expr],
    _environment: &Environment,
    _span: Span,
) -> Result<Value, LanguageError> {
    EXECUTOR_CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(Value::Number(0.0, Exactness::Exact))
}

#[test]
fn unadmitted_ud2_form_is_rejected_before_host_executor() {
    let _serial = test_lock();
    let mut session = Session::default();
    load_core_library(&mut session).expect("core must bootstrap before machine admission witness");
    load_lisp_file("lib/machine/encoding/x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/admission/x86-64.lisp", &mut session);

    EXECUTOR_CALLS.store(0, Ordering::SeqCst);
    register_capability("native-call-u64-raw", spy_executor);
    let _restore = RestoreHostCapabilities;

    let result = eval_program(
        "(x86-call-admitted-u64 (quote ((ud2))) 0)",
        &mut session,
    )
    .expect("unadmitted machine form must be a Lisp-level named rejection");

    assert_eq!(
        result.value.to_string(),
        "(rejected unadmitted-machine-form (ud2))"
    );
    assert_eq!(
        EXECUTOR_CALLS.load(Ordering::SeqCst),
        0,
        "rejected machine form must never reach the host executor"
    );
}

#[test]
fn semantic_lowering_must_produce_structured_forms_before_admission_and_execution() {
    let _serial = test_lock();
    install();
    let mut session = Session::default();
    load_core_library(&mut session).expect("core must bootstrap before structured machine lowering");
    load_lisp_file("lib/machine/layout/pair-x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/encoding/x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/admission/x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/lowering/semantic-x86-64.lisp", &mut session);

    let add_forms = eval_program("(x86-lower-add-u64-forms 2 3)", &mut session)
        .expect("semantic lowering must expose structured admitted machine forms before bytes");
    assert_eq!(
        add_forms.value.to_string(),
        "((mov-r64-imm64 rax 2) (mov-r64-imm64 rcx 3) (add-r64-r64 rax rcx) (ret))"
    );

    let add = eval_program(
        "(x86-call-admitted-u64 (x86-lower-add-u64-forms 2 3) 0)",
        &mut session,
    )
    .expect("admitted semantic ADD forms must execute through the canonical gateway");
    assert_eq!(add.value.to_string(), "5");

    let interpreter_car = eval_program("(перше (сполучити 2 3))", &mut session)
        .expect("interpreter CAR reference witness must remain valid");
    let interpreter_cdr = eval_program("(решта (сполучити 2 3))", &mut session)
        .expect("interpreter CDR reference witness must remain valid");

    let native_car = eval_program(
        "(x86-call-admitted-u64 (x86-lower-cons-car-u64-forms 2 3) x86-pair-cell-bytes)",
        &mut session,
    )
    .expect("CONS+CAR lowering must pass through admission before host execution");
    let native_cdr = eval_program(
        "(x86-call-admitted-u64 (x86-lower-cons-cdr-u64-forms 2 3) x86-pair-cell-bytes)",
        &mut session,
    )
    .expect("CONS+CDR lowering must pass through admission before host execution");

    assert_eq!(native_car.value, interpreter_car.value);
    assert_eq!(native_cdr.value, interpreter_cdr.value);

    let lowering_source = fs::read_to_string(repo_root().join("lib/machine/lowering/semantic-x86-64.lisp"))
        .expect("semantic lowerer source must be readable");
    assert!(
        lowering_source.contains("x86-encode-admitted-program"),
        "byte compatibility wrappers must route through the admitted encoder"
    );
    assert!(
        !lowering_source.contains("(x86-encode-program\n"),
        "semantic lowerer must not bypass admission by flattening raw encoded instructions itself"
    );
}

#[test]
fn x86_pair_layout_is_one_lisp_owned_machine_readable_authority() {
    let _serial = test_lock();
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
    let _serial = test_lock();
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
    let _serial = test_lock();
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
fn lisp_owned_pair_memory_addressing_has_exact_rdi_disp8_bytes() {
    let _serial = test_lock();
    install();
    let mut session = Session::default();
    load_core_library(&mut session).expect("core must bootstrap before machine encoder witness");
    load_lisp_file("lib/machine/layout/pair-x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/encoding/x86-64.lisp", &mut session);

    let car_load = eval_program(
        "(x86-encode-mov-r64-mem-disp8 (quote rax) (quote rdi) x86-pair-car-offset)",
        &mut session,
    )
    .expect("Lisp encoder must encode CAR's [rdi+0] load");
    let cdr_load = eval_program(
        "(x86-encode-mov-r64-mem-disp8 (quote rax) (quote rdi) x86-pair-cdr-offset)",
        &mut session,
    )
    .expect("Lisp encoder must encode CDR's [rdi+8] load");
    let car_store = eval_program(
        "(x86-encode-mov-mem-disp8-r64 (quote rdi) x86-pair-car-offset (quote rax))",
        &mut session,
    )
    .expect("Lisp encoder must encode CONS's [rdi+0] store");

    assert_eq!(car_load.value.to_string(), "(72 139 71 0)");
    assert_eq!(cdr_load.value.to_string(), "(72 139 71 8)");
    assert_eq!(car_store.value.to_string(), "(72 137 71 0)");
}

#[test]
fn native_pair_car_cdr_match_the_interpreter_reference_witness() {
    let _serial = test_lock();
    install();
    let mut session = Session::default();
    load_core_library(&mut session).expect("core must bootstrap before pair parity witness");
    load_lisp_file("lib/machine/layout/pair-x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/encoding/x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/lowering/semantic-x86-64.lisp", &mut session);

    let interpreter_car = eval_program("(перше (сполучити 2 3))", &mut session)
        .expect("interpreter CAR reference witness must remain valid");
    let interpreter_cdr = eval_program("(решта (сполучити 2 3))", &mut session)
        .expect("interpreter CDR reference witness must remain valid");

    let native_car = eval_program(
        "(native-call-u64-raw (x86-lower-cons-car-u64 2 3) x86-pair-cell-bytes)",
        &mut session,
    )
    .expect("Lisp-owned CONS+CAR bytes must execute through the raw arena mechanism");
    let native_cdr = eval_program(
        "(native-call-u64-raw (x86-lower-cons-cdr-u64 2 3) x86-pair-cell-bytes)",
        &mut session,
    )
    .expect("Lisp-owned CONS+CDR bytes must execute through the raw arena mechanism");

    assert_eq!(native_car.value, interpreter_car.value);
    assert_eq!(native_cdr.value, interpreter_cdr.value);
}

#[test]
fn lisp_owned_add_bytes_execute_natively_through_semantics_blind_host() {
    let _serial = test_lock();
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
    let _serial = test_lock();
    let registry = fs::read_to_string(repo_root().join("lib/surface/semantic-registry.lisp"))
        .expect("semantic registry must be readable");
    assert!(
        !registry.contains("native-call-u64-raw"),
        "raw native invocation is host mechanism, never a language semantic identity"
    );
}

#[test]
fn rust_native_executor_contains_no_lisp_or_x86_lowering_decision() {
    let _serial = test_lock();
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

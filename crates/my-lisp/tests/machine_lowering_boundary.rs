use std::fs;
use std::path::PathBuf;

const REGISTRY: &str = include_str!("../../../lib/surface/semantic-registry.lisp");
const REPO_DECLARATION: &str = include_str!("../../../repo.lisp");

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn portable_monotonic_time_keeps_language_semantic_identity() {
    let row = REGISTRY
        .lines()
        .find(|line| line.trim_start().starts_with("(1075 "))
        .expect("semantic ID 1075 must remain the portable monotonic observation");

    assert!(
        row.contains("(en mono-ns stable)"),
        "1075 must keep the portable mono-ns semantic surface"
    );
}

#[test]
fn raw_machine_identity_never_becomes_a_language_semantic() {
    let lower = REGISTRY.to_ascii_lowercase();
    assert!(
        !lower.contains("rdtsc"),
        "raw target instruction names are machine facts, not language semantic identities"
    );
    assert!(
        !REGISTRY
            .lines()
            .any(|line| line.trim_start().starts_with("(1153 ")),
        "historical compiler-originated allocation 1153 must not return as an active semantic row"
    );
}

#[test]
fn vertical_machine_boundary_separates_semantics_isa_optimization_and_host() {
    let path = repo_root().join("machine-lowering-boundary.lisp");
    let contract = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", path.display()));

    for required in [
        "(semantic-authority my-lisp)",
        "(isa-authority hardware-specification)",
        "(isa-source intel-xed/intel-sdm)",
        "(isa-representation my-lisp)",
        "(instruction-encoding my-lisp)",
        "(optimization-authority cml)",
        "(semantic-id-from-isa forbidden)",
        "(raw-execution-mechanism host)",
        "(lowering-direction semantic-to-machine)",
        "(retired-semantic-id 1153)",
    ] {
        assert!(
            contract.contains(required),
            "machine boundary missing required vertical authority fact: {required}"
        );
    }

    for obsolete in [
        "(compiler-authority cml)",
        "(machine-instruction-identity compiler-owned)",
        "(raw-machine-instructions nonportable-compiler-mechanism)",
    ] {
        assert!(
            !contract.contains(obsolete),
            "old compiler-monopoly fact must be removed: {obsolete}"
        );
    }
}

#[test]
fn x86_base_catalogue_is_lisp_data_independent_of_semantic_ids() {
    let path = repo_root().join("lib/machine/isa/x86-base.lisp");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", path.display()));

    my_lisp::parse(&source).expect("x86 base ISA catalogue must be valid my-lisp data");
    assert!(source.contains("(isa-catalogue/1"));
    assert!(source.contains("(extension X86-BASE"));
    for mnemonic in ["MOV", "ADD", "RET"] {
        assert!(
            source.contains(&format!("(instruction {mnemonic} ")),
            "proof catalogue must contain {mnemonic} independently of Lisp semantics"
        );
    }
    assert!(
        !source.contains("semantic-id"),
        "ISA facts must never allocate or embed language semantic IDs"
    );
}

#[test]
fn i5_6400_cpu_profile_references_declared_isa_extensions_not_semantics() {
    let path = repo_root().join("lib/machine/cpu/intel-core-i5-6400.lisp");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", path.display()));

    my_lisp::parse(&source).expect("i5-6400 CPU profile must be valid my-lisp data");
    for required in [
        "(cpu intel-core-i5-6400)",
        "(microarchitecture skylake)",
        "(isa x86-64)",
        "(supported-extension X86-BASE)",
    ] {
        assert!(source.contains(required), "CPU profile missing {required}");
    }
    assert!(
        !source.contains("semantic-id"),
        "CPU capability profile must not own language semantic IDs"
    );
}

#[test]
fn repo_exports_machine_lowering_boundary_for_consumers() {
    assert!(
        REPO_DECLARATION.contains("machine-lowering-boundary"),
        "repo.lisp must export the machine authority boundary"
    );
}

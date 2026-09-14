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
fn compiler_originated_machine_identity_is_not_an_active_language_semantic() {
    let lower = REGISTRY.to_ascii_lowercase();
    assert!(
        !lower.contains("rdtsc"),
        "raw target instruction names belong to compiler machine identity, not the language semantic registry"
    );
    assert!(
        !REGISTRY
            .lines()
            .any(|line| line.trim_start().starts_with("(1153 ")),
        "historical compiler-originated allocation 1153 must not remain an active semantic row"
    );
}

#[test]
fn semantic_to_machine_boundary_is_machine_readable_one_way_and_non_reusing() {
    let path = repo_root().join("machine-lowering-boundary.lisp");
    let contract = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", path.display()));

    for required in [
        "(semantic-authority my-lisp)",
        "(compiler-authority cml)",
        "(portable-monotonic-observation 1075)",
        "(machine-instruction-identity compiler-owned)",
        "(semantic-id-allocation explicit-language-contract-only)",
        "(lowering-direction semantic-to-machine)",
        "(reverse-authority forbidden)",
        "(raw-machine-instructions nonportable-compiler-mechanism)",
        "(machine-text projection-only)",
        "(machine-bytes projection-only)",
        "(retired-semantic-id 1153)",
    ] {
        assert!(
            contract.contains(required),
            "machine lowering boundary missing required authority fact: {required}"
        );
    }

    for forbidden in ["rdtsc", "RDTSC", "x86", "rax", "rdx", "0F 31", "0f 31"] {
        assert!(
            !contract.contains(forbidden),
            "target-specific mechanism leaked into semantic/compiler authority boundary: {forbidden}"
        );
    }
}

#[test]
fn repo_exports_machine_lowering_boundary_for_compiler_consumers() {
    assert!(
        REPO_DECLARATION.contains("machine-lowering-boundary"),
        "repo.lisp must export the semantic-to-machine authority boundary"
    );
}

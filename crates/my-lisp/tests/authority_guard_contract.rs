//! RED contract for #115: host code may observe Lisp verdicts, but may not
//! silently establish new normative Lisp meaning outside the Lisp-owned corpus.

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn authority_guard_rejects_host_authored_semantic_expectation_but_allows_mechanism_assertion() {
    let root = repo_root();
    let guard = root.join("scripts/semantic_authority_guard.py");
    let inventory = root.join("tests/authority-inventory.tsv");
    let forbidden = root.join("tests/fixtures/authority-guard/forbidden-semantic.rs");
    let allowed = root.join("tests/fixtures/authority-guard/allowed-mechanism.rs");

    let forbidden_run = Command::new("python3")
        .arg(&guard)
        .arg("--inventory")
        .arg(&inventory)
        .arg("--check-file")
        .arg(&forbidden)
        .output()
        .expect("#115 authority guard must be executable");
    assert!(
        !forbidden_run.status.success(),
        "host-authored semantic expectation must fail closed"
    );
    let forbidden_stderr = String::from_utf8_lossy(&forbidden_run.stderr);
    assert!(
        forbidden_stderr.contains("semantic authority") && forbidden_stderr.contains("#113"),
        "guard must explain the authority violation and point back to Lisp-owned witnesses: {forbidden_stderr}"
    );

    let allowed_run = Command::new("python3")
        .arg(&guard)
        .arg("--inventory")
        .arg(&inventory)
        .arg("--check-file")
        .arg(&allowed)
        .output()
        .expect("#115 authority guard must be executable");
    assert!(
        allowed_run.status.success(),
        "host mechanism assertion must remain legal: {}",
        String::from_utf8_lossy(&allowed_run.stderr)
    );
}

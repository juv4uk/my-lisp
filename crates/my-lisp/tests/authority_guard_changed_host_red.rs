//! RED probe for #115: changed host tests must cross a Lisp-owned authority
//! boundary. CI may transport paths and observe exit status; it may not encode
//! the semantic allow/deny policy in Python, Rust, JS, or shell.

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn pr_workflow_feeds_changed_host_tests_to_lisp_owned_authority_guard() {
    let workflow = fs::read_to_string(repo_root().join(".github/workflows/ci.yml"))
        .expect("CI workflow must exist");

    assert!(
        workflow.contains("authority-guard.lisp")
            && workflow.contains("/tmp/changed.txt")
            && workflow.contains("authority-inventory.tsv")
            && !workflow.contains("semantic_authority_guard.py"),
        "#115 boundary must be: changed paths -> inventory -> Lisp-owned authority verdict; host code is transport only"
    );
}

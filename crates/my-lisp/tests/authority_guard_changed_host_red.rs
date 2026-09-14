//! RED probe for #115: a changed host test must not escape semantic authority
//! merely because the workflow knows that the path changed.
//!
//! The production CI wiring is intentionally absent while this probe is RED.

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn pr_workflow_feeds_changed_host_tests_to_semantic_authority_guard() {
    let workflow = fs::read_to_string(repo_root().join(".github/workflows/ci.yml"))
        .expect("CI workflow must exist");

    assert!(
        workflow.contains("semantic_authority_guard.py")
            && workflow.contains("/tmp/changed.txt")
            && workflow.contains("authority-inventory.tsv"),
        "#115 requires one simple boundary: changed paths -> authority inventory -> semantic authority guard"
    );
}

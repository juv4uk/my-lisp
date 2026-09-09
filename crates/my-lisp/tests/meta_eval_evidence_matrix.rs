use std::process::Command;

#[test]
fn meta_eval_evidence_matrix_bounds_self_hosting_claims() {
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scripts/check-meta-eval-evidence.py"
    );
    let output = Command::new("python3")
        .arg(script)
        .output()
        .expect("python3 must run meta-eval evidence checker");

    assert!(
        output.status.success(),
        "meta-eval evidence checker failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

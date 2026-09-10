use std::process::Command;

fn run_python(script: &str, args: &[&str], label: &str) {
    let output = Command::new("python3")
        .arg(script)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("python3 must run {label}: {error}"));

    assert!(
        output.status.success(),
        "{label} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn meta_eval_evidence_matrix_bounds_self_hosting_claims() {
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scripts/check-meta-eval-evidence.py"
    );
    run_python(script, &[], "meta-eval evidence checker");
}

#[test]
fn human_meta_eval_evidence_is_generated_from_machine_matrix() {
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scripts/generate-meta-eval-evidence.py"
    );
    run_python(
        script,
        &["--check"],
        "meta-eval human evidence projection check",
    );
}

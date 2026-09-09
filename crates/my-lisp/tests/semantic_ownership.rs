use std::process::Command;

#[test]
fn semantic_ownership_map_and_report_stay_in_sync() {
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scripts/semantic-ownership.py"
    );
    let output = Command::new("python3")
        .args([script, "--check"])
        .output()
        .expect("python3 must run semantic ownership checker");

    assert!(
        output.status.success(),
        "semantic ownership checker failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

use std::process::Command;

#[test]
fn generated_uk_surface_inventory_is_current() {
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scripts/generate-uk-surface-audit.py"
    );

    let output = Command::new("python3")
        .arg(script)
        .arg("--check")
        .output()
        .expect("python3 must launch the UK surface audit generator");

    assert!(
        output.status.success(),
        "UK surface audit projection must be current\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

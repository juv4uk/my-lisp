use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn my_lisp(cwd: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_my-lisp"));
    command.current_dir(cwd);
    command
}

fn run_generator(cwd: &Path, script: &Path) -> Output {
    let output = my_lisp(cwd)
        .arg(script)
        .output()
        .expect("constitution generator should run through the real my-lisp CLI");
    assert!(
        output.status.success(),
        "constitution generator failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "my-lisp-{label}-{}-{nonce}",
        std::process::id()
    ))
}

#[test]
fn generator_reproduces_checked_in_constitution_byte_for_byte() {
    let root = repo_root();
    let generated = run_generator(&root, Path::new("scripts/build-constitution.my"));
    let checked_in = fs::read(root.join("my-lisp-constitution.my"))
        .expect("checked-in constitution should be readable");

    assert_eq!(
        generated.stdout, checked_in,
        "my-lisp-constitution.my drifted from the real language-owned generator; edit authority inputs, regenerate, and review the generated diff"
    );
}

#[test]
fn changing_conformance_input_changes_the_generated_projection() {
    let root = repo_root();
    let temp = unique_temp_dir("constitution-source-mutation");
    let fixture_dir = temp.join("tests/fixtures");
    fs::create_dir_all(&fixture_dir).expect("temporary fixture directory should be created");

    let mut conformance = fs::read_to_string(root.join("tests/fixtures/conformance.my"))
        .expect("conformance authority should be readable");
    conformance.push_str(
        "\n((name . \"constitution-projection-probe\") (tier . 3) (expr . \"(quote constitution-projection-probe)\") (expected . \"constitution-projection-probe\"))\n",
    );
    fs::write(fixture_dir.join("conformance.my"), conformance)
        .expect("mutated temporary conformance authority should be writable");

    let generated = run_generator(&temp, &root.join("scripts/build-constitution.my"));
    let checked_in = fs::read(root.join("my-lisp-constitution.my"))
        .expect("checked-in constitution should be readable");
    let stdout = String::from_utf8_lossy(&generated.stdout);

    assert_ne!(
        generated.stdout, checked_in,
        "changing the authoritative fixture input must change the generated projection"
    );
    assert!(
        stdout.contains("constitution-projection-probe"),
        "the changed source fixture must be observable in generated output"
    );

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn hand_editing_only_the_projection_is_detected_by_the_same_byte_check() {
    let root = repo_root();
    let generated = run_generator(&root, Path::new("scripts/build-constitution.my"));
    let mut hand_edited = fs::read(root.join("my-lisp-constitution.my"))
        .expect("checked-in constitution should be readable");
    hand_edited.extend_from_slice(b"; simulated hand edit\n");

    assert_ne!(
        generated.stdout, hand_edited,
        "a hand edit to only the generated projection must fail byte-for-byte integrity"
    );
}

#[test]
fn generator_has_one_external_semantic_input_and_projection_stays_draft() {
    let root = repo_root();
    let script = fs::read_to_string(root.join("scripts/build-constitution.my"))
        .expect("constitution generator should be readable");
    let constitution = fs::read_to_string(root.join("my-lisp-constitution.my"))
        .expect("checked-in constitution should be readable");

    let executable = script
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with(';'))
        .collect::<Vec<_>>();

    assert_eq!(
        executable
            .iter()
            .filter(|line| line.contains("(read-file "))
            .count(),
        1,
        "generator gained another executable external data authority; resolve ownership explicitly before accepting it"
    );
    assert!(
        executable
            .iter()
            .any(|line| line.contains("(read-file \"tests/fixtures/conformance.my\")")),
        "conformance.my must remain the generator's single external semantic data input"
    );
    assert!(
        constitution.contains("(status . \"draft — not yet ratified; will become read-only once ratified\")"),
        "issue #22 must not ratify or promote the generated constitution to semantic authority"
    );
    assert!(
        constitution.contains("GENERATED — do not hand-edit it"),
        "the projection must keep its explicit generated/mutation-path marker"
    );
}

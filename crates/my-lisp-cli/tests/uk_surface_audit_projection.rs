use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn my_lisp(cwd: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_my-lisp"));
    command.current_dir(cwd);
    command
}

fn numeric_row_ids(source: &str) -> BTreeSet<u32> {
    source
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            let rest = line.strip_prefix('(')?;
            let token = rest.split_whitespace().next()?;
            token.parse::<u32>().ok()
        })
        .collect()
}

#[test]
fn uk_surface_audit_generator_runs_through_real_my_lisp_cli() {
    let root = repo_root();
    let script = root.join("scripts/generate-uk-surface-audit.my");
    let output = my_lisp(&root)
        .arg(&script)
        .output()
        .expect("UK surface audit generator should run through the real my-lisp CLI");

    assert!(
        output.status.success(),
        "language-owned UK surface audit generator must run successfully\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn ukrainian_staging_profile_covers_every_function_table_identity() {
    let root = repo_root();
    let function_table = fs::read_to_string(root.join("lib/generated/function-table.wsm"))
        .expect("generated function table must be readable");
    let profile = fs::read_to_string(root.join("lib/surface/український-профіль-джерела.всм"))
        .expect("Ukrainian staging profile must be readable");

    let expected = numeric_row_ids(&function_table);
    let actual = numeric_row_ids(&profile);

    assert_eq!(expected.len(), 161, "function table inventory changed; review UK coverage gate");
    assert_eq!(
        actual, expected,
        "Ukrainian staging must explicitly cover every semantic identity, including compatibility-only rows"
    );
}

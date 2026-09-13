use std::collections::{BTreeMap, BTreeSet};
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

fn numeric_row_id_list(source: &str) -> Vec<u32> {
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

fn numeric_row_ids(source: &str) -> BTreeSet<u32> {
    numeric_row_id_list(source).into_iter().collect()
}

fn staged_uk_names(source: &str) -> BTreeMap<String, Vec<u32>> {
    let mut by_name = BTreeMap::<String, Vec<u32>>::new();

    for line in source.lines() {
        let line = line.trim_start();
        let Some(rest) = line.strip_prefix('(') else {
            continue;
        };
        let Some(token) = rest.split_whitespace().next() else {
            continue;
        };
        let Ok(id) = token.parse::<u32>() else {
            continue;
        };

        let quoted: Vec<&str> = line.split('"').collect();
        if quoted.len() < 4 {
            continue;
        }
        let full_uk_name = quoted[3].to_owned();
        by_name.entry(full_uk_name).or_default().push(id);
    }

    by_name
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

    let expected_rows = numeric_row_id_list(&function_table);
    let actual_rows = numeric_row_id_list(&profile);
    let expected = numeric_row_ids(&function_table);
    let actual = numeric_row_ids(&profile);

    assert_eq!(
        expected_rows.len(),
        expected.len(),
        "function table must not contain duplicate semantic identity rows"
    );
    assert_eq!(
        actual_rows.len(),
        actual.len(),
        "Ukrainian staging must contain exactly one row per semantic identity"
    );
    assert_eq!(
        expected_rows.len(),
        161,
        "function table inventory changed; review UK coverage gate"
    );
    assert_eq!(
        actual_rows.len(),
        161,
        "Ukrainian staging row count must stay exactly aligned with the 161-row function table"
    );
    assert_eq!(
        actual, expected,
        "Ukrainian staging must explicitly cover every semantic identity, including compatibility-only rows"
    );
}

#[test]
fn ukrainian_staging_full_names_are_injective_over_semantic_identities() {
    let root = repo_root();
    let profile = fs::read_to_string(root.join("lib/surface/український-профіль-джерела.всм"))
        .expect("Ukrainian staging profile must be readable");

    let collisions: BTreeMap<_, _> = staged_uk_names(&profile)
        .into_iter()
        .filter(|(_, ids)| ids.len() > 1)
        .collect();

    assert!(
        collisions.is_empty(),
        "each full Ukrainian staging name must identify exactly one semantic identity; collisions: {collisions:?}"
    );
}

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn inventory() -> String {
    fs::read_to_string(repo_root().join("tests/authority-inventory.tsv"))
        .expect("tests/authority-inventory.tsv must exist")
}

fn require_row(contents: &str, path: &str, test: &str, class: &str) {
    let found = contents.lines().skip(1).any(|line| {
        let fields: Vec<_> = line.split('\t').collect();
        fields.len() >= 3 && fields[0] == path && fields[1] == test && fields[2] == class
    });
    assert!(
        found,
        "#231 requires {path}::{test} to be classified as {class} before the semantic fast lane can trust the inventory"
    );
}

#[test]
fn superseded_truthiness_assertions_are_explicitly_classified() {
    let contents = inventory();

    require_row(
        &contents,
        "crates/my-lisp/tests/mccarthy.rs",
        "comparisons_chain_and_promote_exact_inexact_like_arithmetic",
        "legacy-semantic",
    );
    require_row(
        &contents,
        "crates/my-lisp/tests/forward.rs",
        "match_test_condition_succeeds_when_the_expression_is_truthy",
        "legacy-semantic",
    );
    require_row(
        &contents,
        "crates/my-lisp/tests/ukrainian_api_docs.rs",
        "istina_i_khyba_ie_imenamy_tyh_samykh_znachen",
        "legacy-semantic",
    );
}

#[test]
fn mixed_tests_that_embed_old_truth_results_are_not_mislabeled_as_current_authority() {
    let contents = inventory();

    require_row(
        &contents,
        "crates/my-lisp/tests/mccarthy.rs",
        "bare_large_integer_literals_remain_exact",
        "mixed",
    );
}

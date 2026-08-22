//! RELEASE-VERSION-PARITY: every canonical crate must carry the same
//! version, because scripts/release.my bumps them together with one sed
//! and the release tag is a single version for the whole workspace.
//!
//! This test exists because my-lisp-lsp joined the release list late and
//! silently kept 0.1.0 through the l0.25.0 release — drift no other check
//! would have caught. If you add a canonical crate, add its Cargo.toml to
//! scripts/release.my's bump list AND to CRATE_MANIFESTS below.

use std::path::Path;

/// The five canonical crates bumped by scripts/release.my, in the same
/// order the script lists them.
const CRATE_MANIFESTS: &[&str] = &[
    "crates/my-lisp/Cargo.toml",
    "crates/my-lisp-cli/Cargo.toml",
    "crates/my-lisp-literate/Cargo.toml",
    "crates/my-lisp-wasm/Cargo.toml",
    "crates/my-lisp-lsp/Cargo.toml",
    "crates/my-lisp-host/Cargo.toml",
    "crates/my-lisp-semantic/Cargo.toml",
];

fn manifest_version(repo_root: &Path, manifest: &str) -> String {
    let text = std::fs::read_to_string(repo_root.join(manifest))
        .unwrap_or_else(|e| panic!("cannot read {manifest}: {e}"));
    // First `version = "..."` under [package]; these manifests are simple,
    // and the release script edits exactly this line with its sed.
    let section = text.split("[package]").nth(1).expect("[package] section");
    let line = section
        .lines()
        .find(|l| l.trim_start().starts_with("version ="))
        .unwrap_or_else(|| panic!("{manifest}: no package version line"));
    line.trim()
        .strip_prefix("version =")
        .expect("version assignment")
        .trim()
        .trim_matches('"')
        .to_string()
}

#[test]
fn all_canonical_crates_share_one_release_version() {
    // CARGO_MANIFEST_DIR of this test is crates/my-lisp-lsp/tests.
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf();

    let versions: Vec<(String, String)> = CRATE_MANIFESTS
        .iter()
        .map(|m| (m.to_string(), manifest_version(&repo_root, m)))
        .collect();

    let (_, expected) = &versions[0];
    for (manifest, version) in &versions {
        assert_eq!(
            version, expected,
            "version drift: {manifest} is {version}, but {} is {expected} — \
             add it to scripts/release.my's bump list or fix the version",
            versions[0].0
        );
    }
}

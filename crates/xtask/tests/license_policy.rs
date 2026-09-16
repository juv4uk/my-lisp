use std::fs;
use std::path::{Path, PathBuf};

const WORKSPACE_MANIFESTS: [&str; 13] = [
    "crates/my-lisp/Cargo.toml",
    "crates/my-lisp-cli/Cargo.toml",
    "crates/my-lisp-embed/Cargo.toml",
    "crates/my-lisp-literate/Cargo.toml",
    "crates/my-lisp-wasm/Cargo.toml",
    "crates/swarm-node/Cargo.toml",
    "crates/my-lisp-host/Cargo.toml",
    "crates/my-lisp-semantic/Cargo.toml",
    "crates/my-lisp-lsp/Cargo.toml",
    "crates/wsm-guard-core/Cargo.toml",
    "crates/wsm-guard-slice/Cargo.toml",
    "crates/wsm-guard-facts/Cargo.toml",
    "crates/xtask/Cargo.toml",
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask must live at <workspace>/crates/xtask")
        .to_path_buf()
}

#[test]
fn owned_workspace_crates_point_directly_to_the_single_root_license_file() {
    let root = workspace_root();
    let root_license = fs::canonicalize(root.join("LICENSE"))
        .expect("root LICENSE must exist and be canonicalizable");
    let root_manifest = fs::read_to_string(root.join("Cargo.toml"))
        .expect("workspace Cargo.toml must be readable");

    assert!(
        !root_manifest
            .lines()
            .any(|line| line.trim_start().starts_with("license-file") || line.trim_start().starts_with("license =")),
        "license metadata must stay package-local so a license-only change is not classified as a workspace dependency-graph change"
    );

    for relative in WORKSPACE_MANIFESTS {
        let manifest_path = root.join(relative);
        let manifest = fs::read_to_string(&manifest_path)
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));

        assert!(
            manifest.lines().any(|line| line.trim() == "license-file = \"../../LICENSE\""),
            "{relative} must point directly to the single root LICENSE"
        );
        assert!(
            !manifest.contains("license-file.workspace = true"),
            "{relative} must not inherit license metadata through [workspace.package]"
        );
        assert!(
            !manifest.lines().any(|line| line.trim_start().starts_with("license =")),
            "{relative} must not declare a second package license"
        );

        let manifest_dir = manifest_path
            .parent()
            .expect("workspace manifest path must have a parent");
        let resolved_license = fs::canonicalize(manifest_dir.join("../../LICENSE"))
            .unwrap_or_else(|err| panic!("failed to resolve {relative} license-file: {err}"));
        assert_eq!(
            resolved_license, root_license,
            "{relative} must resolve to the one root LICENSE"
        );
    }
}

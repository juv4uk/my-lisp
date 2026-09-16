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
fn owned_workspace_crates_inherit_the_single_root_license_file() {
    let root = workspace_root();
    let root_manifest = fs::read_to_string(root.join("Cargo.toml"))
        .expect("workspace Cargo.toml must be readable");

    assert!(
        root_manifest.contains("[workspace.package]")
            && root_manifest.contains("license-file = \"LICENSE\""),
        "workspace must define the single root LICENSE as inherited package metadata"
    );

    for relative in WORKSPACE_MANIFESTS {
        let manifest = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|err| panic!("failed to read {relative}: {err}"));

        assert!(
            manifest.contains("license-file.workspace = true"),
            "{relative} must inherit the root LICENSE instead of declaring another project license"
        );
        assert!(
            !manifest.lines().any(|line| line.trim_start().starts_with("license =")),
            "{relative} must not declare a second package license"
        );
    }
}

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const LEGACY_COORDINATION_OPS: &[&str] = &[
    "hello",
    "claim",
    "release",
    "complete-task",
    "next-best-action",
    "list-task-state",
    "presence",
    "list-claims",
    "sync-tasks",
    "subscribe",
    "publish",
    "notify",
    "poll",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scan_tree(root: &Path, dir: &Path, hits: &mut BTreeMap<String, Vec<String>>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if rel == "crates/swarm-node" || rel.starts_with("crates/swarm-node/") {
                continue;
            }
            scan_tree(root, &path, hits);
            continue;
        }

        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        // This file is the compatibility implementation being retired, not a
        // caller of itself. The removal gate tracks it separately as physical
        // legacy surface.
        if rel == "crates/my-lisp-cli/src/swarm.rs" {
            continue;
        }

        let executable_surface = matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("rs" | "py" | "sh" | "wsm" | "my")
        );
        if !executable_surface {
            continue;
        }

        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };

        for op in LEGACY_COORDINATION_OPS {
            let needle = format!("(op {op})");
            if source.contains(&needle) {
                hits.entry(rel.clone()).or_default().push((*op).to_string());
            }
        }
    }
}

#[test]
fn legacy_coordination_executable_callers_are_exactly_the_known_compatibility_test() {
    let root = repo_root();
    let mut actual = BTreeMap::new();

    scan_tree(&root, &root.join("crates"), &mut actual);
    scan_tree(&root, &root.join("scripts"), &mut actual);

    for ops in actual.values_mut() {
        ops.sort();
        ops.dedup();
    }

    let expected = BTreeMap::from([(
        "crates/my-lisp-cli/tests/cli.rs".to_string(),
        vec![
            "complete-task".to_string(),
            "next-best-action".to_string(),
            "sync-tasks".to_string(),
        ],
    )]);

    assert_eq!(
        actual, expected,
        "C5 removal gate changed: any new legacy caller is a regression; any removed caller must be reflected in the audit in the same change"
    );
}

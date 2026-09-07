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

fn inventory() -> BTreeMap<String, Vec<String>> {
    let root = repo_root();
    let mut hits = BTreeMap::new();

    scan_tree(&root, &root.join("crates"), &mut hits);
    scan_tree(&root, &root.join("scripts"), &mut hits);

    for ops in hits.values_mut() {
        ops.sort();
        ops.dedup();
    }
    hits
}

fn is_compatibility_test(path: &str) -> bool {
    path.starts_with("crates/") && path.contains("/tests/")
}

#[test]
fn no_production_or_operational_legacy_coordination_callers_remain() {
    let live: BTreeMap<_, _> = inventory()
        .into_iter()
        .filter(|(path, _)| !is_compatibility_test(path))
        .collect();

    assert!(
        live.is_empty(),
        "C5 no-live-callers gate failed; production/operational legacy callers remain: {live:?}"
    );
}

#[test]
fn compatibility_callers_are_exactly_the_known_legacy_cli_regression() {
    let compatibility: BTreeMap<_, _> = inventory()
        .into_iter()
        .filter(|(path, _)| is_compatibility_test(path))
        .collect();

    let expected = BTreeMap::from([(
        "crates/my-lisp-cli/tests/cli.rs".to_string(),
        vec![
            "complete-task".to_string(),
            "next-best-action".to_string(),
            "sync-tasks".to_string(),
        ],
    )]);

    assert_eq!(
        compatibility, expected,
        "C5 compatibility inventory changed: retire removals together with the legacy surface, and reject any new legacy test caller"
    );
}

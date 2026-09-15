//! #182 policy witness for reusable language-contract workflows.
//! This observes transport/provenance configuration only; it does not define
//! any expected Lisp semantic outcome.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_workflow(name: &str) -> Result<String, String> {
    let path = repo_root().join(".github/workflows").join(name);
    std::fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))
}

pub fn verify() -> Result<(), String> {
    let sync = read_workflow("sync-language-contract.yml")?;
    let verify = read_workflow("verify-language-contract.yml")?;
    let mut problems = Vec::new();

    for (name, workflow) in [("sync", sync.as_str()), ("verify", verify.as_str())] {
        for required in [
            "default: contracts/my-lisp/language-contract.lisp",
            "default: contracts/my-lisp/lock.lisp",
            "UPSTREAM_PATH: language-contract.lisp",
        ] {
            if !workflow.contains(required) {
                problems.push(format!("{name} workflow missing canonical path: {required}"));
            }
        }

        for stale in [
            "language-contract.my",
            "contracts/my-lisp/lock.my",
            "/tmp/language-contract.my",
        ] {
            if workflow.contains(stale) {
                problems.push(format!("{name} workflow still contains removed path: {stale}"));
            }
        }
    }

    for required in [
        "commits?sha=${UPSTREAM_BRANCH}&path=${UPSTREAM_PATH}&per_page=1",
        "[[ \"$contract_revision\" =~ ^[0-9a-f]{40}$ ]]",
        "digest=\"$(sha256sum \"$TARGET_PATH\" | awk '{print $1}')\"",
        "cmp --silent \"$TARGET_PATH\" /tmp/language-contract.lisp",
    ] {
        if !sync.contains(required) {
            problems.push(format!("sync workflow lost exact-revision/digest guard: {required}"));
        }
    }

    for required in [
        "[[ \"$revision\" =~ ^[0-9a-f]{40}$ ]]",
        "[[ \"$locked_digest\" =~ ^[0-9a-f]{64}$ ]]",
        "actual_digest=\"$(sha256sum \"$TARGET_PATH\" | awk '{print $1}')\"",
        "${UPSTREAM_REPO}/${revision}/${UPSTREAM_PATH}",
        "cmp --silent \"$TARGET_PATH\" /tmp/language-contract.lisp",
    ] {
        if !verify.contains(required) {
            problems.push(format!("verify workflow lost exact-revision/digest guard: {required}"));
        }
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

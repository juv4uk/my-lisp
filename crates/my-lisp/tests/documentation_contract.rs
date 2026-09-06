fn defined_names(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let rest = trimmed.strip_prefix("(def ")?;
            rest.split_whitespace().next()
        })
        .collect()
}

fn function_reference_section<'a>(reference: &'a str, file: &str) -> &'a str {
    let marker = format!("### {file} (");
    let start = reference
        .find(&marker)
        .unwrap_or_else(|| panic!("FUNCTIONS.md is missing section for {file}"));
    let tail = &reference[start..];
    let end = tail[marker.len()..]
        .find("\n### ")
        .map(|offset| marker.len() + offset)
        .unwrap_or(tail.len());
    &tail[..end]
}

#[test]
fn public_docs_share_current_project_identity_and_extension() {
    let readme = include_str!("../../../README.md");
    let core = include_str!("../../../docs/language-core.md");

    for doc in [readme, core] {
        assert!(
            doc.contains("reference implementation") || doc.contains("референсна реалізація"),
            "public architecture prose must describe Rust as a reference implementation"
        );
        assert!(
            doc.contains("`.wsm`") && doc.contains("`.my`") && doc.contains("`.lisp`"),
            "public architecture prose must state the current extension family"
        );
        assert!(
            !doc.contains("canonical Rust implementation")
                && !doc.contains("канонічна реалізація на Rust")
                && !doc.contains("kanonische Rust-Implementierung"),
            "implementation wording must not imply that Rust itself owns semantics"
        );
    }
}

#[test]
fn public_docs_point_to_semantic_authority_instead_of_inventing_one() {
    let readme = include_str!("../../../README.md");
    let core = include_str!("../../../docs/language-core.md");
    let authority = include_str!("../../../docs/semantic-authority-map.md");
    let authority_lower = authority.to_lowercase();

    assert!(readme.contains("docs/semantic-authority-map.md"));
    assert!(core.contains("semantic-authority-map.md"));
    assert!(authority.contains("language-contract.my"));
    assert!(authority_lower.contains("ratified adr"));
    assert!(authority_lower.contains("executable conformance"));
}

#[test]
fn host_semantic_surface_documentation_tracks_time_ownership() {
    let hss = include_str!("../../../docs/host-semantic-surface.md");
    let time = include_str!("../../../lib/time.my");
    let builtins = include_str!("../src/eval/builtins.rs");

    assert!(hss.contains("mono-ns"));
    assert!(hss.contains("unix-time-now"));
    assert!(hss.contains("`utc-now` | `lib/time.my` | derived public clock meaning | HOST REMOVED"));
    assert!(time.contains("(def mono-ms"));
    assert!(time.contains("(def utc-now"));

    assert!(
        !builtins.contains("fn civil_from_days")
            && !builtins.contains("fn utc_now_value")
            && !builtins.contains("\"utc-now\","),
        "Rust must not regain Gregorian utc-now semantics after the completed migration"
    );
    assert!(builtins.contains("\"unix-time-now\","));
}

#[test]
fn reasoning_function_reference_tracks_live_library_definitions() {
    let reference = include_str!("../../../docs/FUNCTIONS.md");

    for (file, source) in [
        ("result-status.my", include_str!("../../../lib/result-status.my")),
        ("narrate.my", include_str!("../../../lib/narrate.my")),
    ] {
        let names = defined_names(source);
        let section = function_reference_section(reference, file);
        let expected_heading = format!("### {file} ({})", names.len());
        assert!(
            section.starts_with(&expected_heading),
            "FUNCTIONS.md count for {file} is stale: expected {} definitions",
            names.len()
        );
        for name in names {
            assert!(
                section.contains(&format!("`{name}`")),
                "FUNCTIONS.md section for {file} is missing live definition {name}"
            );
        }
    }
}

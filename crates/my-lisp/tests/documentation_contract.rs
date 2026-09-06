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

    assert!(readme.contains("docs/semantic-authority-map.md"));
    assert!(core.contains("semantic-authority-map.md"));
    assert!(authority.contains("language-contract.my"));
    assert!(authority.contains("ratified ADR"));
    assert!(authority.contains("executable conformance"));
}

#[test]
fn host_semantic_surface_documentation_tracks_time_ownership() {
    let hss = include_str!("../../../docs/host-semantic-surface.md");
    let time = include_str!("../../../lib/time.my");

    assert!(hss.contains("mono-ns"));
    assert!(hss.contains("unix-time-now"));
    assert!(hss.contains("utc-now"));
    assert!(time.contains("(def mono-ms"));
    assert!(time.contains("(def utc-now"));
}

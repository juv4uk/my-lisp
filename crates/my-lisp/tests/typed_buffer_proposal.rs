use std::fs;
use std::path::Path;

fn repository_file(name: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("my-lisp crate should live under repository/crates");
    fs::read_to_string(root.join(name)).unwrap()
}

#[test]
fn typed_buffer_proposal_is_parseable_but_not_a_language_claim() {
    let proposal = repository_file("typed-buffer-proposal.my");
    let expressions = my_lisp::parse(&proposal).expect("proposal must remain valid my-lisp data");
    assert_eq!(expressions.len(), 1);
    assert!(proposal.contains("(status . proposed-not-implemented)"));
    assert!(proposal.contains("(current-language-contract . (2 1))"));
    assert!(proposal.contains("(proposed-language-contract . (2 2))"));
    assert!(proposal.contains("(implicit-exact-to-f32 . forbidden)"));

    let language_contract = repository_file("language-contract.my");
    assert!(language_contract.contains("((major . 2) (minor . 1)"));
    assert!(!language_contract.contains("(minor . 2)"));
}

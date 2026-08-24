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
fn typed_buffer_record_matches_ratified_language_contract() {
    let proposal = repository_file("typed-buffer-proposal.my");
    let expressions = my_lisp::parse(&proposal).expect("proposal must remain valid my-lisp data");
    assert_eq!(expressions.len(), 1);
    assert!(proposal.contains("(status . ratified-implemented)"));
    assert!(proposal.contains("(current-language-contract . (2 2))"));
    assert!(proposal.contains("(implicit-exact-to-f32 . forbidden)"));

    let language_contract = repository_file("language-contract.my");
    assert!(language_contract.contains("((major . 3) (minor . 0)"));
}

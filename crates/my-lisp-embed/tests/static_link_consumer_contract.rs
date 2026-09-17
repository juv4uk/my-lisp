use std::path::PathBuf;

#[test]
fn canonical_embed_declares_staticlib_for_native_consumers() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = std::fs::read_to_string(manifest).expect("read my-lisp-embed Cargo.toml");
    assert!(
        text.contains("crate-type = [\"cdylib\", \"staticlib\", \"rlib\"]"),
        "my-lisp-embed must emit staticlib alongside cdylib/rlib for native one-file consumers"
    );
}

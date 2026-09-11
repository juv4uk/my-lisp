//! Executable witness for GitHub issue juv4uk/my-lisp#62 (2026-09-10 owner
//! decision: `.всм`/`.мій`/`.лісп` are equal-standing Ukrainian Cyrillic
//! spellings of `.wsm`/`.my`/`.lisp` — a file-naming surface policy, not a
//! new semantic authority).
//!
//! Proves the real repo file `tests/fixtures/приклад.мій` — genuine UTF-8
//! Cyrillic filename and extension, not a transliterated stand-in — is read
//! and evaluated through the exact same path any `.my` file goes through,
//! with no code change needed on this side: `crates/my-lisp-cli/src/main.rs`
//! calls plain `fs::read_to_string(filename)` with no extension filtering at
//! all, so Unicode paths were never rejected here to begin with. This test
//! makes that fact executable instead of merely asserted.

use my_lisp::{eval_program, Session};
use std::fs;

#[test]
fn cyrillic_named_myi_file_reads_and_evaluates_like_any_my_file() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/приклад.мій"
    );
    let source = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read Cyrillic-named fixture at {path}: {e}"));
    let mut session = Session::default();
    let result = eval_program(&source, &mut session)
        .unwrap_or_else(|e| panic!("Cyrillic-named fixture failed to evaluate: {e}"));
    assert_eq!(result.value.to_string(), "3");
}

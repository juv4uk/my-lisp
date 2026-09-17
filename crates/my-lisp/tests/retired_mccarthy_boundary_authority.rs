//! #295 observer for Lisp-owned preservation of useful retired mccarthy laws.
//! Rust loads the language-owned witness and checks only its named status
//! envelope. Dotted-pair and macro-boundary semantic expectations stay in Lisp.

use my_lisp::{eval_program, load_core_library, Session};

#[test]
fn retired_mccarthy_boundary_laws_are_owned_by_lisp() {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core library");
    eval_program(
        include_str!("../../../tests/fixtures/retired-mccarthy-boundary-witness.lisp"),
        &mut session,
    )
    .expect("retired mccarthy boundary witness must load");

    let verdict = eval_program("(retired-mccarthy-boundary-witness)", &mut session)
        .expect("retired mccarthy boundary witness must execute")
        .value
        .to_string();

    assert!(
        verdict.starts_with("(retired-mccarthy-boundary-witness (status pass)"),
        "Lisp-owned retired-mccarthy witness rejected runtime semantics: {verdict}"
    );
}

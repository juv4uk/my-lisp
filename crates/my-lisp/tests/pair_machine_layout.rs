use my_lisp::{eval_program, load_core_library, Session};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn x86_pair_layout_is_one_lisp_owned_machine_readable_authority() {
    let path = repo_root().join("lib/machine/layout/pair-x86-64.lisp");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", path.display()));

    my_lisp::parse(&source).expect("x86 pair layout authority must be valid my-lisp");
    for required in [
        "(def x86-pair-cell-bytes 16)",
        "(def x86-pair-car-offset 0)",
        "(def x86-pair-cdr-offset 8)",
        "(target x86-64)",
        "(arena-argument-register rdi)",
        "(lifetime native-call)",
        "(escape forbidden)",
    ] {
        assert!(source.contains(required), "pair layout authority missing {required}");
    }

    assert!(
        !source.contains("semantic-id"),
        "machine representation layout must not allocate language semantic identities"
    );

    let mut session = Session::default();
    load_core_library(&mut session).expect("core must bootstrap before layout authority");
    eval_program(&source, &mut session).expect("pair layout authority must evaluate as ordinary my-lisp");

    for (name, expected) in [
        ("x86-pair-cell-bytes", "16"),
        ("x86-pair-car-offset", "0"),
        ("x86-pair-cdr-offset", "8"),
    ] {
        let actual = eval_program(name, &mut session)
            .unwrap_or_else(|error| panic!("{name} must be queryable: {error}"))
            .value
            .to_string();
        assert_eq!(actual, expected);
    }
}

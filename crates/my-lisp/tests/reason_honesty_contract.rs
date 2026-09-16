//! #219 observer for the Lisp-owned reasoning-honesty witness.
//! Rust transports the actual evaluator result; Lisp fixture + witness logic own
//! the semantic expectation.

use my_lisp::{eval_program, parse, ExprKind, Session};
use std::fs;
use std::path::PathBuf;

fn repo_file(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(relative)
}

fn escape_lisp_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[test]
fn absence_of_proof_does_not_fabricate_negation() {
    let source = include_str!("../../../tests/fixtures/reason-honesty-v1.lisp");
    let forms = parse(source).expect("reason-honesty-v1.lisp must parse");
    assert_eq!(forms.len(), 1, "first #219 honesty slice must contain one row");

    let ExprKind::List(entries) = &forms[0].kind else {
        panic!("#219 witness row must be an alist list");
    };
    let expr = entries.iter().find_map(|entry| {
        let ExprKind::Pair(key, value) = &entry.kind else { return None; };
        let ExprKind::Symbol(key) = &key.kind else { return None; };
        if &**key != "expr" { return None; }
        let ExprKind::String(value) = &value.kind else { return None; };
        Some(value.to_string())
    }).expect("#219 row must contain expr");

    let row_source = &source[forms[0].span.start..forms[0].span.end];

    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.lisp"), &mut session).expect("core library");
    eval_program(include_str!("../../../lib/unify.lisp"), &mut session).expect("unify library");
    eval_program(include_str!("../../../lib/reason.lisp"), &mut session).expect("reason library");
    let witness = fs::read_to_string(repo_file("tests/fixtures/witness-runner.lisp"))
        .expect("Lisp-owned witness runner");
    eval_program(&witness, &mut session).expect("witness runner must load");

    let actual = match eval_program(&expr, &mut session) {
        Ok(result) => format!("(value \"{}\")", escape_lisp_string(&result.value.to_string())),
        Err(error) => format!("(error \"{:?}\")", error.kind),
    };
    let verdict = format!(
        "(witness-status (witness-verdict (quote {}) (quote {})))",
        row_source, actual
    );
    let status = eval_program(&verdict, &mut session)
        .expect("#219 Lisp-owned witness verdict")
        .value
        .to_string();

    assert_eq!(
        status, "pass",
        "#219 no-proof-is-not-negation witness rejected actual outcome for {expr}: {actual}"
    );
}

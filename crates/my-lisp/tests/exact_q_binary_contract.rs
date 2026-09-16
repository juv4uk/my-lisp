//! #216 observer for the Lisp-owned exact-Q binary contract.
//! Rust transports contract bytes and checks migration bookkeeping only.

use std::fs;
use std::path::PathBuf;

use my_lisp::{eval_program, load_core_library, parse, Expr, ExprKind, Session};

#[derive(Clone)]
struct Row {
    expr: String,
    expected: String,
    blocked_by: Option<String>,
}

fn repo_file(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn alist_str<'a>(entries: &'a [Expr], key: &str) -> Option<&'a str> {
    entries.iter().find_map(|entry| {
        let ExprKind::Pair(k, v) = &entry.kind else {
            return None;
        };
        let ExprKind::Symbol(name) = &k.kind else {
            return None;
        };
        if &**name != key {
            return None;
        }
        match &v.kind {
            ExprKind::String(value) => Some(value.as_ref()),
            _ => None,
        }
    })
}

fn alist_symbol<'a>(entries: &'a [Expr], key: &str) -> Option<&'a str> {
    entries.iter().find_map(|entry| {
        let ExprKind::Pair(k, v) = &entry.kind else {
            return None;
        };
        let ExprKind::Symbol(name) = &k.kind else {
            return None;
        };
        if &**name != key {
            return None;
        }
        match &v.kind {
            ExprKind::Symbol(value) => Some(value.as_ref()),
            _ => None,
        }
    })
}

fn rows() -> Vec<Row> {
    let source = include_str!("../../../tests/fixtures/exact-q-binary-v1.lisp");
    parse(source)
        .expect("exact-q-binary-v1.lisp must parse")
        .into_iter()
        .filter_map(|form| {
            let ExprKind::List(entries) = &form.kind else {
                return None;
            };
            Some(Row {
                expr: alist_str(entries, "expr")?.to_string(),
                expected: alist_str(entries, "expected")?.to_string(),
                blocked_by: alist_symbol(entries, "blocked-by").map(str::to_string),
            })
        })
        .collect()
}

fn transport_contract(session: &mut Session) {
    let source = fs::read_to_string(repo_file("contracts/exact-q-binary-contract.lisp"))
        .expect("#216 exact-Q binary contract");
    let forms = parse(&source).expect("exact-Q binary contract must parse");
    assert_eq!(forms.len(), 1, "#216 contract must remain one Lisp data document");
    let form = &forms[0];
    let exact = &source[form.span.start..form.span.end];
    eval_program(
        &format!("(def exact-q-binary-document (quote {exact}))"),
        session,
    )
    .expect("transport exact-Q binary contract");
}

#[test]
fn lisp_owned_exact_q_binary_contract_is_self_consistent() {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core library");
    transport_contract(&mut session);
    let witness = fs::read_to_string(repo_file(
        "tests/fixtures/exact-q-binary-contract-witness.lisp",
    ))
    .expect("#216 exact-Q contract witness");
    eval_program(&witness, &mut session).expect("exact-Q contract witness must load");
    let verdict = eval_program("(exact-q-binary-contract-witness)", &mut session)
        .expect("exact-Q contract witness must execute")
        .value
        .to_string();
    assert!(
        verdict.starts_with("(exact-q-binary-contract-witness (status pass)"),
        "Lisp-owned exact-Q contract rejected itself: {verdict}"
    );
}

#[test]
fn proven_exact_q_runtime_targets_stay_blocked_only_by_control_migration() {
    let rows = rows();
    assert_eq!(rows.len(), 5, "#216 first slice must retain all five exact-Q targets");
    assert!(
        rows.iter()
            .all(|row| row.blocked_by.as_deref() == Some("control-logic-217")),
        "every exact-Q result target must name #217 as its sole current blocker"
    );
    assert!(
        rows.iter().any(|row| row.expected == "0"),
        "#216 blocked target corpus must retain an exact mathematical NO (0/1)"
    );
    assert!(
        rows.iter().any(|row| row.expected == "1"),
        "#216 blocked target corpus must retain an exact mathematical YES (1/1)"
    );
    assert!(
        rows.iter().all(|row| !row.expr.trim().is_empty()),
        "#216 blocked targets must remain executable expressions, not prose-only debt"
    );
}

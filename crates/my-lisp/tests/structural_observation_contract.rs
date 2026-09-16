//! #218 observer for the Lisp-owned PRIM_ATOM / PRIM_EQ result contract.
//! Rust transports contract bytes and runtime actuals only.

use std::fs;
use std::path::PathBuf;

use my_lisp::{eval_program, load_core_library, parse, Expr, ExprKind, Session};

#[derive(Clone)]
struct Row {
    source: String,
    expr: String,
    expected: Option<String>,
    error: Option<String>,
    active: bool,
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

fn alist_true(entries: &[Expr], key: &str) -> bool {
    alist_symbol(entries, key) == Some("t")
}

fn rows() -> Vec<Row> {
    let source = include_str!("../../../tests/fixtures/structural-observation-v1.lisp");
    parse(source)
        .expect("structural-observation-v1.lisp must parse")
        .into_iter()
        .filter_map(|form| {
            let ExprKind::List(entries) = &form.kind else {
                return None;
            };
            Some(Row {
                source: source[form.span.start..form.span.end].to_string(),
                expr: alist_str(entries, "expr")?.to_string(),
                expected: alist_str(entries, "expected").map(str::to_string),
                error: alist_str(entries, "error").map(str::to_string),
                active: alist_true(entries, "active"),
                blocked_by: alist_symbol(entries, "blocked-by").map(str::to_string),
            })
        })
        .collect()
}

fn escape_lisp_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn load_witness_runner(session: &mut Session) {
    let source = fs::read_to_string(repo_file("tests/fixtures/witness-runner.lisp"))
        .expect("Lisp-owned witness runner");
    eval_program(&source, session).expect("witness-runner.lisp must load");
}

fn actual(row: &Row, session: &mut Session) -> String {
    match eval_program(&row.expr, session) {
        Ok(result) => format!("(value \"{}\")", escape_lisp_string(&result.value.to_string())),
        Err(error) => format!("(error \"{:?}\")", error.kind),
    }
}

fn assert_lisp_verdict(session: &mut Session, row: &Row, actual: &str) {
    let program = format!(
        "(witness-pass? (witness-verdict (quote {}) (quote {})))",
        row.source, actual
    );
    let verdict = eval_program(&program, session)
        .unwrap_or_else(|error| panic!("Lisp verdict failed for {}: {error}", row.expr))
        .value
        .to_string();
    assert_eq!(
        verdict, "t",
        "Lisp-owned #218 witness rejected runtime actual for {} (expected={:?}, error={:?}, actual={actual})",
        row.expr, row.expected, row.error
    );
}

fn transport_contract(session: &mut Session) {
    let source = fs::read_to_string(repo_file("contracts/structural-observation-contract.lisp"))
        .expect("#218 structural observation contract");
    let forms = parse(&source).expect("structural observation contract must parse");
    assert_eq!(forms.len(), 1, "#218 contract must remain one Lisp data document");
    let form = &forms[0];
    let exact = &source[form.span.start..form.span.end];
    eval_program(
        &format!("(def structural-observation-document (quote {exact}))"),
        session,
    )
    .expect("transport structural observation contract");
}

#[test]
fn lisp_owned_structural_observation_contract_is_self_consistent() {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core library");
    transport_contract(&mut session);
    let witness = fs::read_to_string(repo_file(
        "tests/fixtures/structural-observation-contract-witness.lisp",
    ))
    .expect("#218 structural contract witness");
    eval_program(&witness, &mut session).expect("structural contract witness must load");
    let verdict = eval_program("(structural-observation-contract-witness)", &mut session)
        .expect("structural contract witness must execute")
        .value
        .to_string();
    assert!(
        verdict.starts_with("(structural-observation-contract-witness (status pass)"),
        "Lisp-owned structural observation contract rejected itself: {verdict}"
    );
}

#[test]
fn superseded_value_results_stay_explicitly_blocked_only_by_control_migration() {
    let rows = rows();
    assert_eq!(rows.len(), 6, "#218 target must retain all six structural rows");

    let blocked: Vec<_> = rows.iter().filter(|row| !row.active).collect();
    assert_eq!(blocked.len(), 5, "exactly five new value-result rows stay blocked after RED");
    assert!(
        blocked
            .iter()
            .all(|row| row.blocked_by.as_deref() == Some("control-logic-217")),
        "every deferred atom/eq value-result row must name #217 as its blocker"
    );

    let active: Vec<_> = rows.iter().filter(|row| row.active).collect();
    assert_eq!(active.len(), 1, "only the already-valid eq Type-domain row stays active");
    assert_eq!(active[0].error.as_deref(), Some("Type"));
}

#[test]
fn active_runtime_rows_match_lisp_owned_structural_observation_results() {
    let rows: Vec<_> = rows().into_iter().filter(|row| row.active).collect();
    assert!(!rows.is_empty(), "#218 must retain at least one live runtime row");

    let mut session = Session::default();
    load_core_library(&mut session).expect("core library");
    load_witness_runner(&mut session);

    for row in &rows {
        let runtime_actual = actual(row, &mut session);
        assert_lisp_verdict(&mut session, row, &runtime_actual);
    }
}

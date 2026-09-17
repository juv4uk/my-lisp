//! #216 observer for the current Lisp-owned exact-Q binary decision contract.
//! Rust executes expressions and transports actual outcomes only; expected
//! semantic results remain in tests/fixtures/binary-math-v2-witness.lisp.

use std::fs;
use std::path::PathBuf;

use my_lisp::{eval_program, load_core_library, parse, Expr, ExprKind, Session};

#[derive(Clone)]
struct Row {
    source: String,
    expr: String,
    active: bool,
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
    let source = include_str!("../../../tests/fixtures/binary-math-v2-witness.lisp");
    parse(source)
        .expect("binary-math-v2-witness.lisp must parse")
        .into_iter()
        .filter_map(|form| {
            let ExprKind::List(entries) = &form.kind else {
                return None;
            };
            Some(Row {
                source: source[form.span.start..form.span.end].to_string(),
                expr: alist_str(entries, "expr")?.to_string(),
                active: alist_symbol(entries, "active") == Some("t"),
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

#[test]
fn preserved_exact_q_rows_are_all_active_on_current_control_semantics() {
    let rows = rows();
    assert_eq!(rows.len(), 7, "#216 replay must preserve all seven valuable rows");
    assert!(
        rows.iter().all(|row| row.active),
        "#217 explicit control has landed; no #216 row should remain blocked"
    );
}

#[test]
fn exact_q_binary_runtime_matches_lisp_owned_current_witnesses() {
    let rows: Vec<_> = rows().into_iter().filter(|row| row.active).collect();
    let mut session = Session::default();
    load_core_library(&mut session).expect("core library");
    load_witness_runner(&mut session);

    for row in &rows {
        let runtime_actual = actual(row, &mut session);
        let program = format!(
            "(witness-status (witness-verdict (quote {}) (quote {})))",
            row.source, runtime_actual
        );
        let status = eval_program(&program, &mut session)
            .unwrap_or_else(|error| panic!("#216 Lisp verdict failed for {}: {error}", row.expr))
            .value
            .to_string();
        assert_eq!(
            status, "pass",
            "Lisp-owned #216 witness rejected runtime actual for {}: {}",
            row.expr, runtime_actual
        );
    }
}

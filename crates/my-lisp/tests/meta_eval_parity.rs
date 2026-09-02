//! First Rust <-> WSM parity witness for `lib/meta-eval.my` — part of the
//! WSM self-hosting migration strategy: WSM takes over one semantic
//! capability at a time, Rust stays alongside as the oracle, nothing here
//! is deleted or rewritten. `my-eval` (a WSM function, defined in
//! `lib/meta-eval.my`, already interpreting WSM source using WSM's own
//! primitives) is checked here against the same, independently-authored
//! fixtures `tests/fixtures/conformance.my` already uses to hold the
//! canonical Rust implementation itself accountable — not against
//! hand-picked examples invented for this test, which would just be the
//! implementer confirming its own expectation (see docs/policy's
//! agent-testing-epistemic-authority note on why an implementation must
//! not be its own oracle).
//!
//! Перший Rust<->WSM parity-свідок для `lib/meta-eval.my`: `my-eval`
//! (WSM-функція, що вже інтерпретує WSM-код власними примітивами)
//! звіряється тут із тими самими незалежними fixtures, якими вже тримають
//! підзвітною саму Rust-реалізацію — Rust лишається оракулом, нічого не
//! видалено й не переписано.

use my_lisp::{eval_program, parse, Expr, ExprKind, Session};

/// Looks up `key` in a my-lisp alist `((k1 . v1) (k2 . v2) ...)`, already
/// parsed as `Expr`s — copied from `mccarthy.rs`'s private helper of the
/// same name since integration test files can't share private items.
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
            ExprKind::String(s) => Some(s.as_ref()),
            _ => None,
        }
    })
}

/// Evaluate `expr` through the canonical native Rust evaluator — the
/// oracle this witness checks `my-eval` against.
fn eval_native(expr: &str) -> String {
    let mut session = Session::default();
    eval_program(expr, &mut session)
        .unwrap_or_else(|e| panic!("native (oracle) eval failed: {e}\nexpr: {expr}"))
        .value
        .to_string()
}

/// Evaluate `expr` through `my-eval`, the metacircular evaluator written in
/// WSM itself (`lib/meta-eval.my`) — same session setup as
/// `crates/my-lisp/tests/meta_eval.rs`'s `eval_meta`, with an empty starting
/// environment (`(quote ())`), since every selected expression here is
/// self-contained.
fn eval_via_meta_eval(expr: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session)
        .expect("lib/core.my should load (meta-eval.my needs second/third)");
    eval_program(include_str!("../../../lib/meta-eval.my"), &mut session)
        .expect("lib/meta-eval.my should load");
    let source = format!(
        r#"(my-eval (read "{}") (quote ()))"#,
        expr.replace('\\', "\\\\").replace('"', "\\\"")
    );
    eval_program(&source, &mut session)
        .unwrap_or_else(|e| panic!("my-eval failed: {e}\nexpr: {expr}"))
        .value
        .to_string()
}

/// Curated tier-1 subset of `tests/fixtures/conformance.my` — every entry
/// here is an *unmodified* `expr` string copied from that file (verified
/// below, not just claimed), selected because it stays entirely inside
/// `my-eval`'s current dispatch: `quote`/`cond`/`atom`/`eq`/`car`/`cdr`/
/// `cons` (see `lib/meta-eval.my`'s `my-eval` cond-branch list — read from
/// the code, not the file's own header comment, which is stale on a couple
/// of points as of this witness).
///
/// Deliberately excluded, and why — these are real, current capability
/// gaps in `my-eval`, not oversights in this selection:
/// - anything tagged `error` in conformance.my: `my-eval`'s `env-lookup`
///   returns an unbound symbol *as itself* rather than raising
///   `UnknownSymbol` (documented in `meta-eval.my`'s own header — this is
///   what lets bare numbers/strings self-evaluate without a
///   `symbolp?`/`numberp?` primitive), and none of `my-eval`'s dispatch
///   branches check arity. Reproducing Rust's error-kind semantics is a
///   later capability, not this one.
/// - variadic/dotted lambda-lists and `defmacro` with a bare-symbol
///   parameter list: `bind-params` only walks a proper list of fixed
///   params (`(atom params) -> env` returns immediately for a bare-symbol
///   or dotted params list without binding anything).
/// - `=`: `my-eval` only dispatches `+`/`-`/`*`; no comparison operators.
/// - `let` / builtin-shadowing (contract 2.1): not modeled by `my-eval`'s
///   plain alist environment.
const IN_SCOPE_EXPRS: &[&str] = &[
    "(quote radio)",
    "(atom (quote radio))",
    "(atom (quote ()))",
    "(atom (quote (radio antenna)))",
    "(eq (quote radio) (quote radio))",
    "(eq (quote radio) (quote antenna))",
    "(car (quote (radio antenna)))",
    "(cdr (quote (radio antenna)))",
    "(cons (quote radio) (quote (antenna)))",
    "(cond (() (quote wrong)) (t (quote right)))",
    "(eq 3 3)",
    "(eq 3 4)",
    "(eq \"radio\" \"radio\")",
    "(eq \"\\r\" \"r\")",
    "(cond (() (quote first)) (() (quote second)) (t (quote third)))",
    "(car (quote (a b . c)))",
    "(cdr (cdr (quote (a b . c))))",
    "(cond (0 (quote truthy)) (t (quote falsy)))",
    "(eq 3 3.0)",
    "(eq 3.0 3.0)",
    "(cond (0 (quote zero-is-truthy)) (t (quote wrong)))",
];

/// Guards `IN_SCOPE_EXPRS` against silent drift: this witness only means
/// anything as long as every expression it checks is still a real,
/// independently-authored fixture in `tests/fixtures/conformance.my`, not
/// a hand-picked example invented for this test file.
#[test]
fn every_selected_expression_is_a_real_conformance_fixture() {
    let forms = parse(include_str!("../../../tests/fixtures/conformance.my"))
        .expect("conformance.my should parse as valid my-lisp source");
    let fixture_exprs: Vec<&str> = forms
        .iter()
        .filter_map(|form| {
            let ExprKind::List(entries) = &form.kind else {
                return None;
            };
            alist_str(entries, "expr")
        })
        .collect();
    for expr in IN_SCOPE_EXPRS {
        assert!(
            fixture_exprs.contains(expr),
            "selected expression is not (or no longer) present in tests/fixtures/conformance.my: {expr}"
        );
    }
}

/// The actual parity witness: `my-eval` (WSM interpreting WSM) reproduces
/// the canonical Rust evaluator's result on every selected fixture,
/// bit-for-bit on the printed value. Rust stays the oracle here — this
/// test does not touch, replace, or reduce authority of anything in
/// `crates/my-lisp/src`.
#[test]
fn my_eval_reproduces_the_native_evaluator_on_every_selected_tier1_fixture() {
    for expr in IN_SCOPE_EXPRS {
        let native = eval_native(expr);
        let via_meta_eval = eval_via_meta_eval(expr);
        assert_eq!(
            via_meta_eval, native,
            "Rust<->WSM parity FAILED for {expr}: native (oracle) = {native:?}, my-eval = {via_meta_eval:?}"
        );
    }
}

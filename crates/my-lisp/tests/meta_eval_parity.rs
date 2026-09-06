//! Rust <-> WSM parity witness for `lib/meta-eval.my` — part of the
//! self-hosting migration strategy: WSM takes over one semantic capability
//! at a time, Rust stays alongside as the oracle, nothing here is deleted or
//! rewritten. `my-eval` (a WSM function defined in `lib/meta-eval.my`) is
//! checked against the same independently-authored fixtures that already
//! hold the canonical Rust evaluator accountable.

use my_lisp::{eval_program, parse, Expr, ExprKind, Session};

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

fn eval_native(expr: &str) -> String {
    let mut session = Session::default();
    eval_program(expr, &mut session)
        .unwrap_or_else(|e| panic!("native (oracle) eval failed: {e}\nexpr: {expr}"))
        .value
        .to_string()
}

fn eval_via_meta_eval(expr: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session)
        .expect("lib/core.my should load (meta-eval.my needs core helpers)");
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

/// Every entry is an unmodified `expr` from
/// `tests/fixtures/conformance.my`. The first group covers the original
/// metacircular primitive/cond witness. The final five are the first
/// deliberate tier-2 expansion: chained numeric `<`, `=`, `>` semantics are
/// now owned by `meta-eval.my` itself, using only binary native comparison as
/// substrate.
///
/// Deliberately still excluded:
/// - error fixtures: `my-eval` does not yet reproduce native named failures;
/// - variadic/dotted lambda parameter semantics;
/// - `let` / builtin-shadowing parity;
/// - `<=` / `>=`: these are derived in `lib/core.my`, not yet imported into
///   the explicit metacircular environment.
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
    "(< 1 2 3)",
    "(< 1 3 2)",
    "(> 3 2 1)",
    "(= 1 1 1)",
    "(= 1/2 0.5)",
];

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

#[test]
fn my_eval_reproduces_native_evaluator_on_every_selected_fixture() {
    for expr in IN_SCOPE_EXPRS {
        let native = eval_native(expr);
        let via_meta_eval = eval_via_meta_eval(expr);
        assert_eq!(
            via_meta_eval, native,
            "Rust<->WSM parity FAILED for {expr}: native (oracle) = {native:?}, my-eval = {via_meta_eval:?}"
        );
    }
}

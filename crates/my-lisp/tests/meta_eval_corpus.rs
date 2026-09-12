//! Corpus-driven witness for `lib/meta-eval.my` — part of the self-hosting
//! migration strategy (see PLAN.md, Krok 9). Replaces `meta_eval_parity.rs`'s
//! hand-copied `IN_SCOPE_EXPRS` array (25 expressions typed by hand, then
//! re-verified against `tests/fixtures/conformance.my` in a second test just
//! to confirm the hand-copy was accurate) with a runner that reads the
//! corpus directly.
//!
//! Filters `tests/fixtures/conformance.my` down to fixtures carrying the
//! `(meta-eval . t)` tag (see `tests/fixtures/README.md`) and, for each one,
//! checks native evaluation and `my-eval` (the metacircular evaluator
//! defined in `lib/meta-eval.my`) *independently* against the fixture's own
//! `expected`/`error` field. Per this project's core principle — the corpus
//! is the oracle, never one backend vs another (see
//! `tests/fixtures/README.md`) — native and meta are never compared to each
//! other here: a bug shared by both must not stay hidden behind a
//! native-vs-meta parity check that stays green while both are wrong.

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

/// True when `key` is present with the bare symbol `t` as its value — the
/// flat-boolean tag shape this file's `meta-eval` tag and `compiler-corpus`
/// already use (see `tests/fixtures/README.md`).
fn alist_flag(entries: &[Expr], key: &str) -> bool {
    entries.iter().any(|entry| {
        let ExprKind::Pair(k, v) = &entry.kind else {
            return false;
        };
        let ExprKind::Symbol(name) = &k.kind else {
            return false;
        };
        if &**name != key {
            return false;
        }
        matches!(&v.kind, ExprKind::Symbol(s) if &**s == "t")
    })
}

fn meta_eval_tagged_fixtures() -> Vec<(String, Option<String>, Option<String>)> {
    let forms = parse(include_str!("../../../tests/fixtures/conformance.my"))
        .expect("conformance.my should parse as valid my-lisp source");

    forms
        .iter()
        .filter_map(|form| {
            let ExprKind::List(entries) = &form.kind else {
                panic!("each top-level form in conformance.my should be an alist: {form:?}");
            };
            if !alist_flag(entries, "meta-eval") {
                return None;
            }
            let expr = alist_str(entries, "expr")
                .expect("meta-eval-tagged fixture needs an \"expr\" string")
                .to_string();
            let expected = alist_str(entries, "expected").map(str::to_string);
            let error = alist_str(entries, "error").map(str::to_string);
            Some((expr, expected, error))
        })
        .collect()
}

/// Evaluates `expr` through the metacircular evaluator using the session's
/// running `--meta-env--` binding as the environment, then threads the
/// updated environment `my-eval-program` returns back into that same
/// binding — giving the meta side the same one-shared-session-in-corpus-order
/// semantics `tests/fixtures/README.md` specifies, matching how the native
/// runner reuses one `Session` across all tagged fixtures. `my-eval-program`
/// (not a bare `my-eval` call against a throwaway `'()` environment) is what
/// makes this threading possible: for a single-form list it returns exactly
/// `(updated-env . value)`, the same shape `my-eval-top-form` produces.
fn eval_via_meta(session: &mut Session, expr: &str) -> Result<String, String> {
    // Three sequential top-level forms, not one `let` with a multi-form
    // body: this codebase's `let` macro takes exactly one body expression
    // (`(defmacro let (bindings body) ...)` in lib/core.my), so threading
    // `--meta-env--` forward needs its own top-level `def` step.
    let escaped = expr.replace('\\', "\\\\").replace('"', "\\\"");
    eval_program(
        &format!(r#"(def --meta-eval-step-- (my-eval-program (list (read "{escaped}")) --meta-env--))"#),
        session,
    )
    .map_err(|e| format!("{:?}", e.kind))?;
    eval_program(
        "(def --meta-env-- (car --meta-eval-step--))",
        session,
    )
    .map_err(|e| format!("{:?}", e.kind))?;
    eval_program("(cdr --meta-eval-step--)", session)
        .map(|r| r.value.to_string())
        .map_err(|e| format!("{:?}", e.kind))
}

#[test]
fn at_least_the_original_twenty_five_hand_picked_expressions_are_tagged() {
    let fixtures = meta_eval_tagged_fixtures();
    assert!(
        fixtures.len() >= 25,
        "expected at least 25 (meta-eval . t)-tagged fixtures in conformance.my, found {}",
        fixtures.len()
    );
}

/// Native evaluation, checked against each fixture's own `expected`/`error`
/// — never against `my-eval`'s output. Uses one shared session across all
/// tagged fixtures, in corpus order, per this file's shared-session rule.
#[test]
fn native_evaluator_matches_corpus_expected_on_every_meta_eval_tagged_fixture() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session)
        .expect("lib/core.my should load");

    for (expr, expected, error) in meta_eval_tagged_fixtures() {
        if let Some(expected_error) = error {
            let err = eval_program(&expr, &mut session)
                .expect_err(&format!("expected an error but evaluation succeeded: {expr}"));
            assert_eq!(
                format!("{:?}", err.kind),
                expected_error,
                "native: wrong error kind for expression: {expr}"
            );
        } else {
            let expected = expected.expect("fixture needs \"expected\" or \"error\"");
            let actual = eval_program(&expr, &mut session)
                .unwrap_or_else(|e| panic!("native fixture failed: {e}\nexpr: {expr}"))
                .value
                .to_string();
            assert_eq!(actual, expected, "native: failed on expression: {expr}");
        }
    }
}

/// `my-eval`, checked against each fixture's own `expected`/`error` — never
/// against native's output. One shared meta-evaluator environment
/// (`--meta-env--`) is threaded across all tagged fixtures in corpus order,
/// matching the native runner's one-shared-session rule from
/// `tests/fixtures/README.md`: a later fixture is free to reference a `def`
/// an earlier tagged fixture introduced, exactly as it could against the
/// native evaluator's shared session.
#[test]
fn my_eval_matches_corpus_expected_on_every_meta_eval_tagged_fixture() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session)
        .expect("lib/core.my should load (meta-eval.my needs core helpers)");
    my_lisp::load_meta_evaluator_library(&mut session)
        .expect("lib/meta-eval.my should load");
    eval_program("(def --meta-env-- (quote ()))", &mut session)
        .expect("--meta-env-- should initialize to the empty environment");

    for (expr, expected, error) in meta_eval_tagged_fixtures() {
        if error.is_some() {
            // my-eval does not yet reproduce native named failures for the
            // currently-tagged fixtures (none of the 25 original in-scope
            // expressions are error fixtures) — nothing to assert here yet.
            // If an error fixture is ever tagged, this branch must be
            // implemented against my-eval's own structured error shape
            // (see meta_eval_errors.rs) rather than skipped silently.
            panic!("meta-eval tag applied to an error fixture with no my-eval error-shape check implemented yet: {expr}");
        }

        let expected = expected.expect("fixture needs \"expected\" or \"error\"");
        let actual = eval_via_meta(&mut session, &expr)
            .unwrap_or_else(|e| panic!("my-eval failed: {e}\nexpr: {expr}"));
        assert_eq!(actual, expected, "my-eval: failed on expression: {expr}");
    }
}

//! Backend-neutral semantic oracle, first concrete instance — per the
//! 2026-09-11 course correction: the goal is not "less Rust," it's
//! "my-lisp is the semantic authority regardless of which engine
//! executes it." The acceptance test for that claim: can the SAME
//! semantic corpus (#67's frozen `tests/fixtures/conformance.my`
//! `(compiler-corpus . t)` fixtures) pass against more than one
//! independent implementation, with no fixture rewritten per backend?
//!
//! This is not a new mechanism — it reuses `crates/my-lisp/tests/meta_eval_parity.rs`'s
//! own `eval_native`/`eval_via_meta_eval` pattern (native Rust evaluator
//! vs. `lib/meta-eval.my`'s Lisp-owned meta-evaluator), applied
//! specifically to the compiler-relevant corpus rather than the general
//! self-hosting-tracking corpus those two files already cover. The
//! point is not duplicating that infrastructure — it's making explicit,
//! for the corpus a future compiler backend must match, exactly which
//! fixtures a SECOND real implementation (not just the native Rust one)
//! already agrees on, and which do not, honestly, rather than assuming
//! coverage that was never checked.

use my_lisp::{eval_program, load_core_library, load_meta_evaluator_library, parse, Expr, ExprKind, Session};

fn alist_bool_true(entries: &[Expr], key: &str) -> bool {
    entries.iter().any(|entry| {
        let ExprKind::Pair(k, v) = &entry.kind else {
            return false;
        };
        let ExprKind::Symbol(name) = &k.kind else {
            return false;
        };
        &**name == key && matches!(&v.kind, ExprKind::Symbol(s) if &**s == "t")
    })
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
            ExprKind::String(s) => Some(s.as_ref()),
            _ => None,
        }
    })
}

fn compiler_corpus_exprs() -> Vec<String> {
    let source = include_str!("../../../tests/fixtures/conformance.my");
    let forms = parse(source).expect("conformance.my should parse");
    forms
        .iter()
        .filter_map(|form| {
            let ExprKind::List(entries) = &form.kind else {
                return None;
            };
            if !alist_bool_true(entries, "compiler-corpus") {
                return None;
            }
            // Only fixtures with an `expected` value (success fixtures)
            // are meaningful for a value-parity check; error fixtures
            // are a different question (does the second backend raise
            // the same named failure), not attempted here.
            alist_str(entries, "expected")?;
            Some(alist_str(entries, "expr")?.to_string())
        })
        .collect()
}

fn eval_native(expr: &str) -> Result<String, String> {
    let mut session = Session::default();
    load_core_library(&mut session).expect("lib/core.my should load (real CLI always loads it)");
    eval_program(expr, &mut session)
        .map(|r| r.value.to_string())
        .map_err(|e| e.to_string())
}

fn eval_via_meta_eval(expr: &str) -> Result<String, String> {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session)
        .expect("lib/core.my should load (meta-eval.my needs core helpers)");
    load_meta_evaluator_library(&mut session).expect("lib/meta-eval.my should load");
    let source = format!(
        r#"(my-eval (read "{}") (quote ()))"#,
        expr.replace('\\', "\\\\").replace('"', "\\\"")
    );
    eval_program(&source, &mut session)
        .map(|r| r.value.to_string())
        .map_err(|e| e.to_string())
}

/// Not a pass/fail test by itself -- a report. Some compiler-corpus
/// fixtures use forms `my-eval` does not yet cover (multi-statement
/// `def` + call, `defmacro`, dotted/variadic lambda lists); failing
/// those here would conflate "meta-eval.my doesn't cover this yet"
/// with "the backends disagree," which are different findings. Real
/// disagreements (both backends ran, results differ) are hard
/// failures; "meta-eval.my errored where native succeeded" is recorded
/// as a coverage gap instead.
#[test]
fn compiler_corpus_dual_backend_parity_report() {
    let exprs = compiler_corpus_exprs();
    assert!(!exprs.is_empty(), "no compiler-corpus fixtures found");

    let mut agree = Vec::new();
    let mut coverage_gap = Vec::new();
    let mut disagreements = Vec::new();

    for expr in &exprs {
        let native = eval_native(expr);
        let meta = eval_via_meta_eval(expr);
        match (native, meta) {
            (Ok(n), Ok(m)) if n == m => agree.push(expr.clone()),
            // `my-eval` (lib/meta-eval.my) represents its own failures as
            // ORDINARY LISP DATA -- a value shaped like `(error
            // unbound-symbol NAME)` -- not a Rust `Result::Err` (matches
            // docs/meta-eval-evidence.md's documented UnknownSymbol<->
            // unbound-symbol correspondence). `eval_program` on the
            // wrapping `(my-eval ...)` call itself still returns `Ok`
            // even when my-eval semantically couldn't dispatch the form,
            // so this case must be detected by inspecting the *value*,
            // not the `Result` variant. When native succeeded but
            // meta-eval's result is this specific error-data shape, that
            // is a coverage gap (my-eval hasn't been taught this
            // primitive/form yet), not a genuine semantic disagreement --
            // found live while writing this test, not assumed.
            (Ok(_), Ok(m)) if m.starts_with("(error unbound-symbol") => {
                coverage_gap.push(expr.clone())
            }
            (Ok(n), Ok(m)) => disagreements.push(format!(
                "DISAGREEMENT for {expr:?}: native={n:?}, meta-eval={m:?}"
            )),
            (Ok(_), Err(_)) => coverage_gap.push(expr.clone()),
            (Err(e), _) => panic!("native (oracle) itself failed on {expr:?}: {e}"),
        }
    }

    assert!(
        disagreements.is_empty(),
        "backend-neutral semantic oracle FAILED -- native and meta-eval.my \
         disagree on fixtures both claim to support:\n{}",
        disagreements.join("\n")
    );

    // The actual, current fact this test exists to establish -- printed
    // so it's visible in `cargo test -- --nocapture`, not just asserted
    // silently. Not a hard requirement that every fixture pass both
    // backends yet (meta-eval.my is still a partial self-hosting
    // effort) -- the hard requirement is that where both backends DO
    // run, they never disagree.
    println!(
        "compiler-corpus dual-backend report: {}/{} fixtures pass BOTH native and meta-eval.my; \
         {} not yet covered by meta-eval.my (native-only): {:?}",
        agree.len(),
        exprs.len(),
        coverage_gap.len(),
        coverage_gap
    );
    assert!(
        agree.len() >= 7,
        "expected at least the 7 known-overlapping McCarthy-7/Canon fixtures \
         (quote/atom/eq/car/cdr/cons/cond) to pass both backends, got {}",
        agree.len()
    );
}

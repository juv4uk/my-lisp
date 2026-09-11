//! Backend-neutral semantic oracle, first concrete instance — per the
//! 2026-09-11 course correction: the goal is not "less Rust," it's
//! "my-lisp is the semantic authority regardless of which engine
//! executes it." The acceptance test for that claim: can the SAME
//! semantic corpus (#67's frozen `tests/fixtures/conformance.my`
//! `(compiler-corpus . t)` fixtures) pass against more than one
//! independent implementation, with no fixture rewritten per backend?
//!
//! Revised 2026-09-12 per owner review: the corpus is the oracle, not
//! the native Rust evaluator. The first version of this file compared
//! backends to EACH OTHER and let Rust classify disagreements
//! ("native succeeded, meta-eval returned an error-shaped value" =
//! coverage gap; magic `agree.len() >= 7` for "the known McCarthy-7/Canon
//! fixtures") — that is semantic policy living in Rust, exactly the
//! anti-pattern this whole effort exists to remove. This version is a
//! dumb runner: it loads the corpus, runs each backend, and compares
//! each backend's raw result against the fixture's OWN `expected` field.
//! The only backend-specific fact this file is allowed to read from the
//! corpus is the `meta-eval-gap` tag — data the corpus carries about
//! itself, not a judgment Rust invents at match time. A future CML/FPGA
//! runner needs the same two facts (`expected`, and its own
//! `<backend>-gap` tag if it wants one) and nothing else from this file.

use my_lisp::{eval_program, load_core_library, load_meta_evaluator_library, parse, Expr, ExprKind, Session};

struct CompilerCorpusFixture {
    expr: String,
    expected: String,
    meta_eval_gap: bool,
}

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

/// Only fixtures with an `expected` value (success fixtures) are
/// meaningful for this value-parity check; `error` fixtures are a
/// different question (does a second backend raise the same NAMED
/// failure), not attempted here.
fn compiler_corpus_fixtures() -> Vec<CompilerCorpusFixture> {
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
            let expected = alist_str(entries, "expected")?;
            let expr = alist_str(entries, "expr")?;
            Some(CompilerCorpusFixture {
                expr: expr.to_string(),
                expected: expected.to_string(),
                meta_eval_gap: alist_bool_true(entries, "meta-eval-gap"),
            })
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

/// A backend's raw result against one fixture: whether it matched the
/// corpus's own `expected` value, and if not, whether the fixture
/// itself already excused this backend from covering it.
enum FixtureOutcome {
    Match,
    ExcusedMiss,
    Mismatch(String),
}

fn check_backend(fixture: &CompilerCorpusFixture, result: &Result<String, String>, gap_excused: bool) -> FixtureOutcome {
    match result {
        Ok(value) if *value == fixture.expected => FixtureOutcome::Match,
        _ if gap_excused => FixtureOutcome::ExcusedMiss,
        Ok(value) => FixtureOutcome::Mismatch(format!(
            "{:?}: expected {:?}, got {:?}",
            fixture.expr, fixture.expected, value
        )),
        Err(error) => FixtureOutcome::Mismatch(format!(
            "{:?}: expected {:?}, backend errored: {error}",
            fixture.expr, fixture.expected
        )),
    }
}

/// Both backends are checked against the corpus's own `expected` field,
/// never against each other. Native Rust gets no gap excuse: the
/// compiler-corpus fixtures are Rust's own frozen contract (#67), so a
/// native mismatch is always a hard failure, not a coverage gap.
/// `meta-eval.my` may fall short only on fixtures the corpus itself
/// tags `meta-eval-gap` — that tag is data a future CML/FPGA runner can
/// read the same way, not a judgment this file invents from an error
/// shape at match time.
#[test]
fn compiler_corpus_dual_backend_parity_report() {
    let fixtures = compiler_corpus_fixtures();
    assert!(!fixtures.is_empty(), "no compiler-corpus fixtures found");

    let mut native_failures = Vec::new();
    let mut meta_eval_failures = Vec::new();
    let mut meta_eval_covered = 0usize;
    let mut meta_eval_excused_gaps = Vec::new();

    for fixture in &fixtures {
        match check_backend(&fixture, &eval_native(&fixture.expr), false) {
            FixtureOutcome::Match => {}
            FixtureOutcome::ExcusedMiss => unreachable!("native is never excused"),
            FixtureOutcome::Mismatch(detail) => native_failures.push(detail),
        }

        match check_backend(&fixture, &eval_via_meta_eval(&fixture.expr), fixture.meta_eval_gap) {
            FixtureOutcome::Match => meta_eval_covered += 1,
            FixtureOutcome::ExcusedMiss => meta_eval_excused_gaps.push(fixture.expr.clone()),
            FixtureOutcome::Mismatch(detail) => meta_eval_failures.push(detail),
        }
    }

    assert!(
        native_failures.is_empty(),
        "native Rust evaluator diverged from its own frozen compiler-corpus contract:\n{}",
        native_failures.join("\n")
    );
    assert!(
        meta_eval_failures.is_empty(),
        "backend-neutral semantic oracle FAILED -- meta-eval.my disagrees with the \
         corpus on a fixture it does not carry a meta-eval-gap excuse for:\n{}",
        meta_eval_failures.join("\n")
    );

    // Not asserted against a hardcoded count -- the corpus's own
    // meta-eval-gap tags already say exactly which fixtures meta-eval.my
    // is excused from, so total coverage is whatever's left, printed for
    // visibility rather than pinned to a number that would need editing
    // by hand every time meta-eval.my's coverage changes.
    println!(
        "compiler-corpus dual-backend report: {}/{} fixtures pass meta-eval.my \
         (both backends matched the corpus's own `expected` value); \
         {} excused by the corpus's own meta-eval-gap tag: {:?}",
        meta_eval_covered,
        fixtures.len(),
        meta_eval_excused_gaps.len(),
        meta_eval_excused_gaps
    );
}

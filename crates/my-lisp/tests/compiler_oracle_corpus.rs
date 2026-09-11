//! Compiler oracle corpus — GitHub issue juv4uk/my-lisp#67. See
//! docs/COMPILER-ORACLE-CORPUS.md for what this corpus is and is not.
//!
//! This is deliberately NOT a new, parallel fixture file: it selects a
//! small, representative subset of the existing `tests/fixtures/conformance.my`
//! (already the implementation-independent semantic contract every native
//! and meta-eval witness in this repo is checked against) via a new
//! `(compiler-corpus . t)` alist key, added directly on selected entries —
//! the same pattern `tier`/`axioms`/`role`/`note` already established, per
//! wsm-my-lisp's own proposal for how a consumer-side status key should be
//! layered onto this file rather than forked into a second table.

use my_lisp::{eval_program, load_core_library, parse, Expr, ExprKind, Session};

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

struct CorpusEntry {
    expr: String,
    expected: Option<String>,
    error: Option<String>,
}

fn load_corpus() -> Vec<CorpusEntry> {
    let forms = parse(include_str!("../../../tests/fixtures/conformance.my"))
        .expect("conformance.my should parse as valid my-lisp source");
    forms
        .iter()
        .filter_map(|form| {
            let ExprKind::List(entries) = &form.kind else {
                return None;
            };
            if !alist_bool_true(entries, "compiler-corpus") {
                return None;
            }
            let expr = alist_str(entries, "expr")?.to_string();
            let expected = alist_str(entries, "expected").map(str::to_string);
            let error = alist_str(entries, "error").map(str::to_string);
            Some(CorpusEntry {
                expr,
                expected,
                error,
            })
        })
        .collect()
}

/// The actual oracle check: does the current native evaluator still produce
/// exactly the recorded observation for this fixture? A future compiled
/// execution path must match this same function's result, not a separately
/// invented notion of "correct."
fn check_against_native_oracle(entry: &CorpusEntry) -> Result<(), String> {
    let mut session = Session::default();
    load_core_library(&mut session).expect("lib/core.my should load (real CLI always loads it)");
    let result = eval_program(&entry.expr, &mut session);
    match (&entry.expected, &entry.error, result) {
        (Some(expected), None, Ok(outcome)) => {
            let observed = outcome.value.to_string();
            if &observed == expected {
                Ok(())
            } else {
                Err(format!(
                    "value mismatch for {:?}: expected {expected:?}, observed {observed:?}",
                    entry.expr
                ))
            }
        }
        (None, Some(expected_kind), Err(err)) => {
            let observed_kind = format!("{:?}", err.kind);
            if &observed_kind == expected_kind {
                Ok(())
            } else {
                Err(format!(
                    "error-kind mismatch for {:?}: expected {expected_kind:?}, observed {observed_kind:?}",
                    entry.expr
                ))
            }
        }
        (Some(_), None, Err(err)) => Err(format!(
            "expected a value for {:?} but evaluation failed: {err}",
            entry.expr
        )),
        (None, Some(_), Ok(outcome)) => Err(format!(
            "expected a named failure for {:?} but evaluation succeeded with {:?}",
            entry.expr, outcome.value
        )),
        (None, None, _) | (Some(_), Some(_), _) => Err(format!(
            "fixture {:?} must have exactly one of expected/error",
            entry.expr
        )),
    }
}

#[test]
fn corpus_is_nonempty_and_covers_the_required_categories() {
    let corpus = load_corpus();
    assert!(
        !corpus.is_empty(),
        "no (compiler-corpus . t) fixtures found in conformance.my"
    );
    // Small enough for ordinary CI, per #67's own acceptance criterion --
    // this is a curated representative sample, not the full 227-fixture
    // conformance.my. If this grows past a few dozen, split deep variants
    // into a nightly-only corpus instead, per the CI split precedent
    // already landed for meta-eval witnesses (46b1fec).
    assert!(
        corpus.len() < 40,
        "compiler oracle corpus has grown to {} fixtures -- reconsider a \
         fast-CI/nightly split before this becomes a scaling problem, \
         same as the meta-eval evidence witness already did",
        corpus.len()
    );
}

#[test]
fn every_corpus_fixture_matches_the_current_native_oracle() {
    let corpus = load_corpus();
    let mut failures = Vec::new();
    for entry in &corpus {
        if let Err(message) = check_against_native_oracle(entry) {
            failures.push(message);
        }
    }
    assert!(
        failures.is_empty(),
        "compiler oracle corpus drifted from the native reference \
         implementation:\n{}",
        failures.join("\n")
    );
}

/// Acceptance evidence for #67: "a deliberate result/error mutation is
/// detected." Rather than temporarily hand-editing conformance.my (the
/// approach used to verify #66's gate, then reverted), this test
/// constructs an in-memory corrupted entry and proves the same checking
/// function used above rejects it -- an always-green CI test that
/// exercises the detection path itself, not a manual one-off experiment.
#[test]
fn a_deliberately_wrong_expectation_is_detected_not_silently_accepted() {
    let corrupted = CorpusEntry {
        expr: "(quote radio)".to_string(),
        expected: Some("NOT-radio".to_string()),
        error: None,
    };
    let result = check_against_native_oracle(&corrupted);
    assert!(
        result.is_err(),
        "a deliberately wrong expected value must be rejected, not silently accepted"
    );

    let corrupted_error = CorpusEntry {
        expr: "(defmacro foo)".to_string(),
        expected: None,
        error: Some("InvalidForm".to_string()), // real kind is Arity
    };
    let result = check_against_native_oracle(&corrupted_error);
    assert!(
        result.is_err(),
        "a deliberately wrong expected error kind must be rejected, not silently accepted"
    );
}

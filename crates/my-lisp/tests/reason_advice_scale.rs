//! B5 performance probe for the actual Advice Taker read path.
//!
//! Unlike `reason_scale.rs`, which isolates one worst-case full scan, this
//! harness keeps the knowledge journal/module projection and canonical
//! `reason-in-observe` adapter visible. Fixture installation is outside the
//! timed region and deliberately bypasses `advise-all`: atomic batch admission
//! has its own conflict-check complexity and must not contaminate a reasoning
//! measurement. Timings are diagnostic evidence only; semantic assertions
//! remain deterministic.

use my_lisp::{eval_program, Session};
use std::time::Instant;

fn loaded_session() -> Session {
    let mut session = Session::default();
    for library in [
        include_str!("../../../lib/core.my"),
        include_str!("../../../lib/unify.my"),
        include_str!("../../../lib/reason.my"),
        include_str!("../../../lib/forward.my"),
        include_str!("../../../lib/knowledge.my"),
        include_str!("../../../lib/result-status.my"),
    ] {
        eval_program(library, &mut session).unwrap();
    }
    session
}

fn eval_session(session: &mut Session, source: &str) -> String {
    eval_program(source, session)
        .unwrap_or_else(|e| panic!("evaluation failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

/// Install a mixed module directly through the ordinary module/journal shape.
/// Distractor predicates model a heterogeneous knowledge base. The target is
/// intentionally placed after them and requires a two-rule derivation:
///
///   planet(target) -> has(target, mass) -> valuable(target)
///
/// Without predicate/head indexing, each proof level still scans the whole
/// projected clause list.
fn install_mixed_module(session: &mut Session, distractors: usize) -> usize {
    let mut source = String::from("(defmodule bench (quote (");
    for i in 0..distractors {
        match i % 4 {
            0 => source.push_str(&format!("((planet f{i}))")),
            1 => source.push_str(&format!("((orbits f{i} s{i}))")),
            2 => source.push_str(&format!("((has-mass f{i}))")),
            _ => source.push_str(&format!("((located-in f{i} sector{i}))")),
        }
    }
    source.push_str("((planet target))");
    source.push_str("((has (var x) mass) (planet (var x)))");
    source.push_str("((valuable (var x)) (has (var x) mass))");
    source.push_str(")))");

    eval_session(session, &source);
    distractors + 3
}

fn median_ns<F>(mut f: F) -> u128
where
    F: FnMut() -> u128,
{
    let mut samples = [f(), f(), f()];
    samples.sort_unstable();
    samples[1]
}

fn timed_eval(session: &mut Session, source: &str, expected: &str) -> u128 {
    let start = Instant::now();
    let value = eval_session(session, source);
    let elapsed = start.elapsed().as_nanos();
    assert_eq!(value, expected, "profile query changed semantics");
    elapsed
}

fn profile_size(distractors: usize) -> (u128, u128, u128, usize) {
    let mut session = loaded_session();
    let clauses = install_mixed_module(&mut session, distractors);

    let projection = "(length (module-clauses-now (quote bench)))";
    let expected_len = clauses.to_string();
    assert_eq!(eval_session(&mut session, projection), expected_len);

    // Pre-project once so the raw reasoning number excludes journal replay.
    eval_session(
        &mut session,
        "(def bench-rules (module-clauses-now (quote bench)))",
    );

    let raw_reason = "(result-status (reason-observe (quote (valuable target)) bench-rules))";
    let end_to_end = "(result-status (reason-in-observe (quote bench) (quote (valuable target))))";

    // Warm both reasoning paths before the three-sample median.
    assert_eq!(eval_session(&mut session, raw_reason), "proved");
    assert_eq!(eval_session(&mut session, end_to_end), "proved");

    let projection_ns = median_ns(|| timed_eval(&mut session, projection, &expected_len));
    let raw_reason_ns = median_ns(|| timed_eval(&mut session, raw_reason, "proved"));
    let end_to_end_ns = median_ns(|| timed_eval(&mut session, end_to_end, "proved"));

    (projection_ns, raw_reason_ns, end_to_end_ns, clauses)
}

fn profile_sizes(sizes: &[usize]) {
    let mut table = String::from(
        "Advice Taker B5 profile (3-sample median, default stack)\n\
         distractors clauses projection_ns raw_reason_ns end_to_end_ns\n",
    );

    for &n in sizes {
        let (projection_ns, raw_reason_ns, end_to_end_ns, clauses) = profile_size(n);
        table.push_str(&format!(
            "{n:<11} {clauses:<7} {projection_ns:<13} {raw_reason_ns:<13} {end_to_end_ns}\n"
        ));
    }

    println!("\n{table}");
}

#[test]
fn advice_taker_profile_100_500_1000_distractors() {
    profile_sizes(&[100, 500, 1_000]);
}

#[test]
#[ignore = "manual B5 extended profile before an indexing decision"]
fn advice_taker_profile_5000_10000_distractors() {
    // Run explicitly with:
    // cargo test -p my-lisp --test reason_advice_scale advice_taker_profile_5000_10000 -- --ignored --nocapture
    profile_sizes(&[5_000, 10_000]);
}

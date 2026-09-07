//! B5 realistic Advice Taker performance profile.
//!
//! This is deliberately not another worst-case single-predicate edge chain.
//! It builds a mixed knowledge corpus with several ordinary predicates and a
//! multi-step query, then measures the current public indexed `reason` path
//! against an exact forced-linear reference using the same proof engine.
//!
//! Timing is diagnostic only. Shared CI wall-clock numbers are noisy and must
//! never become semantic assertions. Stable assertions here are:
//! - the generated corpus is actually indexable;
//! - indexed and forced-linear results are structurally identical;
//! - the expected query has exactly one proof.
//!
//! The purpose is to gather evidence before changing the representation again.

use my_lisp::{eval_program, Session};
use std::time::Instant;

fn loaded_session() -> Session {
    let mut session = Session::default();
    for library in [
        include_str!("../../../lib/core.my"),
        include_str!("../../../lib/unify.my"),
        include_str!("../../../lib/reason.my"),
    ] {
        eval_program(library, &mut session).expect("Advice Taker profile library should load");
    }
    session
}

fn eval(session: &mut Session, source: &str) -> String {
    eval_program(source, session)
        .unwrap_or_else(|e| panic!("evaluation failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

/// Build a mixed corpus resembling an Advice Taker module rather than a single
/// homogeneous scan. Each entity contributes eight independent observations;
/// the target query needs a five-predicate derivation through two rules.
///
/// All rule heads are ordinary symbolic predicates and there are only ten
/// distinct predicates, so this corpus should exercise indexed mode rather
/// than the compatibility fallback.
fn install_mixed_corpus(session: &mut Session, entities: usize) -> String {
    let name = format!("advice-rules-{entities}");
    let mut source = format!("(def {name} (quote (");

    for i in 0..entities {
        let entity = format!("e{i}");
        source.push_str(&format!(
            "((planet {entity}))\
             ((atmosphere {entity}))\
             ((orbit-stable {entity}))\
             ((temperature-safe {entity}))\
             ((massive {entity}))\
             ((observed {entity}))\
             ((catalogued {entity}))\
             ((named {entity}))"
        ));
    }

    source.push_str(
        "((habitable (var x))\
            (planet (var x))\
            (atmosphere (var x))\
            (orbit-stable (var x))\
            (temperature-safe (var x)))\
         ((valuable (var x))\
            (habitable (var x))\
            (massive (var x))))))",
    );

    eval(session, &source);
    name
}

fn timed_eval(session: &mut Session, source: &str) -> (u128, String) {
    let start = Instant::now();
    let value = eval(session, source);
    (start.elapsed().as_nanos(), value)
}

fn profile_one(session: &mut Session, entities: usize) -> (u128, u128) {
    let rules = install_mixed_corpus(session, entities);
    let target = format!("e{}", entities - 1);

    assert_eq!(
        eval(
            session,
            &format!("(reason-index-mode (reason-make-index {rules}))")
        ),
        "indexed",
        "mixed Advice Taker corpus should exercise the predicate index"
    );

    let indexed_source = format!(
        "(reason (quote (valuable {target})) {rules})"
    );
    let linear_source = format!(
        "(prove-goal\
            (quote (valuable {target}))\
            {rules}\
            (quote ())\
            (reason-index-linear {rules})\
            0)"
    );

    let (indexed_ns, indexed) = timed_eval(session, &indexed_source);
    let (linear_ns, linear) = timed_eval(session, &linear_source);

    assert_eq!(
        indexed, linear,
        "indexing must not alter proof order, substitutions, or proof shape at N={entities}"
    );

    assert_eq!(
        eval(session, &format!("(length {indexed_source})")),
        "1",
        "target entity should have exactly one proof at N={entities}"
    );

    (indexed_ns, linear_ns)
}

fn profile_sizes(sizes: &[usize]) {
    let mut session = loaded_session();
    let mut table = String::from(
        "Advice Taker B5 profile (mixed predicates, two-step derivation, default stack)\n",
    );
    table.push_str("  entities   rules     indexed_ns      linear_ns      linear/indexed\n");

    for &entities in sizes {
        let (indexed_ns, linear_ns) = profile_one(&mut session, entities);
        let rule_count = entities * 8 + 2;
        let ratio = if indexed_ns == 0 {
            0.0
        } else {
            linear_ns as f64 / indexed_ns as f64
        };
        table.push_str(&format!(
            "  {entities:<10} {rule_count:<9} {indexed_ns:>12} {linear_ns:>14} {ratio:>16.2}x\n"
        ));
    }

    println!("\n{table}");
}

#[test]
fn advice_taker_realistic_index_profile_32_128_512() {
    profile_sizes(&[32, 128, 512]);
}

#[test]
#[ignore = "manual larger realistic profile before further reason representation changes"]
fn advice_taker_realistic_index_profile_extended_1000_2500() {
    profile_sizes(&[1_000, 2_500]);
}

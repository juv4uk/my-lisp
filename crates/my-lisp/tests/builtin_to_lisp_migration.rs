//! First vertical slice of the owner-directed Rust-semantic-surface
//! reduction (2026-09-11): "Lisp owns meaning, Rust owns only irreducible
//! mechanism." `abs`/`min`/`max`/`min-list`/`max-list` were plain Rust
//! `Builtin` values in `crates/my-lisp/src/eval/builtins.rs` — none of
//! them touch OS/host capability, only arithmetic comparison and
//! cons-list traversal already expressible in the language itself, so
//! keeping them in Rust was Rust-authority with no substrate reason.
//!
//! This is the parity witness: every case here was run against the real
//! (still-Rust) builtin before the Rust implementation was deleted, not
//! assumed from reading the old source. Two real bugs surfaced during
//! that verification (documented at their fix sites in `lib/core.my`):
//! `eq` erroring on a non-atom `items` argument, and `atom` being an
//! unsafe "is this the recursion's base case" test for the *accumulator*
//! specifically (a number is also an atom, colliding with the empty-list
//! sentinel).
//!
//! See `docs/BUILTIN-IDENTITY-MIGRATION-MAP-2026-09-11.md` for the wider
//! migration this is the first slice of, and the real semantic-registry
//! entries these five already had (abs=1004, min=1005, max=1006,
//! min-list=1011, max-list=1012) that the deleted Rust code never
//! consulted.

use my_lisp::{eval_program, Session};

fn session_with_core() -> Session {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session)
        .expect("core.my should preload cleanly");
    session
}

fn eval_source(source: &str) -> String {
    let result = eval_program(source, &mut session_with_core()).expect("eval should succeed");
    result.value.to_string()
}

#[test]
fn abs_matches_the_former_rust_builtin_on_integers_rationals_and_negatives() {
    assert_eq!(eval_source("(abs -5)"), "5");
    assert_eq!(eval_source("(abs 5)"), "5");
    assert_eq!(eval_source("(abs 5.5)"), "11/2");
    assert_eq!(eval_source("(abs (/ -3 4))"), "3/4");
    assert_eq!(eval_source("(abs 0)"), "0");
}

#[test]
fn min_and_max_match_the_former_rust_builtin_variadic_behavior() {
    assert_eq!(eval_source("(min 3 1 2)"), "1");
    assert_eq!(eval_source("(max 3 1 2)"), "3");
    assert_eq!(eval_source("(min 5)"), "5");
    assert_eq!(eval_source("(max 5)"), "5");
    assert_eq!(eval_source("(min 5 5 5)"), "5");
}

#[test]
fn min_and_max_still_raise_arity_on_zero_arguments() {
    // The former Rust builtin raised a named Arity error for zero
    // arguments -- the Lisp version reproduces this via a required
    // first parameter (dotted lambda-list), not a hand-written check.
    let mut session = session_with_core();
    let err = eval_program("(min)", &mut session).expect_err("zero-arg min must be an Arity error");
    assert_eq!(err.kind, my_lisp::ErrorKind::Arity);

    let mut session = session_with_core();
    let err = eval_program("(max)", &mut session).expect_err("zero-arg max must be an Arity error");
    assert_eq!(err.kind, my_lisp::ErrorKind::Arity);
}

#[test]
fn min_list_and_max_list_match_the_former_rust_builtin() {
    assert_eq!(eval_source("(min-list (quote (5 2 8 1)))"), "1");
    assert_eq!(eval_source("(max-list (quote (5 2 8 1)))"), "8");
    assert_eq!(eval_source("(min-list (quote (7)))"), "7");
    assert_eq!(eval_source("(max-list (quote (7)))"), "7");
}

#[test]
fn min_list_and_max_list_return_the_empty_list_for_empty_input() {
    // The former Rust builtin returned Nil (not an Arity error) for an
    // empty list argument -- distinct from min/max's own zero-*argument*
    // Arity error above.
    assert_eq!(eval_source("(min-list (quote ()))"), "()");
    assert_eq!(eval_source("(max-list (quote ()))"), "()");
}

#[test]
fn ukrainian_peer_names_resolve_to_the_same_migrated_definitions() {
    // модуль/найменше/найбільше/найменше-у-списку/найбільше-у-списку are
    // uk.my's peer bindings to the same lib/core.my values -- loading
    // uk.my on top of core.my must produce identical results through
    // the Ukrainian spelling, matching semantic-registry.wsm's real
    // entries (1004-1006, 1011-1012), not a new hand-copied table.
    // uk.my depends on the same prerequisite chain the real CLI loads
    // before it (crates/my-lisp-cli/src/repl.rs's SURFACE_PREREQUISITES) --
    // core.my alone is not enough (found live: uk.my failed on an
    // unresolved `map-empty` from persistent-map.my when this test first
    // tried loading only core.my + uk.my).
    let mut session = session_with_core();
    for source in [
        include_str!("../../../lib/unify.my"),
        include_str!("../../../lib/reason.my"),
        include_str!("../../../lib/forward.my"),
        include_str!("../../../lib/knowledge.my"),
        include_str!("../../../lib/persistent-map.my"),
        include_str!("../../../lib/persistent-vector.my"),
        include_str!("../../../lib/time.my"),
        include_str!("../../../lib/epistemic.my"),
        include_str!("../../../lib/surface/uk.my"),
    ] {
        eval_program(source, &mut session).expect("surface prerequisite should load cleanly");
    }
    let result = |source: &str, session: &mut Session| {
        eval_program(source, session).unwrap().value.to_string()
    };
    assert_eq!(result("(модуль -7)", &mut session), "7");
    assert_eq!(result("(найменше 4 1 9)", &mut session), "1");
    assert_eq!(result("(найбільше 4 1 9)", &mut session), "9");
    assert_eq!(
        result("(найменше-у-списку (quote (5 2 8 1)))", &mut session),
        "1"
    );
    assert_eq!(
        result("(найбільше-у-списку (quote (5 2 8 1)))", &mut session),
        "8"
    );
}

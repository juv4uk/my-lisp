//! Registry-driven UK/EN surface equivalence sweep.
//!
//! Data source: `lib/surface/semantic-registry.wsm`, the byte-SID surface
//! authority (see `semantic_registry.rs` for the runtime parser this test
//! mirrors, and `peer_surface_identity.rs` for the same pattern applied to
//! one byte SID). This file used to read the legacy EN-shaped
//! `lib/surface/uk-sa-coverage.wsm`, which `rivnopravnist_mov.rs` and
//! `runtime_peer_operators.rs` already assert is no longer executable
//! authority (TEST-ARCHITECTURE-1 step 2 migration, 2026-09-12).

use my_lisp::{eval_program, load_core_library, Session, Value};
use std::rc::Rc;

const REGISTRY: &str = include_str!("../../../lib/surface/semantic-registry.lisp");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Admission {
    Stable,
    CompatibilityOnly,
    Candidate,
    Missing,
}

/// One surface declaration from a semantic-registry row. Admitted/stable is implicit.
struct Surface {
    namespace: &'static str,
    name: &'static str,
    admission: Admission,
}

/// Parse every row of the registry into its byte SID plus the raw
/// surface declarations it carries. Mirrors
/// `semantic_registry::parse_rows` (crate-internal, not reachable from an
/// integration test), but keeps every admission kind instead of dropping
/// `candidate`/`missing` at parse time, since this file needs those to
/// report accurate status breakdowns.
fn surface_groups(line: &'static str) -> Vec<&'static str> {
    let mut groups = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (index, byte) in line.bytes().enumerate() {
        match byte {
            b'(' => {
                depth += 1;
                if depth == 2 {
                    start = Some(index + 1);
                }
            }
            b')' => {
                if depth == 2 {
                    if let Some(group_start) = start.take() {
                        let group = line[group_start..index].trim();
                        if !group.is_empty() {
                            groups.push(group);
                        }
                    }
                }
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    groups
}

fn registry_rows() -> Vec<(&'static str, Vec<Surface>)> {
    REGISTRY
        .lines()
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            let semantic_id = fields
                .first()?
                .strip_prefix("(\\\"")?
                .strip_suffix('"')?;
            if semantic_id.len() != 8
                || !semantic_id.bytes().all(|byte| matches!(byte, b'0' | b'1'))
            {
                return None;
            }

            let surfaces = surface_groups(line)
                .into_iter()
                .map(|group| {
                    let fields = group.split_whitespace().collect::<Vec<_>>();
                    let (namespace, name, admission) = match fields.as_slice() {
                        [namespace, name] => (*namespace, *name, Admission::Stable),
                        [namespace, name, exception_status] => {
                            let admission = match *exception_status {
                                "compatibility-only" => Admission::CompatibilityOnly,
                                "candidate" => Admission::Candidate,
                                "missing" => Admission::Missing,
                                "stable" => panic!("sr/2 must not spell stable explicitly"),
                                other => panic!("unknown sr/2 status {other}"),
                            };
                            (*namespace, *name, admission)
                        }
                        _ => panic!("malformed sr/2 surface group: ({group})"),
                    };
                    Surface {
                        namespace,
                        name,
                        admission,
                    }
                })
                .collect();
            Some((semantic_id, surfaces))
        })
        .collect()
}

fn surface<'a>(surfaces: &'a [Surface], namespace: &str) -> Option<&'a Surface> {
    surfaces.iter().find(|surface| surface.namespace == namespace)
}

/// Every byte SID whose EN spelling AND UK spelling are both `stable` --
/// exactly the set for which "does the UK spelling resolve to the same
/// runtime operation as the EN spelling" is a meaningful question. IDs that
/// are UK-only (e.g. `додати`/`+`, which has no spelled-out EN name, only a
/// symbolic one) are covered separately by `rivnopravnist_mov.rs` and
/// `runtime_peer_operators.rs`, which compare against the symbolic/SA
/// spellings instead.
fn stable_en_uk_pairs() -> Vec<(&'static str, &'static str, &'static str)> {
    registry_rows()
        .into_iter()
        .filter_map(|(id, surfaces)| {
            let en = surface(&surfaces, "en")?;
            let uk = surface(&surfaces, "uk")?;
            (en.admission == Admission::Stable && uk.admission == Admission::Stable)
                .then_some((id, en.name, uk.name))
        })
        .collect()
}

/// UK-column admission counts across the whole registry (not just rows with
/// a stable EN counterpart) -- used only for the internal-consistency check
/// below, kept separate from `stable_en_uk_pairs` above so a parser bug in
/// one can't hide behind agreement with the other.
fn uk_admission_counts() -> (usize, usize, usize, usize, usize) {
    let rows = registry_rows();
    let total = rows
        .iter()
        .filter(|(_, surfaces)| surface(surfaces, "uk").is_some())
        .count();
    let mut stable = 0;
    let mut compatibility = 0;
    let mut candidate = 0;
    let mut missing = 0;
    for (_, surfaces) in &rows {
        match surface(surfaces, "uk").map(|s| s.admission) {
            Some(Admission::Stable) => stable += 1,
            Some(Admission::CompatibilityOnly) => compatibility += 1,
            Some(Admission::Candidate) => candidate += 1,
            Some(Admission::Missing) => missing += 1,
            None => {}
        }
    }
    (total, stable, compatibility, candidate, missing)
}

fn uk_session() -> Session {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core bootstrap");
    for source in [
        include_str!("../../../lib/unify.lisp"),
        include_str!("../../../lib/reason.lisp"),
        include_str!("../../../lib/forward.lisp"),
        include_str!("../../../lib/knowledge.lisp"),
        include_str!("../../../lib/persistent-map.lisp"),
        include_str!("../../../lib/persistent-vector.lisp"),
        include_str!("../../../lib/time.lisp"),
        include_str!("../../../lib/epistemic.lisp"),
    ] {
        eval_program(source, &mut session).expect("surface prerequisite should load");
    }
    eval_program(include_str!("../../../lib/surface/uk.lisp"), &mut session)
        .expect("Ukrainian surface should load");
    session
}

fn is_same_runtime_value(left: &Value, right: &Value) -> bool {
    match (left, right) {
        // A builtin is an operation handle. Canon EN/UK/SA spellings and
        // ordinary aliases must retain one allocation, not merely similar code.
        (Value::Builtin(left), Value::Builtin(right)) => Rc::ptr_eq(left, right),
        _ => left == right,
    }
}

#[test]
fn every_stable_uk_surface_entry_resolves_to_its_declared_operation() {
    let pairs = stable_en_uk_pairs();
    // Floor, not exact count: the registry only grows. An exact hardcoded
    // count here would silently rot every time a new stable pair is added --
    // this just guards against the parse producing an (almost) empty set.
    assert!(
        pairs.len() >= 100,
        "expected a substantial number of stable EN/UK pairs, found {}",
        pairs.len()
    );

    let mut session = uk_session();
    // Evaluation-control/necessary forms (quote/cond/lambda/define) are
    // syntax, not first-class values -- Rc-identity comparison is not
    // meaningful for them, so they're verified behaviorally in
    // uk_surface.rs / uk_sa_surface.rs instead of here.
    let syntax = [
        ("quote", "як-є"),
        ("cond", "за-умовою"),
        ("lambda", "функція"),
        ("define", "визначити"),
    ];
    let mut checked_values = 0;

    for (semantic_id, english, ukrainian) in &pairs {
        if syntax.contains(&(*english, *ukrainian)) {
            continue;
        }
        let english_value = eval_program(english, &mut session)
            .unwrap_or_else(|error| {
                panic!("stable English value is missing: {semantic_id}/{english}: {error}")
            })
            .value;
        let ukrainian_value = eval_program(ukrainian, &mut session)
            .unwrap_or_else(|error| {
                panic!("stable Ukrainian value is missing: {semantic_id}/{ukrainian}: {error}")
            })
            .value;
        assert!(
            is_same_runtime_value(&english_value, &ukrainian_value),
            "stable surface changed runtime identity: {semantic_id}/{english} -> {ukrainian}"
        );
        checked_values += 1;
    }

    // Derived, not restated: every pair except the syntax forms must have
    // been checked above -- catches a silent early `continue`/`break` bug
    // in the loop without hardcoding the pair count twice.
    assert_eq!(checked_values, pairs.len() - syntax.len());
}

#[test]
fn ukrainian_surface_status_counts_are_internally_consistent() {
    let (total, stable, compatibility, candidate, missing) = uk_admission_counts();
    assert_eq!(
        stable + compatibility + candidate + missing,
        total,
        "every registry row's UK column must fall into exactly one admission bucket"
    );
    assert!(stable > 0 && total > 0, "registry must not be empty");
    // `candidate` UK rows exist for in-progress work; `missing` UK rows exist
    // for EN-only host primitives (process/tcp/file) that never got a
    // spelled-out Ukrainian name. Neither count is asserted to be zero here
    // -- that was the old (now-false) assumption from the legacy curated
    // uk-sa-coverage.wsm, which excluded unattempted rows outright instead
    // of tracking them.
}

// every_stable_ukrainian_name_is_typeable_on_the_ukrainian_layout and
// ukrainian_acceptance_program_code_never_requires_latin_layout were
// keyboard/text-policy lints, relocated to `cargo xtask verify` per
// TEST-ARCHITECTURE-1 step 4 -- see crates/xtask/src/checks.rs.

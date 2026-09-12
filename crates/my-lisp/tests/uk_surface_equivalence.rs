//! Registry-driven UK/EN surface equivalence sweep.
//!
//! Data source: `lib/surface/semantic-registry.wsm`, the numeric-ID surface
//! authority (see `semantic_registry.rs` for the runtime parser this test
//! mirrors, and `peer_surface_identity.rs` for the same pattern applied to
//! one semantic ID). This file used to read the legacy EN-shaped
//! `lib/surface/uk-sa-coverage.wsm`, which `rivnopravnist_mov.rs` and
//! `runtime_peer_operators.rs` already assert is no longer executable
//! authority (TEST-ARCHITECTURE-1 step 2 migration, 2026-09-12).

use my_lisp::{eval_program, load_core_library, Session, Value};
use std::rc::Rc;

const REGISTRY: &str = include_str!("../../../lib/surface/semantic-registry.wsm");
const UK_ACCEPTANCE: &str = include_str!("../../../lib/surface/uk-acceptance.my");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Admission {
    Stable,
    CompatibilityOnly,
    Candidate,
    Missing,
}

/// One `(namespace name status)` triple from a semantic-registry row.
struct Surface {
    namespace: &'static str,
    name: &'static str,
    admission: Admission,
}

/// Parse every row of the registry into its semantic ID plus the raw
/// `(namespace name status)` triples it declares. Mirrors
/// `semantic_registry::parse_rows` (crate-internal, not reachable from an
/// integration test), but keeps every admission kind instead of dropping
/// `candidate`/`missing` at parse time, since this file needs those to
/// report accurate status breakdowns.
fn registry_rows() -> Vec<(&'static str, Vec<Surface>)> {
    REGISTRY
        .lines()
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            let semantic_id = fields.first()?.strip_prefix('(')?;
            if semantic_id.is_empty() || !semantic_id.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }

            let mut surfaces = Vec::new();
            for triple in fields[1..].chunks(3) {
                if triple.len() != 3 {
                    break;
                }
                let namespace = triple[0].trim_start_matches('(');
                let name = triple[1];
                let admission = match triple[2].trim_end_matches(')') {
                    "stable" => Admission::Stable,
                    "compatibility-only" => Admission::CompatibilityOnly,
                    "candidate" => Admission::Candidate,
                    "missing" => Admission::Missing,
                    _ => continue,
                };
                surfaces.push(Surface {
                    namespace,
                    name,
                    admission,
                });
            }
            Some((semantic_id, surfaces))
        })
        .collect()
}

fn surface<'a>(surfaces: &'a [Surface], namespace: &str) -> Option<&'a Surface> {
    surfaces.iter().find(|surface| surface.namespace == namespace)
}

/// Every semantic ID whose EN spelling AND UK spelling are both `stable` --
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
    (rows.len(), stable, compatibility, candidate, missing)
}

fn uk_session() -> Session {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core bootstrap");
    for source in [
        include_str!("../../../lib/unify.my"),
        include_str!("../../../lib/reason.my"),
        include_str!("../../../lib/forward.my"),
        include_str!("../../../lib/knowledge.my"),
        include_str!("../../../lib/persistent-map.my"),
        include_str!("../../../lib/persistent-vector.my"),
        include_str!("../../../lib/time.my"),
        include_str!("../../../lib/epistemic.my"),
    ] {
        eval_program(source, &mut session).expect("surface prerequisite should load");
    }
    eval_program(include_str!("../../../lib/surface/uk.my"), &mut session)
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

fn is_ukrainian_layout_identifier_char(character: char) -> bool {
    "абвгґдеєжзиіїйклмнопрстуфхцчшщьюяАБВГҐДЕЄЖЗИІЇЙКЛМНОПРСТУФХЦЧШЩЬЮЯ0123456789-?!'*"
        .contains(character)
}

// TODO(TEST-ARCHITECTURE-1 step 4): relocate to policy/lint tool once xtask
// exists -- this is a keyboard/text-policy lint, not a semantic mutation test.
#[test]
fn every_stable_ukrainian_name_is_typeable_on_the_ukrainian_layout() {
    for (semantic_id, _english, ukrainian) in stable_en_uk_pairs() {
        assert!(
            ukrainian.chars().all(is_ukrainian_layout_identifier_char),
            "stable Ukrainian name needs another keyboard layout: {semantic_id}/{ukrainian}"
        );
    }
}

fn executable_characters(source: &str) -> String {
    let mut output = String::new();
    let mut characters = source.chars().peekable();
    let mut in_string = false;

    while let Some(character) = characters.next() {
        if in_string {
            match character {
                '\\' => {
                    characters.next();
                }
                '"' => in_string = false,
                _ => {}
            }
        } else {
            match character {
                ';' => {
                    for comment_character in characters.by_ref() {
                        if comment_character == '\n' {
                            output.push('\n');
                            break;
                        }
                    }
                }
                '"' => in_string = true,
                _ => output.push(character),
            }
        }
    }
    output
}

// TODO(TEST-ARCHITECTURE-1 step 4): relocate to policy/lint tool once xtask
// exists -- this is a keyboard/text-policy lint, not a semantic mutation test.
#[test]
fn ukrainian_acceptance_program_code_never_requires_latin_layout() {
    let code = executable_characters(UK_ACCEPTANCE);
    let latin = code
        .chars()
        .filter(|character| character.is_ascii_alphabetic())
        .collect::<String>();
    assert!(
        latin.is_empty(),
        "executable Ukrainian program still contains Latin letters: {latin}"
    );
}

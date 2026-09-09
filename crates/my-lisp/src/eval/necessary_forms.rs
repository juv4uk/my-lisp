//! Immutable routing for evaluator mechanisms necessary beyond Canon 0 + McCarthy7.
//!
//! Machine semantic authority lives in `lib/surface/semantic-registry.wsm`.
//! Rust owns only the mapping from numeric semantic IDs to evaluator mechanisms;
//! stable human/symbolic spellings are projected from the authority file once
//! and indexed for O(1) hot-path lookup. Human spellings are not duplicated in
//! evaluator routing data.

use std::{collections::HashMap, sync::OnceLock};

const SEMANTIC_REGISTRY: &str =
    include_str!("../../../../lib/surface/semantic-registry.wsm");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NecessaryFormIdentity {
    Define,
    Lambda,
}

#[derive(Debug)]
struct SemanticRow {
    semantic_id: &'static str,
    stable_surfaces: Vec<&'static str>,
}

pub(crate) const LAMBDA_SEMANTIC_ID: &str = "0010";
pub(crate) const DEFINE_SEMANTIC_ID: &str = "0011";

fn parse_rows(source: &'static str) -> Vec<SemanticRow> {
    source
        .lines()
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            let first = fields.first()?;
            let semantic_id = first.strip_prefix('(')?;
            if semantic_id.is_empty()
                || !semantic_id
                    .as_bytes()
                    .iter()
                    .all(|byte| byte.is_ascii_digit())
            {
                return None;
            }

            let mut stable_surfaces = Vec::new();
            for triple in fields[1..].chunks(3) {
                if triple.len() != 3 {
                    break;
                }
                let surface = triple[1];
                let status = triple[2].trim_end_matches(')');
                if status == "stable" && surface != "—" {
                    stable_surfaces.push(surface);
                }
            }

            Some(SemanticRow {
                semantic_id,
                stable_surfaces,
            })
        })
        .collect()
}

fn build_surface_index(source: &'static str) -> HashMap<&'static str, &'static str> {
    let mut index = HashMap::new();
    for row in parse_rows(source) {
        for surface in std::iter::once(row.semantic_id).chain(row.stable_surfaces) {
            if let Some(previous) = index.insert(surface, row.semantic_id) {
                panic!(
                    "semantic registry surface must be unique: {surface} maps to both {previous} and {}",
                    row.semantic_id
                );
            }
        }
    }
    index
}

fn surface_index() -> &'static HashMap<&'static str, &'static str> {
    static INDEX: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    INDEX.get_or_init(|| build_surface_index(SEMANTIC_REGISTRY))
}

fn semantic_id_for_surface(name: &str) -> Option<&'static str> {
    surface_index().get(name).copied()
}

fn identity_for_semantic_id(semantic_id: &str) -> Option<NecessaryFormIdentity> {
    match semantic_id {
        DEFINE_SEMANTIC_ID => Some(NecessaryFormIdentity::Define),
        LAMBDA_SEMANTIC_ID => Some(NecessaryFormIdentity::Lambda),
        _ => None,
    }
}

/// Resolve an executable list-head symbol through the authority registry first,
/// then select the evaluator mechanism by numeric semantic ID. Registry parsing
/// and indexing happen once; every evaluator lookup after that is O(1).
pub(crate) fn identity_for_symbol(name: &str) -> Option<NecessaryFormIdentity> {
    semantic_id_for_surface(name).and_then(identity_for_semantic_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn necessary_forms_are_selected_by_numeric_semantic_identity() {
        assert_eq!(
            identity_for_symbol(DEFINE_SEMANTIC_ID),
            Some(NecessaryFormIdentity::Define)
        );
        assert_eq!(
            identity_for_symbol(LAMBDA_SEMANTIC_ID),
            Some(NecessaryFormIdentity::Lambda)
        );
    }

    #[test]
    fn ukrainian_and_english_names_are_registry_driven_peer_spellings() {
        assert_eq!(
            identity_for_symbol("визначити"),
            Some(NecessaryFormIdentity::Define)
        );
        assert_eq!(
            identity_for_symbol("define"),
            Some(NecessaryFormIdentity::Define)
        );
        assert_eq!(
            identity_for_symbol("функція"),
            Some(NecessaryFormIdentity::Lambda)
        );
        assert_eq!(
            identity_for_symbol("lambda"),
            Some(NecessaryFormIdentity::Lambda)
        );
    }

    #[test]
    fn non_stable_or_unrelated_spellings_do_not_gain_necessary_form_identity() {
        assert_eq!(identity_for_symbol("def"), None);
        assert_eq!(identity_for_symbol("#0010"), None);
        assert_eq!(identity_for_symbol("id0010"), None);
        assert_eq!(identity_for_symbol("quote"), None);
    }

    #[test]
    fn registry_projection_accepts_only_stable_surface_status() {
        const SYNTHETIC: &str =
            "(4242 (xx comet stable) (yy asteroid candidate) (zz — missing))";
        let parsed = parse_rows(SYNTHETIC);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].semantic_id, "4242");
        assert_eq!(parsed[0].stable_surfaces, vec!["comet"]);
    }

    #[test]
    fn registry_index_contains_machine_ids_and_stable_surfaces_only() {
        const SYNTHETIC: &str =
            "(4242 (xx comet stable) (yy asteroid candidate) (zz — missing))";
        let index = build_surface_index(SYNTHETIC);
        assert_eq!(index.get("4242"), Some(&"4242"));
        assert_eq!(index.get("comet"), Some(&"4242"));
        assert_eq!(index.get("asteroid"), None);
        assert_eq!(index.get("—"), None);
    }

    #[test]
    fn synthetic_registry_constructively_controls_necessary_form_routing() {
        const SYNTHETIC: &str = "(0010 (xx comet stable))\n(0011 (xx asteroid stable))";
        let index = build_surface_index(SYNTHETIC);

        let route = |surface: &str| {
            index
                .get(surface)
                .copied()
                .and_then(identity_for_semantic_id)
        };

        assert_eq!(route("comet"), Some(NecessaryFormIdentity::Lambda));
        assert_eq!(route("asteroid"), Some(NecessaryFormIdentity::Define));
        assert_eq!(route("lambda"), None);
        assert_eq!(route("define"), None);
    }

    #[test]
    #[should_panic(expected = "semantic registry surface must be unique")]
    fn duplicate_stable_surface_is_rejected_deterministically() {
        const CONFLICTING: &str =
            "(0010 (xx collision stable))\n(0011 (yy collision stable))";
        let _ = build_surface_index(CONFLICTING);
    }

    #[test]
    fn stage_f1_registry_cost_profile() {
        let started = std::time::Instant::now();
        let index = build_surface_index(SEMANTIC_REGISTRY);
        let elapsed = started.elapsed();
        eprintln!("STAGE_F1_REGISTRY_BYTES={}", SEMANTIC_REGISTRY.len());
        eprintln!("STAGE_F1_INDEX_ENTRIES={}", index.len());
        eprintln!("STAGE_F1_INDEX_CAPACITY={}", index.capacity());
        eprintln!("STAGE_F1_INDEX_BUILD_NS={}", elapsed.as_nanos());
    }

    #[test]
    fn registry_projection_resolves_unrelated_stable_rows_without_routing_them() {
        assert_eq!(semantic_id_for_surface("+"), Some("0104"));
        assert_eq!(identity_for_symbol("+"), None);
    }
}

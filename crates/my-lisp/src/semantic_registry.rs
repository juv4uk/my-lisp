//! Shared projection from the language surface authority to numeric semantic IDs.
//!
//! `lib/surface/semantic-registry.wsm` owns stable human/symbolic spellings.
//! This module owns only the mechanical projection of those spellings to opaque
//! numeric IDs. Evaluator meaning remains in the modules that interpret each ID.

use std::{collections::HashMap, sync::OnceLock};

const SEMANTIC_REGISTRY: &str = include_str!("../../../lib/surface/semantic-registry.wsm");

#[derive(Debug)]
struct SemanticRow {
    semantic_id: &'static str,
    stable_surfaces: Vec<&'static str>,
}

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

pub(crate) fn build_surface_index(
    source: &'static str,
) -> HashMap<&'static str, &'static str> {
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

fn stable_surfaces_from_index(
    index: &HashMap<&'static str, &'static str>,
    semantic_id: &str,
) -> Vec<&'static str> {
    let mut surfaces = index
        .iter()
        .filter_map(|(surface, mapped_id)| {
            (*mapped_id == semantic_id && *surface != semantic_id).then_some(*surface)
        })
        .collect::<Vec<_>>();
    surfaces.sort_unstable();
    surfaces
}

pub(crate) fn semantic_id_for_surface(name: &str) -> Option<&'static str> {
    surface_index().get(name).copied()
}

pub(crate) fn stable_surfaces_for_semantic_id(semantic_id: &str) -> Vec<&'static str> {
    stable_surfaces_from_index(surface_index(), semantic_id)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn stable_surfaces_are_constructively_selected_by_semantic_id() {
        const SYNTHETIC: &str =
            "(0104 (uk comet stable) (sa asteroid candidate) (sym + stable))";
        let index = build_surface_index(SYNTHETIC);
        assert_eq!(stable_surfaces_from_index(&index, "0104"), vec!["+", "comet"]);
        assert!(stable_surfaces_from_index(&index, "9999").is_empty());
    }

    #[test]
    #[should_panic(expected = "semantic registry surface must be unique")]
    fn duplicate_stable_surface_is_rejected_deterministically() {
        const CONFLICTING: &str =
            "(0010 (xx collision stable))\n(0011 (yy collision stable))";
        let _ = build_surface_index(CONFLICTING);
    }

    #[test]
    fn unrelated_stable_rows_are_projected_without_assigning_evaluator_meaning() {
        assert_eq!(semantic_id_for_surface("+"), Some("0104"));
    }
}

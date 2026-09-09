//! Shared projection from the language surface authority to numeric semantic IDs.
//!
//! `lib/surface/semantic-registry.wsm` owns human/symbolic spellings and their
//! admission status. This module owns only the mechanical projection of those
//! spellings to opaque numeric IDs. Evaluator meaning remains in the modules
//! that interpret each ID.

use std::{collections::HashMap, sync::OnceLock};

const SEMANTIC_REGISTRY: &str = include_str!("../../../lib/surface/semantic-registry.wsm");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SurfaceAdmission {
    Stable,
    CompatibilityOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SemanticSurface {
    name: &'static str,
    admission: SurfaceAdmission,
}

#[derive(Debug)]
struct SemanticRow {
    semantic_id: &'static str,
    surfaces: Vec<SemanticSurface>,
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

            let mut surfaces = Vec::new();
            for triple in fields[1..].chunks(3) {
                if triple.len() != 3 {
                    break;
                }
                let surface = triple[1];
                if surface == "—" {
                    continue;
                }
                let admission = match triple[2].trim_end_matches(')') {
                    "stable" => SurfaceAdmission::Stable,
                    "compatibility-only" => SurfaceAdmission::CompatibilityOnly,
                    _ => continue,
                };
                surfaces.push(SemanticSurface {
                    name: surface,
                    admission,
                });
            }

            Some(SemanticRow {
                semantic_id,
                surfaces,
            })
        })
        .collect()
}

pub(crate) fn build_surface_index(
    source: &'static str,
) -> HashMap<&'static str, &'static str> {
    let mut index = HashMap::new();
    for row in parse_rows(source) {
        let stable_surfaces = row
            .surfaces
            .iter()
            .filter(|surface| surface.admission == SurfaceAdmission::Stable)
            .map(|surface| surface.name);
        for surface in std::iter::once(row.semantic_id).chain(stable_surfaces) {
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

fn admitted_surfaces_from_rows(
    rows: &[SemanticRow],
    semantic_id: &str,
) -> Vec<&'static str> {
    let mut surfaces = rows
        .iter()
        .find(|row| row.semantic_id == semantic_id)
        .into_iter()
        .flat_map(|row| row.surfaces.iter().map(|surface| surface.name))
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

/// Stable and compatibility-only spellings admitted for direct runtime binding.
/// Candidate/missing surfaces and the opaque machine ID itself are excluded.
pub(crate) fn admitted_surfaces_for_semantic_id(semantic_id: &str) -> Vec<&'static str> {
    admitted_surfaces_from_rows(&parse_rows(SEMANTIC_REGISTRY), semantic_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_projection_tracks_stable_and_compatibility_admission_only() {
        const SYNTHETIC: &str = "(4242 (xx comet stable) (yy meteor compatibility-only) (zz asteroid candidate) (qq — missing))";
        let parsed = parse_rows(SYNTHETIC);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].semantic_id, "4242");
        assert_eq!(
            parsed[0].surfaces,
            vec![
                SemanticSurface {
                    name: "comet",
                    admission: SurfaceAdmission::Stable,
                },
                SemanticSurface {
                    name: "meteor",
                    admission: SurfaceAdmission::CompatibilityOnly,
                },
            ]
        );
    }

    #[test]
    fn registry_index_contains_machine_ids_and_stable_surfaces_only() {
        const SYNTHETIC: &str = "(4242 (xx comet stable) (yy meteor compatibility-only) (zz asteroid candidate) (qq — missing))";
        let index = build_surface_index(SYNTHETIC);
        assert_eq!(index.get("4242"), Some(&"4242"));
        assert_eq!(index.get("comet"), Some(&"4242"));
        assert_eq!(index.get("meteor"), None);
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
    fn admitted_surfaces_include_compatibility_without_promoting_it_to_stable() {
        const SYNTHETIC: &str = "(0012 (en comet stable) (uk asteroid candidate) (compat meteor compatibility-only) (sa — missing))";
        let rows = parse_rows(SYNTHETIC);
        assert_eq!(
            admitted_surfaces_from_rows(&rows, "0012"),
            vec!["comet", "meteor"]
        );
        let stable = build_surface_index(SYNTHETIC);
        assert_eq!(stable.get("comet"), Some(&"0012"));
        assert_eq!(stable.get("meteor"), None);
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

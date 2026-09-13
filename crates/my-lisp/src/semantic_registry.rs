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
    namespace: &'static str,
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
                let namespace = triple[0].trim_start_matches('(');
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
                    namespace,
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

/// Same shape as `build_surface_index`, but also indexes
/// `compatibility-only` surfaces (still excludes `candidate`/`missing`,
/// which never become `SemanticSurface` rows at parse time at all). Only
/// `stable` names get automatic priority elsewhere (tooling, display,
/// canon shadowing protection); a `compatibility-only` name is still a
/// real, admitted spelling for its identity, and evaluator dispatch (or
/// any other consumer that needs "does this spelling mean anything at
/// all") should not need a second hardcoded lookup path just because the
/// spelling happens to be legacy rather than current.
pub(crate) fn build_admitted_surface_index(
    source: &'static str,
) -> HashMap<&'static str, &'static str> {
    let mut index = HashMap::new();
    for row in parse_rows(source) {
        let admitted_surfaces = row.surfaces.iter().map(|surface| surface.name);
        for surface in std::iter::once(row.semantic_id).chain(admitted_surfaces) {
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

fn admitted_surface_index() -> &'static HashMap<&'static str, &'static str> {
    static INDEX: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    INDEX.get_or_init(|| build_admitted_surface_index(SEMANTIC_REGISTRY))
}

/// Semantic ID for any admitted (stable OR compatibility-only) surface --
/// unlike `semantic_id_for_surface`, which only resolves `stable` names.
pub(crate) fn admitted_semantic_id_for_surface(name: &str) -> Option<&'static str> {
    admitted_surface_index().get(name).copied()
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

#[cfg(test)]
pub(crate) fn stable_surfaces_for_semantic_id_from_source(
    source: &'static str,
    semantic_id: &str,
) -> Vec<&'static str> {
    stable_surfaces_from_index(&build_surface_index(source), semantic_id)
}

pub(crate) fn admitted_surfaces_for_semantic_id_from_source(
    source: &'static str,
    semantic_id: &str,
) -> Vec<&'static str> {
    admitted_surfaces_from_rows(&parse_rows(source), semantic_id)
}

pub(crate) fn stable_surfaces_for_semantic_id(semantic_id: &str) -> Vec<&'static str> {
    stable_surfaces_from_index(surface_index(), semantic_id)
}

/// Stable and compatibility-only spellings admitted for direct runtime binding.
/// Candidate/missing surfaces and the opaque machine ID itself are excluded.
pub(crate) fn admitted_surfaces_for_semantic_id(semantic_id: &str) -> Vec<&'static str> {
    admitted_surfaces_for_semantic_id_from_source(SEMANTIC_REGISTRY, semantic_id)
}

/// Same admission filter as `admitted_surfaces_for_semantic_id`, but keeps
/// each surface's namespace (en/uk/sa/sym/...) alongside its spelling —
/// needed by consumers (e.g. the CML semantic export) that must know which
/// human/symbolic language a spelling belongs to, not just that it's
/// admitted. Sorted by (namespace, name) for deterministic output.
pub(crate) fn admitted_surfaces_with_namespace_for_semantic_id(
    semantic_id: &str,
) -> Vec<(&'static str, &'static str)> {
    let rows = parse_rows(SEMANTIC_REGISTRY);
    let mut surfaces = rows
        .iter()
        .find(|row| row.semantic_id == semantic_id)
        .into_iter()
        .flat_map(|row| row.surfaces.iter().map(|s| (s.namespace, s.name)))
        .collect::<Vec<_>>();
    surfaces.sort_unstable();
    surfaces
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
                    namespace: "xx",
                    name: "comet",
                    admission: SurfaceAdmission::Stable,
                },
                SemanticSurface {
                    namespace: "yy",
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
    fn public_reverse_projection_preserves_identity_across_admitted_surfaces() {
        for surface in admitted_surfaces_for_semantic_id("1003") {
            assert_eq!(
                crate::semantic_registry_export::semantic_id_for_admitted_surface(surface),
                Some("1003")
            );
        }
        assert_eq!(
            crate::semantic_registry_export::semantic_id_for_admitted_surface("not-a-surface"),
            None
        );
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

    #[test]
    fn admitted_surfaces_with_namespace_matches_admitted_names_and_keeps_namespace() {
        // Real registry row, not synthetic — 0001 is `quote`, checked
        // against the live lib/surface/semantic-registry.wsm.
        let with_namespace = admitted_surfaces_with_namespace_for_semantic_id("0001");
        let names_only = admitted_surfaces_for_semantic_id("0001");
        assert_eq!(with_namespace.len(), names_only.len());
        assert!(with_namespace.contains(&("en", "quote")));
        assert!(with_namespace.contains(&("uk", "як-є")));
        assert!(with_namespace.contains(&("sym", "'")));
    }
}

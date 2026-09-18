//! Shared projection from the language surface authority to compact byte SIDs.
//!
//! `lib/surface/semantic-registry.lisp` owns human/symbolic spellings and their
//! admission status. Its sr/2 rows carry exactly eight binary digits.
//! Runtime code stores that identity as one `u8`; the textual bit spelling is
//! provenance/serialization only and is never itself admitted as Lisp surface.
//! Evaluator meaning remains in the modules that interpret each SID.

use std::{collections::HashMap, sync::OnceLock};

const SEMANTIC_REGISTRY: &str = include_str!("../../../lib/surface/semantic-registry.lisp");

pub(crate) type SemanticId = u8;
pub(crate) const EMPTY_LIST_SEMANTIC_ID: SemanticId = 0;

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
    semantic_id: SemanticId,
    surfaces: Vec<SemanticSurface>,
}

fn parse_sid_bits(text: &str) -> Option<SemanticId> {
    if text.len() != 8 || !text.bytes().all(|byte| matches!(byte, b'0' | b'1')) {
        return None;
    }
    u8::from_str_radix(text, 2).ok()
}

pub(crate) fn semantic_id_bits(semantic_id: SemanticId) -> String {
    format!("{semantic_id:08b}")
}

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

fn surface_name_token(token: &'static str) -> &'static str {
    token
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(token)
}

fn parse_surface_group(group: &'static str) -> Option<SemanticSurface> {
    let fields = group.split_whitespace().collect::<Vec<_>>();
    match fields.as_slice() {
        [namespace, name] => {
            let name = surface_name_token(name);
            assert_ne!(
                name, "—",
                "missing surface must carry explicit missing status"
            );
            Some(SemanticSurface {
                namespace,
                name,
                admission: SurfaceAdmission::Stable,
            })
        }
        [namespace, name, status] => {
            let name = surface_name_token(name);
            assert_ne!(
                *status, "stable",
                "sr/2 admitted surfaces are implicit; do not spell stable"
            );
            if name == "—" {
                assert!(
                    matches!(*status, "missing" | "compatibility-only"),
                    "absent surface must be missing or compatibility-only"
                );
                return None;
            }
            match *status {
                "compatibility-only" => Some(SemanticSurface {
                    namespace,
                    name,
                    admission: SurfaceAdmission::CompatibilityOnly,
                }),
                "candidate" | "missing" => None,
                other => panic!("unknown sr/2 surface status: {other}"),
            }
        }
        _ => panic!("malformed sr/2 surface group: ({group})"),
    }
}

fn parse_rows(source: &'static str) -> Vec<SemanticRow> {
    source
        .lines()
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            let first = fields.first()?;
            let sid_token = first.strip_prefix("(\"")?.strip_suffix('\"')?;
            let semantic_id = parse_sid_bits(sid_token)?;
            let surfaces = surface_groups(line)
                .into_iter()
                .filter_map(parse_surface_group)
                .collect();

            Some(SemanticRow {
                semantic_id,
                surfaces,
            })
        })
        .collect()
}

fn assert_contiguous_byte_axis(rows: &[SemanticRow]) {
    assert!(!rows.is_empty(), "semantic registry must contain Canon 0");
    assert_eq!(
        rows[0].semantic_id, EMPTY_LIST_SEMANTIC_ID,
        "semantic registry must start at Canon 0 / SID 0"
    );
    for (expected, row) in rows.iter().enumerate() {
        assert_eq!(
            usize::from(row.semantic_id),
            expected,
            "semantic registry byte SIDs must be contiguous and ordered"
        );
    }
    assert!(
        rows.len() <= 256,
        "semantic registry must fit the declared 8-bit SID axis"
    );
}

fn insert_surface_mapping(
    index: &mut HashMap<&'static str, SemanticId>,
    surface: &'static str,
    semantic_id: SemanticId,
) {
    if let Some(previous) = index.insert(surface, semantic_id) {
        if previous != semantic_id {
            panic!(
                "semantic registry surface must be unique: {surface} maps to both {} and {}",
                semantic_id_bits(previous),
                semantic_id_bits(semantic_id)
            );
        }
    }
}

pub(crate) fn build_surface_index(
    source: &'static str,
) -> HashMap<&'static str, SemanticId> {
    let rows = parse_rows(source);
    let mut index = HashMap::new();
    for row in rows {
        for surface in row
            .surfaces
            .iter()
            .filter(|surface| surface.admission == SurfaceAdmission::Stable)
            .map(|surface| surface.name)
        {
            insert_surface_mapping(&mut index, surface, row.semantic_id);
        }
    }
    index
}

pub(crate) fn build_admitted_surface_index(
    source: &'static str,
) -> HashMap<&'static str, SemanticId> {
    let rows = parse_rows(source);
    let mut index = HashMap::new();
    for row in rows {
        for surface in row.surfaces.iter().map(|surface| surface.name) {
            insert_surface_mapping(&mut index, surface, row.semantic_id);
        }
    }
    index
}

fn live_rows() -> &'static [SemanticRow] {
    static ROWS: OnceLock<Vec<SemanticRow>> = OnceLock::new();
    ROWS.get_or_init(|| {
        let rows = parse_rows(SEMANTIC_REGISTRY);
        assert_contiguous_byte_axis(&rows);
        rows
    })
}

fn admitted_surface_index() -> &'static HashMap<&'static str, SemanticId> {
    static INDEX: OnceLock<HashMap<&'static str, SemanticId>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let _ = live_rows();
        build_admitted_surface_index(SEMANTIC_REGISTRY)
    })
}

pub(crate) fn admitted_semantic_id_for_surface(name: &str) -> Option<SemanticId> {
    admitted_surface_index().get(name).copied()
}

fn surface_index() -> &'static HashMap<&'static str, SemanticId> {
    static INDEX: OnceLock<HashMap<&'static str, SemanticId>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let _ = live_rows();
        build_surface_index(SEMANTIC_REGISTRY)
    })
}

fn stable_surfaces_from_index(
    index: &HashMap<&'static str, SemanticId>,
    semantic_id: SemanticId,
) -> Vec<&'static str> {
    let mut surfaces = index
        .iter()
        .filter_map(|(surface, mapped_id)| (*mapped_id == semantic_id).then_some(*surface))
        .collect::<Vec<_>>();
    surfaces.sort_unstable();
    surfaces
}

fn admitted_surfaces_from_rows(
    rows: &[SemanticRow],
    semantic_id: SemanticId,
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

pub(crate) fn semantic_id_for_surface(name: &str) -> Option<SemanticId> {
    surface_index().get(name).copied()
}

#[cfg(test)]
pub(crate) fn stable_surfaces_for_semantic_id_from_source(
    source: &'static str,
    semantic_id: SemanticId,
) -> Vec<&'static str> {
    stable_surfaces_from_index(&build_surface_index(source), semantic_id)
}

pub(crate) fn admitted_surfaces_for_semantic_id_from_source(
    source: &'static str,
    semantic_id: SemanticId,
) -> Vec<&'static str> {
    admitted_surfaces_from_rows(&parse_rows(source), semantic_id)
}

pub(crate) fn stable_surfaces_for_semantic_id(
    semantic_id: SemanticId,
) -> Vec<&'static str> {
    stable_surfaces_from_index(surface_index(), semantic_id)
}

/// Stable and compatibility-only spellings admitted for direct runtime binding.
/// Candidate/missing surfaces and the machine SID bit spelling itself are excluded.
pub(crate) fn admitted_surfaces_for_semantic_id(
    semantic_id: SemanticId,
) -> Vec<&'static str> {
    admitted_surfaces_from_rows(live_rows(), semantic_id)
}

pub(crate) fn admitted_surfaces_with_namespace_for_semantic_id(
    semantic_id: SemanticId,
) -> Vec<(&'static str, &'static str)> {
    let mut surfaces = live_rows()
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
        const SYNTHETIC: &str =
            "(\"00101010\" (xx comet) (yy meteor compatibility-only) (zz asteroid candidate) (qq — missing))";
        let parsed = parse_rows(SYNTHETIC);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].semantic_id, 42);
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
    #[should_panic(expected = "do not spell stable")]
    fn explicit_stable_status_is_rejected_by_sr2() {
        let _ = parse_rows("(\"00101010\" (xx comet stable))");
    }

    #[test]
    fn machine_sid_bit_spelling_is_not_a_lisp_surface() {
        const SYNTHETIC: &str =
            "(\"00101010\" (xx comet) (yy meteor compatibility-only))";
        let stable = build_surface_index(SYNTHETIC);
        let admitted = build_admitted_surface_index(SYNTHETIC);
        assert_eq!(stable.get("comet"), Some(&42));
        assert_eq!(stable.get("00101010"), None);
        assert_eq!(admitted.get("00101010"), None);
    }

    #[test]
    fn peer_namespaces_may_repeat_one_spelling_for_the_same_identity() {
        const SYNTHETIC: &str =
            "(\"00101010\" (uk comet) (ukr comet) (compat comet compatibility-only))";
        let stable = build_surface_index(SYNTHETIC);
        let admitted = build_admitted_surface_index(SYNTHETIC);
        assert_eq!(stable.get("comet"), Some(&42));
        assert_eq!(admitted.get("comet"), Some(&42));
    }

    #[test]
    fn stable_surfaces_are_constructively_selected_by_semantic_id() {
        const SYNTHETIC: &str =
            "(\"00101010\" (uk comet) (sa asteroid candidate) (sym +))";
        let index = build_surface_index(SYNTHETIC);
        assert_eq!(stable_surfaces_from_index(&index, 42), vec!["+", "comet"]);
        assert!(stable_surfaces_from_index(&index, 99).is_empty());
    }

    #[test]
    fn admitted_surfaces_include_compatibility_without_promoting_it_to_stable() {
        const SYNTHETIC: &str =
            "(\"00101010\" (en comet) (uk asteroid candidate) (compat meteor compatibility-only) (sa — missing))";
        let rows = parse_rows(SYNTHETIC);
        assert_eq!(
            admitted_surfaces_from_rows(&rows, 42),
            vec!["comet", "meteor"]
        );
        let stable = build_surface_index(SYNTHETIC);
        assert_eq!(stable.get("comet"), Some(&42));
        assert_eq!(stable.get("meteor"), None);
    }

    #[test]
    fn live_registry_is_one_contiguous_byte_axis_starting_at_canon_zero() {
        let rows = parse_rows(SEMANTIC_REGISTRY);
        assert_contiguous_byte_axis(&rows);
        assert_eq!(rows.len(), 168);
        assert_eq!(rows[0].semantic_id, 0);
        assert!(rows[0].surfaces.is_empty(), "Canon 0 is ground, not a surface spelling");
        assert_eq!(rows.last().map(|row| row.semantic_id), Some(167));
    }

    #[test]
    fn public_reverse_projection_preserves_identity_across_admitted_surfaces() {
        // division is the 15th non-ground row in the compact axis.
        for surface in admitted_surfaces_for_semantic_id(15) {
            assert_eq!(
                crate::semantic_registry_export::semantic_id_for_admitted_surface(surface),
                Some(15)
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
            "(\"00000001\" (xx collision))\n(\"00000010\" (yy collision))";
        let _ = build_surface_index(CONFLICTING);
    }

    #[test]
    fn unrelated_stable_rows_are_projected_without_assigning_evaluator_meaning() {
        assert_eq!(semantic_id_for_surface("+"), Some(12));
    }

    #[test]
    fn admitted_surfaces_with_namespace_matches_admitted_names_and_keeps_namespace() {
        let with_namespace = admitted_surfaces_with_namespace_for_semantic_id(1);
        let names_only = admitted_surfaces_for_semantic_id(1);
        assert_eq!(with_namespace.len(), names_only.len());
        assert!(with_namespace.contains(&("en", "quote")));
        assert!(with_namespace.contains(&("uk", "як-є")));
        assert!(with_namespace.contains(&("sym", "'")));
    }
}

//! Runtime projection of the numeric semantic surface registry.
//!
//! `lib/surface/semantic-registry.wsm` is the single machine authority for
//! human and symbolic surface spellings. This module does not invent aliases;
//! it only projects `stable` registry rows into a small immutable runtime view.
//! Evaluator mechanisms may then route by semantic ID instead of copying human
//! names into Rust tables.

use std::sync::OnceLock;

const SEMANTIC_REGISTRY: &str =
    include_str!("../../../../lib/surface/semantic-registry.wsm");

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

fn rows() -> &'static [SemanticRow] {
    static ROWS: OnceLock<Vec<SemanticRow>> = OnceLock::new();
    ROWS.get_or_init(|| parse_rows(SEMANTIC_REGISTRY)).as_slice()
}

/// Resolve either a pure numeric machine handle or a ratified stable surface
/// spelling to the semantic ID declared by `semantic-registry.wsm`.
pub(crate) fn semantic_id_for_surface(name: &str) -> Option<&'static str> {
    if !name.is_empty() && name.as_bytes().iter().all(|byte| byte.is_ascii_digit()) {
        return rows()
            .iter()
            .find(|row| row.semantic_id == name)
            .map(|row| row.semantic_id);
    }

    rows()
        .iter()
        .find(|row| row.stable_surfaces.contains(&name))
        .map(|row| row.semantic_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_machine_handles_resolve_only_when_the_registry_contains_them() {
        assert_eq!(semantic_id_for_surface("0010"), Some("0010"));
        assert_eq!(semantic_id_for_surface("0011"), Some("0011"));
        assert_eq!(semantic_id_for_surface("999999"), None);
    }

    #[test]
    fn stable_human_and_symbolic_surfaces_resolve_from_the_registry() {
        assert_eq!(semantic_id_for_surface("lambda"), Some("0010"));
        assert_eq!(semantic_id_for_surface("функція"), Some("0010"));
        assert_eq!(semantic_id_for_surface("визначити"), Some("0011"));
        assert_eq!(semantic_id_for_surface("+"), Some("0104"));
    }

    #[test]
    fn non_stable_registry_spellings_do_not_become_runtime_authority() {
        assert_eq!(semantic_id_for_surface("def"), None);
        assert_eq!(semantic_id_for_surface("rūpa"), None);
        assert_eq!(semantic_id_for_surface("—"), None);
    }

    #[test]
    fn parser_projects_stable_status_instead_of_hard_coding_surface_names() {
        const SYNTHETIC: &str = "(4242 (xx comet stable) (yy asteroid candidate) (zz — missing))";
        let parsed = parse_rows(SYNTHETIC);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].semantic_id, "4242");
        assert_eq!(parsed[0].stable_surfaces, vec!["comet"]);
    }
}

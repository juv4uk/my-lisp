//! Immutable routing for evaluator mechanisms necessary beyond Canon 0 + McCarthy7.
//!
//! Machine semantic authority lives in `lib/surface/semantic-registry.wsm`.
//! Rust owns only the mapping from numeric semantic IDs to evaluator mechanisms;
//! stable human/symbolic spellings are projected from the authority file at
//! runtime and are not duplicated in this module.

use std::sync::OnceLock;

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

fn registry_rows() -> &'static [SemanticRow] {
    static ROWS: OnceLock<Vec<SemanticRow>> = OnceLock::new();
    ROWS.get_or_init(|| parse_rows(SEMANTIC_REGISTRY)).as_slice()
}

fn semantic_id_for_surface(name: &str) -> Option<&'static str> {
    if !name.is_empty() && name.as_bytes().iter().all(|byte| byte.is_ascii_digit()) {
        return registry_rows()
            .iter()
            .find(|row| row.semantic_id == name)
            .map(|row| row.semantic_id);
    }

    registry_rows()
        .iter()
        .find(|row| row.stable_surfaces.contains(&name))
        .map(|row| row.semantic_id)
}

fn identity_for_semantic_id(semantic_id: &str) -> Option<NecessaryFormIdentity> {
    match semantic_id {
        DEFINE_SEMANTIC_ID => Some(NecessaryFormIdentity::Define),
        LAMBDA_SEMANTIC_ID => Some(NecessaryFormIdentity::Lambda),
        _ => None,
    }
}

/// Resolve an executable list-head symbol through the authority registry first,
/// then select the evaluator mechanism by numeric semantic ID. No human surface
/// spelling is duplicated in Rust routing data.
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
    fn registry_projection_resolves_unrelated_stable_rows_without_routing_them() {
        assert_eq!(semantic_id_for_surface("+"), Some("0104"));
        assert_eq!(identity_for_symbol("+"), None);
    }
}

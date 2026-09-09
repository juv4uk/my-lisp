//! Immutable routing for evaluator mechanisms necessary beyond Canon 0 + McCarthy7.
//!
//! Stable human/symbolic spellings are resolved by the shared semantic registry.
//! This module owns only the mapping from opaque numeric semantic IDs to the
//! evaluator mechanisms for DEFINE and LAMBDA.

use crate::semantic_registry;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NecessaryFormIdentity {
    Define,
    Lambda,
}

pub(crate) const LAMBDA_SEMANTIC_ID: &str = "0010";
pub(crate) const DEFINE_SEMANTIC_ID: &str = "0011";

fn identity_for_semantic_id(semantic_id: &str) -> Option<NecessaryFormIdentity> {
    match semantic_id {
        DEFINE_SEMANTIC_ID => Some(NecessaryFormIdentity::Define),
        LAMBDA_SEMANTIC_ID => Some(NecessaryFormIdentity::Lambda),
        _ => None,
    }
}

/// Resolve an executable list-head symbol through the shared authority registry,
/// then select the evaluator mechanism by numeric semantic ID.
pub(crate) fn identity_for_symbol(name: &str) -> Option<NecessaryFormIdentity> {
    semantic_registry::semantic_id_for_surface(name).and_then(identity_for_semantic_id)
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
    fn synthetic_registry_constructively_controls_necessary_form_routing() {
        const SYNTHETIC: &str = "(0010 (xx comet stable))\n(0011 (xx asteroid stable))";
        let index = semantic_registry::build_surface_index(SYNTHETIC);

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
    fn unrelated_registry_rows_do_not_gain_necessary_form_meaning() {
        assert_eq!(semantic_registry::semantic_id_for_surface("+"), Some("0104"));
        assert_eq!(identity_for_symbol("+"), None);
    }
}

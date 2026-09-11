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
    fn every_admitted_surface_for_define_and_lambda_is_a_registry_driven_peer_spelling() {
        // Which spellings mean "define"/"lambda" (визначити/define,
        // функція/lambda, ...) is a semantic-registry FACT, not Rust
        // knowledge to enumerate here -- this test asserts only the
        // implementation invariant: whatever surfaces the registry admits
        // for 0011/0010 all route through this same numeric-ID dispatch,
        // regardless of which language they're spelled in.
        for (semantic_id, identity) in [
            (DEFINE_SEMANTIC_ID, NecessaryFormIdentity::Define),
            (LAMBDA_SEMANTIC_ID, NecessaryFormIdentity::Lambda),
        ] {
            let surfaces = semantic_registry::admitted_surfaces_for_semantic_id(semantic_id);
            assert!(
                surfaces.len() >= 2,
                "{semantic_id} should admit at least two surfaces for this invariant to be \
                 meaningful, got {surfaces:?}"
            );
            for surface in &surfaces {
                assert_eq!(
                    identity_for_symbol(surface),
                    Some(identity),
                    "registry-admitted surface {surface:?} for {semantic_id} did not route to \
                     {identity:?}"
                );
            }
        }
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

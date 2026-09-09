//! Immutable routing for evaluator mechanisms necessary beyond Canon 0 + McCarthy7.
//!
//! Machine semantic authority lives in `lib/surface/semantic-registry.wsm`.
//! Rust owns only the mapping from numeric semantic IDs to evaluator mechanisms;
//! stable human/symbolic spellings are resolved constructively from the registry.

use super::semantic_registry;

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

/// Resolve an executable list-head symbol through the authority registry first,
/// then select the evaluator mechanism by numeric semantic ID. No human surface
/// spelling is duplicated in this module.
pub(crate) fn identity_for_symbol(name: &str) -> Option<NecessaryFormIdentity> {
    let semantic_id = semantic_registry::semantic_id_for_surface(name)?;
    identity_for_semantic_id(semantic_id)
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
}

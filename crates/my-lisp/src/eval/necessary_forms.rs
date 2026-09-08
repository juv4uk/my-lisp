//! Immutable routing for evaluator mechanisms necessary beyond Canon 0 + McCarthy7.
//!
//! Machine semantic authority lives in `lib/surface/semantic-registry.wsm`.
//! The Rust enum below selects an evaluator mechanism; it is NOT a semantic
//! identity registry. Each route is pinned to the numeric identity from the
//! authority file, and a unit test proves the small runtime spelling cache has
//! not drifted from that authority.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NecessaryFormIdentity {
    Define,
    Lambda,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NecessaryFormEntry {
    pub identity: NecessaryFormIdentity,
    pub semantic_id: &'static str,
    pub surfaces: &'static [&'static str],
}

pub(crate) const LAMBDA_SEMANTIC_ID: &str = "0010";
pub(crate) const DEFINE_SEMANTIC_ID: &str = "0011";

/// Closed evaluator-routing cache. Numeric IDs are the machine handles; human
/// spellings are direct peers. CI proves these rows equal the stable rows in
/// the numeric registry, so this cache cannot silently become a second authority.
pub(crate) const NECESSARY_FORMS: [NecessaryFormEntry; 2] = [
    NecessaryFormEntry {
        identity: NecessaryFormIdentity::Define,
        semantic_id: DEFINE_SEMANTIC_ID,
        surfaces: &["define", "визначити"],
    },
    NecessaryFormEntry {
        identity: NecessaryFormIdentity::Lambda,
        semantic_id: LAMBDA_SEMANTIC_ID,
        surfaces: &["lambda", "функція"],
    },
];

/// Resolve an executable list-head symbol. A pure numeric semantic handle and
/// every ratified stable human spelling enter the same evaluator mechanism.
pub(crate) fn identity_for_symbol(name: &str) -> Option<NecessaryFormIdentity> {
    NECESSARY_FORMS
        .iter()
        .find(|entry| entry.semantic_id == name || entry.surfaces.contains(&name))
        .map(|entry| entry.identity)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEMANTIC_REGISTRY: &str =
        include_str!("../../../../lib/surface/semantic-registry.wsm");

    fn stable_registry_names(identity: &str) -> Vec<&str> {
        let prefix = format!("  ({identity} ");
        let line = SEMANTIC_REGISTRY
            .lines()
            .find(|line| line.starts_with(&prefix))
            .unwrap_or_else(|| panic!("semantic registry must contain identity {identity}"));
        let fields = line.split_whitespace().collect::<Vec<_>>();
        let (triples, remainder) = fields[1..].as_chunks::<3>();
        assert!(remainder.is_empty(), "malformed registry row: {line}");

        triples
            .iter()
            .filter_map(|triple| {
                let name = triple[1];
                let status = triple[2].trim_end_matches(')');
                (status == "stable" && name != "—").then_some(name)
            })
            .collect()
    }

    #[test]
    fn necessary_forms_have_two_numeric_semantic_identities() {
        assert_eq!(NECESSARY_FORMS.len(), 2);
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
    fn ukrainian_and_english_names_are_direct_peer_spellings() {
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
    fn runtime_routing_cache_matches_numeric_registry_authority() {
        for entry in NECESSARY_FORMS {
            let mut from_registry = stable_registry_names(entry.semantic_id);
            let mut cached = entry.surfaces.to_vec();
            from_registry.sort_unstable();
            cached.sort_unstable();
            assert_eq!(
                cached, from_registry,
                "necessary-form routing cache drifted from semantic identity {}",
                entry.semantic_id
            );
        }
    }

    #[test]
    fn non_numeric_compatibility_spellings_do_not_gain_identity() {
        assert_eq!(identity_for_symbol("def"), None);
        assert_eq!(identity_for_symbol("#0010"), None);
        assert_eq!(identity_for_symbol("id0010"), None);
    }
}

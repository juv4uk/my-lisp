//! Immutable identities for the evaluator mechanisms that are necessary
//! beyond Canon 0 + McCarthy7.
//!
//! This registry is deliberately separate from `canon.rs`: the 0+7 canon
//! stays closed. These entries describe evaluator forms that cannot be
//! ordinary first-class value primitives because they control binding or
//! evaluation itself.
//!
//! Human spellings are peer surfaces of an identity. No spelling is the
//! canonical machine key. A missing human surface stays explicit in the
//! semantic registry instead of silently falling back through English.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NecessaryFormIdentity {
    Define,
    Lambda,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NecessaryFormEntry {
    pub identity: NecessaryFormIdentity,
    pub surfaces: &'static [&'static str],
}

/// Closed, immutable identity registry. There is intentionally no Environment,
/// setter, `define`, mutable static, or mutation API here.
///
/// UK and EN are already admitted as direct peer spellings. SA remains absent
/// here until its spelling is ratified; absence must never be filled by an EN
/// fallback.
pub(crate) const NECESSARY_FORMS: [NecessaryFormEntry; 2] = [
    NecessaryFormEntry {
        identity: NecessaryFormIdentity::Define,
        surfaces: &["define", "визначити"],
    },
    NecessaryFormEntry {
        identity: NecessaryFormIdentity::Lambda,
        surfaces: &["lambda", "функція"],
    },
];

pub(crate) fn identity_for_surface(name: &str) -> Option<NecessaryFormIdentity> {
    NECESSARY_FORMS
        .iter()
        .find(|entry| entry.surfaces.contains(&name))
        .map(|entry| entry.identity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn необхідні_форми_мають_дві_семантичні_тотожності() {
        assert_eq!(NECESSARY_FORMS.len(), 2);
        assert_eq!(
            identity_for_surface("define"),
            Some(NecessaryFormIdentity::Define)
        );
        assert_eq!(
            identity_for_surface("lambda"),
            Some(NecessaryFormIdentity::Lambda)
        );
    }

    #[test]
    fn українські_й_англійські_назви_є_прямими_peer_spellings() {
        assert_eq!(
            identity_for_surface("визначити"),
            Some(NecessaryFormIdentity::Define)
        );
        assert_eq!(
            identity_for_surface("define"),
            Some(NecessaryFormIdentity::Define)
        );
        assert_eq!(
            identity_for_surface("функція"),
            Some(NecessaryFormIdentity::Lambda)
        );
        assert_eq!(
            identity_for_surface("lambda"),
            Some(NecessaryFormIdentity::Lambda)
        );
    }

    #[test]
    fn сумісне_def_не_стає_семантичною_тотожністю_define() {
        assert_eq!(identity_for_surface("def"), None);
    }
}

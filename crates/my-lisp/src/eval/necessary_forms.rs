//! Immutable identities for the evaluator mechanisms that are necessary
//! beyond Canon 0 + McCarthy7.
//!
//! This registry is deliberately separate from `canon.rs`: the 0+7 canon
//! stays closed.  These entries describe evaluator forms that cannot be
//! ordinary first-class value primitives because they control binding or
//! evaluation itself.
//!
//! Historical note: `DEFINE` is grounded in Lisp 1.5's defining mechanism,
//! but my-lisp does not claim syntax-isomorphism with Lisp 1.5 `define[x]`.
//! Here the canonical identity is the act of introducing a binding into the
//! current lexical environment.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NecessaryFormIdentity {
    Define,
    Lambda,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NecessaryFormEntry {
    pub identity: NecessaryFormIdentity,
    pub canonical_surface: &'static str,
}

/// Closed, immutable registry.  There is intentionally no Environment,
/// setter, `define`, mutable static, or mutation API here.
pub(crate) const NECESSARY_FORMS: [NecessaryFormEntry; 2] = [
    NecessaryFormEntry {
        identity: NecessaryFormIdentity::Define,
        canonical_surface: "define",
    },
    NecessaryFormEntry {
        identity: NecessaryFormIdentity::Lambda,
        canonical_surface: "lambda",
    },
];

pub(crate) fn identity_for_surface(name: &str) -> Option<NecessaryFormIdentity> {
    NECESSARY_FORMS
        .iter()
        .find(|entry| entry.canonical_surface == name)
        .map(|entry| entry.identity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn necessary_forms_are_exactly_define_and_lambda() {
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
    fn compatibility_and_natural_language_names_are_not_kernel_surfaces() {
        assert_eq!(identity_for_surface("def"), None);
        assert_eq!(identity_for_surface("визначити"), None);
        assert_eq!(identity_for_surface("функція"), None);
    }
}

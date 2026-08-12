//! Semantic Atom Registry — Sanskrit migration Phase 2
//! (docs/sanskrit-semantic-migration.md §3).
//!
//! The single authoritative source of semantic atoms. Deliberately
//! minimal for Phase 2: this delivers the *registry mechanism* (the
//! `Atom` shape + lookup API) proven against the spec's own worked
//! example (`DHATU_DA`, §0/§3), not the full populated vocabulary —
//! populating the 12-dhātu core with exact per-atom semantics (§18) is
//! `SANSKRIT-P3-DHATU-CORE`'s job, and the six kāraka roles are
//! `SANSKRIT-P4-KARAKA-LAYER`'s. Keeping that boundary is what spec §34
//! means by "не виконувати всі фази одним commit".
//!
//! The load-bearing design rule from spec §3: **the semantic `id` is the
//! identity, never the SLP1 spelling.** `DHATU_DA -> dA` is correct;
//! treating `"dA"` itself as the identity is not — that would tie AST/
//! bytecode/ABI stability to orthography, which spec §3 exists to avoid.
//! `atoms_test_no_identity_is_its_own_spelling` below enforces this
//! mechanically, not just as a doc comment.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomCategory {
    Dhatu,
    Karaka,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomStatus {
    /// In the registry for pipeline validation but not yet vetted per
    /// spec §18's full exact-semantics writeup.
    Experimental,
    Stable,
    Deprecated,
}

#[derive(Debug, Clone, Copy)]
pub struct Atom {
    /// Stable identity — independent of spelling (spec §3). Never equals
    /// `slp1`.
    pub id: &'static str,
    /// Canonical ASCII storage spelling (spec §1: SLP1 is canonical).
    pub slp1: &'static str,
    /// Presentation-only romanization (spec §1: never an internal
    /// identifier). Must equal `transliteration::slp1_to_iast(slp1)` —
    /// enforced by test, so this can't silently drift from Phase 1's
    /// table.
    pub iast: &'static str,
    /// Presentation-only Devanāgarī spelling. Hand-verified against
    /// engineer-1's KARAKA-REFERENCE.md / PANINI-GRAMMAR-REFERENCE.md
    /// (themselves cross-checked against the Aṣṭādhyāyī), not invented —
    /// full SLP1<->Devanāgarī conversion is SANSKRIT-P2-DEVANAGARI-MAPPING,
    /// a separate task; these are literal, individually-sourced spellings.
    pub devanagari: &'static str,
    pub category: AtomCategory,
    /// Human-readable one-word gloss — a hint, never the formal semantics
    /// (spec §23: "Не використовувати англійський gloss як definition").
    pub gloss: &'static str,
    /// The formal operational semantics (spec §18). `SANSKRIT-P3` (dhātu)
    /// and `SANSKRIT-P4` (kāraka) own writing these out fully per-atom;
    /// Phase 2 entries carry a placeholder pending that pass, tracked via
    /// `status: Experimental`.
    pub semantics: &'static str,
    /// Legacy English names this atom subsumes (spec §13, feeds
    /// `SANSKRIT-P6-COMPAT-ALIASES`).
    pub aliases: &'static [&'static str],
    pub status: AtomStatus,
}

/// The registry. Deliberately one entry for Phase 2 — the spec's own
/// worked example, reused end-to-end by `SANSKRIT-P4`'s planned first
/// vertical slice (`docs/sanskrit-semantic-migration.md` §0, §35).
pub const REGISTRY: &[Atom] = &[Atom {
    id: "DHATU_DA",
    slp1: "dA",
    iast: "dā",
    devanagari: "दा",
    category: AtomCategory::Dhatu,
    gloss: "give",
    semantics: "transfer an entity from an agent (kartṛ) toward a recipient (sampradāna); required role karman, optional kartṛ/sampradāna — see spec §18 for the full field shape once SANSKRIT-P3 vets this beyond the worked example",
    aliases: &["give", "transfer", "send"],
    status: AtomStatus::Experimental,
}];

pub fn by_id(id: &str) -> Option<&'static Atom> {
    REGISTRY.iter().find(|a| a.id == id)
}

pub fn by_slp1(slp1: &str) -> Option<&'static Atom> {
    REGISTRY.iter().find(|a| a.slp1 == slp1)
}

pub fn by_alias(alias: &str) -> Option<&'static Atom> {
    REGISTRY.iter().find(|a| a.aliases.contains(&alias))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::transliteration::slp1_to_iast;

    #[test]
    fn no_atom_id_equals_its_own_slp1_spelling() {
        // Spec §3's explicit correct/incorrect example: DHATU_DA -> dA is
        // correct; identity == "dA" is not. Mechanically enforced.
        for atom in REGISTRY {
            assert_ne!(atom.id, atom.slp1, "atom {} must not use its SLP1 spelling as its identity", atom.id);
        }
    }

    #[test]
    fn registry_ids_are_unique() {
        let mut ids: Vec<&str> = REGISTRY.iter().map(|a| a.id).collect();
        ids.sort();
        let mut deduped = ids.clone();
        deduped.dedup();
        assert_eq!(ids, deduped, "duplicate atom id in registry");
    }

    #[test]
    fn every_atom_iast_field_matches_the_phase_1_transliteration_table() {
        // Prevents the registry's hand-written `iast` field from silently
        // drifting away from the verified Phase 1 table.
        for atom in REGISTRY {
            let computed = slp1_to_iast(atom.slp1).unwrap_or_else(|e| panic!("atom {} has invalid SLP1 `{}`: {e}", atom.id, atom.slp1));
            assert_eq!(atom.iast, computed, "atom {}'s stored IAST doesn't match Phase 1 transliteration of its SLP1 spelling", atom.id);
        }
    }

    #[test]
    fn spec_worked_example_dhatu_da_is_registered_correctly() {
        let atom = by_id("DHATU_DA").expect("DHATU_DA must be in the registry — it's the spec's own worked example");
        assert_eq!(atom.slp1, "dA");
        assert_eq!(atom.iast, "dā");
        assert_eq!(atom.devanagari, "दा");
        assert_eq!(atom.category, AtomCategory::Dhatu);
    }

    #[test]
    fn lookup_by_slp1_and_by_alias_agree_with_lookup_by_id() {
        let by_id_result = by_id("DHATU_DA").unwrap();
        let by_slp1_result = by_slp1("dA").unwrap();
        let by_alias_result = by_alias("give").unwrap();
        assert_eq!(by_id_result.id, by_slp1_result.id);
        assert_eq!(by_id_result.id, by_alias_result.id);
    }

    #[test]
    fn unknown_lookups_return_none() {
        assert!(by_id("DHATU_NONEXISTENT").is_none());
        assert!(by_slp1("zzz").is_none());
        assert!(by_alias("nonexistent-alias").is_none());
    }
}

//! Semantic vocabulary layer — Sanskrit/Pāṇinian migration.
//! Незалежний семантичний шар — санскритська/панініанська міграція.
//!
//! See `docs/sanskrit-semantic-migration.md` at the repo root for the full
//! specification and phase plan. Phases 1-4 live here: transliteration
//! (SLP1⟷IAST⟷Devanāgarī), the Semantic Atom Registry, the 12-dhātu core,
//! and the six kāraka roles + `SemanticCall` AST type. Still NOT wired
//! into the real parser/evaluator — `SemanticCall` values here are built
//! directly in tests, not produced by parsing SLP1 source syntax. That
//! pipeline (source -> tokenizer -> parser -> atom resolver -> this type)
//! is `SANSKRIT-P5-AST-SEMANTIC-IDS`'s job, per the task board's own phase
//! split (spec §34: don't do every phase in one commit).

pub mod atoms;
pub mod devanagari;
pub mod karaka;
pub mod transliteration;

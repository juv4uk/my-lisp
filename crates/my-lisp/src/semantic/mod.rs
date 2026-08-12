//! Semantic vocabulary layer — Sanskrit/Pāṇinian migration, Phase 1.
//! Незалежний семантичний шар — санскритська/панініанська міграція, Фаза 1.
//!
//! See `docs/sanskrit-semantic-migration.md` at the repo root for the full
//! specification and phase plan. This module currently contains ONLY
//! Phase 1 (transliteration): nothing here is wired into the parser,
//! evaluator, or AST yet — that starts at Phase 2 (Semantic Atom
//! Registry) and Phase 5 (AST semantic IDs). Building this in isolation
//! first is deliberate: the spec explicitly forbids starting the Dhātu
//! Core (Phase 3) before a verified, round-trip-tested transliteration
//! layer exists (spec §4, §22).

pub mod atoms;
pub mod transliteration;

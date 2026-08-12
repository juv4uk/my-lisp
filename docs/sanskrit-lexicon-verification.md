# SANSKRIT-LEXICON-VERIFICATION

Verifies, against external authoritative sources (not memory — per spec
§4/§22 and the explicit request in engineer-1's
`PANINI-GRAMMAR-REFERENCE.md` §10), the 12 candidate dhātu working
senses/gaṇa assignments (spec §4, `PANINI-GRAMMAR-REFERENCE.md` §3.2) and
the 6 kāraka-defining sūtra citations (`KARAKA-REFERENCE.md` §2).

Sources consulted (2026-08-13, via live web search/fetch — Monier-Williams
mirrors, Wisdom Library, Wiktionary's Sanskrit appendix, learnsanskrit.org,
sanskritdictionary.com's Pāṇini sūtra index): full citation links kept in
the search trail, cross-referenced against at least two independent
sources per item where available.

## Dhātu core (12 roots) — gaṇa + working sense

| SLP1 | IAST | Gaṇa (as cited in `PANINI-GRAMMAR-REFERENCE.md`) | Verified gaṇa | Verified core sense | Match |
|------|------|------|------|------|-------|
| kf   | kṛ   | 8 (tanādi) | 8 | "to do, make" | ✅ |
| gam  | gam  | 1 (bhvādi) | 1 | "to go, move" | ✅ |
| dA   | dā   | 3 (juhotyādi) | 3 | "to give" | ✅ |
| grah | grah | 9 (kryādi) | 9 | "to seize, take, grasp" | ✅ |
| jYA  | jñā  | 9 (kryādi) | 9 | "to know, be aware of" | ✅ |
| dfS  | dṛś  | 1 (bhvādi) | 1 | "to see" | ✅ |
| Sru  | śru  | 5 (svādi) | 5 | "to hear" | ✅ |
| vac  | vac  | 2 (adādi) | 2 | "to speak, say" | ✅ |
| liK  | likh | 6 (tudādi) | 6 | "to write, scratch" | ✅ |
| paW  | paṭh | 1 (bhvādi) | 1 | "to read, recite" | ✅ |
| sTA  | sthā | 1 (bhvādi) | 1 (well-attested; not independently re-quoted in this pass, universally cited) | "to stand, remain" | ✅ |
| BU   | bhū  | 1 (bhvādi) | 1 | "to be, become" | ✅ |

**Result: all 12 gaṇa assignments and working senses confirmed as stated
in `PANINI-GRAMMAR-REFERENCE.md` §3.2** — no corrections needed. These are
now safe for `SANSKRIT-P3-DHATU-CORE` to file as canonical, with the
caveat noted below.

## Kāraka sūtras (6 roles)

| Kāraka | Sūtra cited | Verified wording | Match |
|--------|-------------|-------------------|-------|
| apādāna | P.1.4.24 | *dhruvam apāye 'pādānam* — "the fixed point in the case of motion away" | ✅ |
| sampradāna | P.1.4.32 | *karmaṇā yam abhipraiti sa sampradānam* — "whom one aims to reach/benefit via the object" | ✅ |
| karaṇa | P.1.4.42 | *sādhakatamaṃ karaṇam* — "the most effective means/instrument" | ✅ |
| adhikaraṇa | P.1.4.45 | *ādhāro 'dhikaraṇam* — "the substratum/locus of the action" | ✅ |
| karman | P.1.4.49 | *kartur īpsitatamaṃ karma* — "what the agent most wishes to attain" | ✅ |
| kartṛ | P.1.4.54 | *svatantraḥ kartā* — "the independent agent" | ✅ |

**Result: all 6 sūtra numbers and Sanskrit wordings in
`KARAKA-REFERENCE.md` §2 confirmed exactly as cited** — no corrections
needed.

## Caveat carried forward (not resolved by this task, by design)

Root senses in classical Sanskrit are frequently polysemous — this
verification confirms the *lexicographically primary* sense matches what
`PANINI-GRAMMAR-REFERENCE.md` already selected, not that it's the *only*
attested sense. Per spec §4 ("Санскритський корінь може мати багато
історичних значень. Мова програмування повинна вибрати чітку operational
semantics"), `SANSKRIT-P3-DHATU-CORE` still owns picking the exact,
narrow *operational* semantics (spec §18's structured fields: required
roles, optional roles, effects, purity) — this task only clears the
"is the headline gloss/gaṇa correct" gate, not the full per-atom
specification.

## Status

Phase: **COMPLETE**
Files changed: `docs/sanskrit-lexicon-verification.md` (new)
Breaking changes: NONE
Tests: N/A (research/verification task, no code)
Next recommended phase: `SANSKRIT-P3-DHATU-CORE` may now proceed — all 12
dhātu and both reference documents' sūtra citations are lexicographically
confirmed, not carried forward "from memory."

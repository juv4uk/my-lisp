# Finding: my-lisp's SLP1 usage does not conflict with UPC-8's canonical phoneme ordering

Status: RESOLVED — no conflict found. `MYLISP-SLP1-VS-UPC8-CANONICAL-ORDER-CHECK`,
2026-08-18.

## What was checked

`shiva-sutras/prototype/upc8.py` assigns 8-bit hardware codes (`0x00`-`0x29`)
to the 42 unique sounds of the Śiva Sūtras, in canonical sūtra sequence
order — a specific, now mathematically proven-unique addressing scheme
(CP-SAT `UNSAT` proof, `M_min = 14`, see `shiva-sutras/RESEARCH_MAP.md`).
Checked whether `crates/my-lisp/src/semantic/{transliteration,devanagari}.rs`
implicitly assumes any different sound-to-code ordering that could
conflict once more `SANSKRIT-P*` work builds on `atoms.rs`.

## Result: no shared concern to conflict over

`transliteration.rs`'s `TABLE` and `devanagari.rs` do **character-level
SLP1 ⟷ IAST ⟷ Devanāgarī mapping only** — a fixed, standard SLP1 alphabet
(the same widely-used external convention UPC-8 also uses correctly:
`f`=vocalic ṛ, `x`=vocalic ḷ, `E`=ai, `O`=au, `N`/`Y`/`R`=the three
nasals, etc., verified identical letter-for-letter against
`upc8.py`'s `SIVA_SUTRAS` list). Neither `atoms.rs` nor either
transliteration module assigns its own numeric ordinal, address, or
hardware code to any individual sound — `my-lisp`'s semantic layer
operates on whole dhātu/kāraka atoms (morphemes/roots like `BU`, `gam`),
never on individual-phoneme addressing.

Since `my-lisp` never invented a phoneme-ordering scheme of its own,
there is no scheme for UPC-8's now-proven canonical one to conflict
with. The two projects operate at different granularities by
construction (whole-morpheme semantic atoms vs. individual-phoneme
hardware codes), not by any coordination that happened to avoid a
collision.

## What would actually create a conflict (for future reference)

If `my-lisp` (or `fpga-lisp`, downstream) ever needs to assign hardware
addresses to individual Sanskrit phonemes — e.g. an FPGA opcode encoding
a single sound rather than a whole dhātu — that work should reference
`shiva-sutras/prototype/upc8.py`'s canonical `0x00`-`0x29` assignment
directly rather than deriving a new one, per `docs/agent-doctrine.md`
rule 3 (don't duplicate a neighbor's semantics/proof as your own).

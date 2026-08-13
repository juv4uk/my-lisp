# SANSKRIT-P6-ALIAS-ENUMERATION

Prep work for `SANSKRIT-P6-COMPAT-ALIASES` (docs/sanskrit-semantic-migration.md
Phase 6), separable from the actual `alias_table` code per
`SANSKRIT-P1-DESIGN-DECISIONS`'s ruling (additive lookup, not a rewrite
of `eval/mod.rs`'s dispatch). This enumerates which of the 15 LANGUAGE
SEMANTICS builtins found in the Phase 0 audit map to one of the 12
`SANSKRIT-P3-DHATU-CORE` roots — and, honestly, which do not.

## Maps cleanly (already captured as `aliases` on the atom, P3)

| Builtin | Atom | Note |
|---|---|---|
| `lambda` | `DHATU_KF` (kṛ, "make") | constructs a procedure — a `kf`-family action |
| `print` | `DHATU_VAC` (vac, "say") | impure member of the `vac` family |
| `princ` | `DHATU_VAC` | same family, plain-text variant |
| `write-to-string` | `DHATU_VAC` | pure member of the same family (spec §18's `Purity: context dependent` field is exactly what distinguishes these three) |

These four are the entire clean-mapping set. All four already have their
English name listed in the relevant atom's `aliases` field in
`crates/my-lisp/src/semantic/atoms.rs` from P3 — nothing new to add here.

## Does NOT map — and should not be forced to

The remaining 11 LANGUAGE SEMANTICS builtins from the audit (`cons`,
`car`, `cdr`, `eq`, `cond`, `def`, `defmacro`, `+`/`-`/`*`/`/`,
`<`/`>`/`=`, and the string-op family that mirrors `cons`/`car`/`cdr`/`eq`)
do **not** correspond to any of the current 12 dhātu. This is not an
oversight to fix by stretching a root's meaning — the 12-root core was
drawn from the spec's own example dhātu list (§4), which was never chosen
to cover structural/comparison/arithmetic operations in the first place.
The roots those builtins *would* need (per the Phase 0 audit's own
suggestions) are not in the current core at all: `cons`/`car` would want
something like √grah or √yuj (join), `cdr` √śiṣ (remain), `eq`/comparisons
√tul (weigh/compare), `cond` √cit (discern), `def`/`defmacro` √dhā
(establish) + √kḷp, arithmetic √yuj/√kṣip/√nī/√guṇ. None of these are
among `kf/gam/dA/grah/jYA/dfS/Sru/vac/liK/paW/sTA/BU`.

Per spec §16's own ontology (`dhatu, karaka, entity, property, relation,
mathematical, structural, special-form, literal, type`), the honest
answer is that these 11 builtins simply belong to different categories:

| Category | Builtins |
|---|---|
| **structural** | `cons`, `car`, `cdr`, and their string-op mirrors (`string-append`, `string-first`, `string-rest`) |
| **relation** | `eq`, `<`, `>`, `=`, `string<?` |
| **mathematical** | `+`, `-`, `*`, `/` |
| **special-form** | `cond`, `def`, `defmacro` (bind/branch mechanics, not dhātu actions — consistent with `quote`'s classification in `SANSKRIT-P1-DESIGN-DECISIONS`) |

Spec §16 explicitly warns against forcing everything into dhātu form
("Санскритська модель не означає, що все має бути дієсловом") — this
enumeration is that principle applied concretely, not a gap to close later
without first deciding these categories deserve their own atom kinds
(a separate, larger design question, out of scope for this task).

## What P6 actually needs to alias, then

Given the above, `SANSKRIT-P6-COMPAT-ALIASES`'s `alias_table` has exactly
4 real entries to wire up from this analysis: `lambda -> DHATU_KF`,
`print -> DHATU_VAC`, `princ -> DHATU_VAC`, `write-to-string -> DHATU_VAC`.
Everything else in the current 12-dhātu core's `aliases` fields
(`give`/`transfer`/`send` for `dA`, `take`/`acquire`/`grab` for `grah`,
etc.) are *not* existing my-lisp builtin names — they are the atom's own
suggested vocabulary for code that might be written directly in SLP1/
Sanskrit syntax going forward, not legacy names being aliased.

## Status

Phase: research/enumeration only, no code changed.
Next: `SANSKRIT-P6-COMPAT-ALIASES` can now implement the 4-entry
`alias_table` with confidence it is not silently missing an
11th-hour-obvious mapping — the gap for `cons`/`car`/`cdr`/`eq`/etc. is
a documented, deliberate non-mapping, not an oversight.

# CML semantic export v1 — design (fixed before code)

Status: proposed 2026-09-10, per direct owner instruction on
`MYLISP-CML-SEMANTIC-EXPORT-V1`: *"компілятор має завжди бути
синхронним із Lisp. my-lisp лишається єдиним джерелом значення
програми; CML не повинен вручну дублювати або вибірково вигадувати
мовні правила. Перед кодом зафіксувати format, producer, consumer і
digest."* This document is that fixed-before-code step.

## Problem

`cml` currently derives its own understanding of which forms exist,
what they mean, and what to call them from reading `my-lisp` source and
docs by hand. That is exactly the "manually duplicate or selectively
invent language rules" the owner's instruction names as the failure
mode to close. `my-lisp` already has a single authoritative machine
source for this — `lib/surface/semantic-registry.wsm` (numeric semantic
IDs → admitted human/symbolic surfaces) plus `language-contract.my`
(the versioned Level 1/2 semantic contract) — but nothing currently
projects that authority into a form `cml` can consume without re-typing
it by hand.

## Format

A new file, `mylisp-cml-export.wsm`, schema-tagged like the existing
`.wsm` convention (`sr/1` for the semantic registry):

```
(cml-export/1
  (contract (major 6) (minor 0))
  (digest "<sha256 of the canonical serialization below, hex>")
  (forms
    (0001 (surfaces (en quote) (uk як-є) (sym ')) (role syntax) (callable nil))
    (0007 (surfaces (en cond) (uk за-умовою) (sym ?:)) (role syntax) (callable nil))
    (0010 (surfaces (en lambda) (uk функція)) (role syntax) (callable nil))
    (0011 (surfaces (en define) (uk визначити)) (role syntax) (callable nil))
    (0003 (surfaces (en eq) (uk тотожне?) (sym =?)) (role primitive) (callable t))
    (1001 (surfaces (uk відняти) (sym -)) (role library) (callable t))))
```

Design choices, each deliberate:

- **`.wsm`, not JSON/a new Rust struct dump.** Matches the existing
  `semantic-registry.wsm`/`guard-reference.wsm` convention this repo
  already committed to — one more machine format would itself be the
  kind of ungoverned duplication this task exists to prevent.
- **Only `admitted` surfaces (stable + compatibility-only), never
  `candidate`/`missing`.** Same admission filter `semantic_registry.rs`
  already applies for runtime binding — CML must never treat a
  not-yet-ratified surface as authoritative.
- **`role` and `callable` are new fields, not currently in
  `semantic-registry.wsm`.** They're the two facts CML's IR lowering
  actually needs per form (is this syntax the evaluator special-cases,
  or an ordinary callable value it can treat uniformly) and that
  `semantic-registry.wsm` doesn't carry today (it only tracks
  spelling/admission, per its own header: "Значення первинне"). Sourced
  from `language-contract.my`'s `special-forms-boundary` clause
  (`quote cond lambda def defmacro are NOT callable values`), not
  invented — this export's job is to make that existing contract fact
  machine-readable for a consumer, not to add a new rule.
- **`digest` over the `forms` block only**, not the whole file — so a
  comment-only or contract-metadata-only edit doesn't force every
  consumer to re-validate form data that didn't change. **FNV-1a
  (64-bit), hex-encoded** over the canonical serialization below, not
  SHA-256 — no `sha2`/crypto dependency exists anywhere in this
  workspace today, and this digest exists for drift-detection between
  trusted collaborators, not adversarial tamper-resistance; adding a
  crypto crate for that would be exactly the kind of unjustified new
  dependency `docs/agent-doctrine.md` rule 7 warns against. Revisit if
  a real cross-trust-boundary need for a cryptographic digest appears.
- **No `since-contract` per-form versioning in v1.** `conformance.my`
  already has this field for individual fixtures; the export's `forms`
  block is scoped to exactly the forms the first vertical slice needs
  (see below), so a consumer diffing the whole file against the
  contract's major/minor is sufficient for now. Revisit only if a real
  need for per-form version pinning shows up.

## Producer

`my-lisp` (this repo), via a new binary:
`crates/my-lisp-cli/src/bin/cml-export.rs`. Reads
`lib/surface/semantic-registry.wsm` and `language-contract.my` (both
already `include_str!`-able the way `semantic_registry.rs` does),
filters to the `forms` allow-list for the current vertical slice
(below), computes the digest, and writes `mylisp-cml-export.wsm`
deterministically — same inputs always produce byte-identical output
(no HashMap-iteration-order nondeterminism; sorted by semantic ID).

## Consumer

`cml`'s own tooling reads `mylisp-cml-export.wsm` (this repo owns
*producing* it; `cml` owns *consuming* it — no code changes to `cml`
happen from this repo, per rule 4 of `docs/agent-doctrine.md`, a
neighboring repo is an external authority, not a file I edit). `cml`
compares the `digest` against what it last validated against; a
mismatch means my-lisp's semantics moved and `cml`'s own IR-lowering
assumptions need re-checking before trusting them, not silently
continuing.

## First vertical slice: one real program shape

Per the task: "один реальний program shape (named def + recursion)".
Concretely, `tests/fixtures/conformance.my`'s own fixture #69 already
is exactly this shape and is already contract-ratified evidence, not a
new example invented for this export:

```lisp
(def count-down (lambda (n) (cond ((eq n 0) (quote done)) (t (count-down (- n 1))))))
(count-down 100000)
```

Forms this exercises, and therefore the v1 `forms` allow-list: `define`
(0011), `lambda` (0010), `cond` (0007), `eq` (0003), `quote` (0001),
subtraction (1001). Nothing else is in v1's export — a second vertical
slice adds more forms only once this one is proven end-to-end, per
`docs/agent-doctrine.md` rule 7 (minimize change surface).

## Evidence gate for this design doc

- Deterministic: running the producer binary twice on an unchanged
  tree produces byte-identical `mylisp-cml-export.wsm`.
- The six forms' surfaces and admission states match
  `semantic_registry.rs`'s own `admitted_surfaces_for_semantic_id` for
  each ID (cross-checked, not re-derived independently).
- `role`/`callable` for each form matches `language-contract.my`'s
  `special-forms-boundary` clause verbatim.

## Questions for the owner (per instruction, asked after building, not before)

1. Is `.wsm` the right home for this, or should a *contract-adjacent*
   export like this live next to `language-contract.my` instead (both
   are `my-lisp`-owned machine contracts, just different scopes)?
2. Should `mylisp-cml-export.wsm` be committed to this repo (so `cml`
   pulls it as a file), or served over the existing swarm-node/oracle
   TCP protocol as a query (so it's always live, never stale-by-commit-
   lag)? The design above assumes committed-file for v1 simplicity, but
   the "always in sync" instruction may want the live-query answer
   instead.
3. Once this vertical slice is proven, should slice 2 be chosen by
   `cml`'s own next real need, or should I propose the next
   conformance-fixture-shaped slice from this side?

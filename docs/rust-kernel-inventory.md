# Rust kernel inventory — WSM-RUST-DELETION-AUDIT

Status: minimal inventory, per the task's own explicit scope. **No code
deleted, no primitives added.** For each surface: can the behavior live
in WSM (`.my`/`.wsm` code), what specifically requires the Rust
substrate, and the evidence for that claim.

## `car` / `cdr` / `cons` / `eq` / `atom`

**Could live in WSM: no, not these five specifically — they're the
floor everything else, including WSM itself, is built from.**

Source-confirmed: `crates/my-lisp/src/eval/builtins.rs` registers all
five as ordinary environment bindings (`environment.define("car", ...)`)
at bootstrap, not as special-form syntax dispatch. Empirically confirmed
live this session: `(let ((car (lambda (x) 'shadowed))) (car '(1 2)))`
→ `shadowed`, matching `tests/fixtures/conformance.my`'s own current
fixture (`since-contract (2 1)`, note: "builtins are ordinary first-class
values bootstrapped into the global environment, so lexical bindings may
shadow them"). **This corrects a stale claim** in the companion
`docs/conformance-tier-map.md` (already self-flagged STALE there,
last synced ~fixture 91) that these five are "syntax, never shadowable" —
that described a pre-2.1 contract state, not the current one.

Being shadowable first-class values doesn't mean they could be *defined*
in WSM, though: `cons`/`car`/`cdr` are exactly the structural algebra G2
says everything else (lists, `lib/core.my`'s `list`/`map`/`filter`, etc.)
is built *from* — moving their own implementation into WSM would need
WSM primitives to build pairs with, which is circular. Rust substrate
requirement: the underlying `Value::Pair` heap representation and
allocation itself; `eq`'s identity semantics on that representation.

## `lambda`

**Could live in WSM: no.** `SYNTAX_FORMS` in `language_items.rs`
(`quote`, `lambda`, `def`, `defmacro`, `cond`, ...) are dispatched in
`eval/mod.rs` before any environment lookup — this is the one syntax
tier contract 2.1 kept genuinely special-form, per the same fixture note
above. Rust substrate requirement: closure capture (the environment
chain a closure holds), and per S3/the tail-call fixture
(`tests/fixtures/conformance.my`'s 100,000-deep `count-down` case), the
evaluator's own tail-call loop that keeps self-tail-recursion O(1)
host-stack regardless of depth — a property that has to live at the
evaluator's own call structure, not be re-derivable from WSM code
running on top of it.

## Arithmetic (`+`, `-`, `*`, `/`, comparisons)

**Could live in WSM: partially, but the exact-rational core cannot.**
Source-confirmed: `crates/my-lisp/src/eval/arithmetic.rs` implements
exact rational arithmetic (checked add/sub/mul, S1's exactness
guarantee, `NumericOverflow` on bit-limit) directly over Rust's
arbitrary-precision path. WSM-level code could express arithmetic
*policy* (e.g. how `average`/`clamp`/domain-specific numeric helpers
compose the primitives — much like `lib/core.my` already does for
list operations over `cons`/`car`/`cdr`), but the primitives themselves
— exact rational representation, overflow detection, the
exact+inexact→inexact promotion rule (S1) — need Rust's numeric types
and checked-arithmetic host support; re-implementing bignum/rational
math in WSM atop WSM would just relocate the same requirement one level
up, not remove it.

## `json-parse` / string operations

**Could live in WSM: the parsing/serialization logic conceptually
could; today it doesn't.** Source-confirmed: `Some("json-parse")` is
handled directly in `eval/mod.rs`, not delegated to a WSM library. No
strict Rust-only requirement identified here beyond convenience and
performance — a hand-written WSM JSON parser is possible in principle
(string manipulation + the existing primitives), just not attempted.
Flagged as a real candidate for a future `lib/json.my`, not acted on in
this audit (scope: inventory only, no code changes).

## `sha256-hex`

**Could live in WSM: no, not without WSM gaining bitwise/word-level
primitives it doesn't have.** Source-confirmed:
`crates/my-lisp/src/lib.rs::sha256_source` and the `sha256-hex`
builtin (`language_items.rs:160`) delegate to a real SHA-256
implementation over raw bytes — 32-bit word rotations, XOR, modular
addition at fixed word width. WSM's own numeric model (S1: exact
rationals, arbitrary precision) has no native fixed-width-word/bitwise
layer to build this from without inventing one — doing so would be
adding capability, explicitly out of this audit's scope.

## Host capabilities (`crates/my-lisp/src/eval/capabilities.rs`)

**Already correctly NOT in the core — this is the one category that's
already structured the way the audit would otherwise recommend.**
Source-confirmed: the canonical core ships zero host capabilities by
design (its own doc comment: "no filesystem, no processes, no
sockets"); a capability name that nothing has registered falls through
to ordinary function application and fails `UnknownSymbol`, same as any
unbound name (S2). Adapters (`my-lisp-host`, a WASM shim, an embedder)
install named handlers at startup. No Rust-deletion question applies
here — the architecture already separates "what the language core
requires" from "what an adapter chooses to expose," which is exactly
the boundary the rest of this inventory is trying to draw for the other
five categories.

## Summary table

| Surface | Lives in Rust because | WSM-movable? |
|---|---|---|
| car/cdr/cons/eq/atom | structural floor everything else builds from | no (foundational, not circular-safe) |
| lambda | closure capture + O(1)-stack tail-call loop | no (evaluator structure) |
| arithmetic core | exact-rational representation + overflow detection | no (numeric substrate); policy atop it — yes, already partly done in lib/core.my-style code |
| json-parse/string | convenience/performance today, no hard requirement found | plausible future WSM lib, not attempted |
| sha256-hex | needs fixed-width word/bitwise ops WSM doesn't have | no, without adding new WSM primitives (out of scope) |
| host capabilities | deliberately zero in core by design already | n/a — already correctly separated |

## Evidence

- `crates/my-lisp/src/eval/builtins.rs` (car/cdr/cons/eq/atom
  registration), `src/language_items.rs` (SYNTAX_FORMS,
  sha256-hex/car/cdr/cons/eq/atom doc entries),
  `src/eval/arithmetic.rs`, `src/eval/capabilities.rs`,
  `src/eval/mod.rs` (json-parse dispatch), `src/lib.rs` (sha256_source).
- `tests/fixtures/conformance.my` line 145 (the shadowing fixture,
  current/authoritative, `since-contract (2 1)`).
- Live oracle test this session: `(let ((car (lambda (x) 'shadowed)))
  (car '(1 2)))` → `shadowed`, confirming the fixture and correcting a
  stale reading from `docs/conformance-tier-map.md`.

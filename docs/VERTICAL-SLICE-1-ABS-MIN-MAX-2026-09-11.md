# First vertical slice: abs/min/max/min-list/max-list moved to Lisp

Per owner directive 2026-09-11 following `docs/BUILTIN-IDENTITY-MIGRATION-MAP-2026-09-11.md`:
"Lisp owns meaning, Rust owns only irreducible mechanism." Success
criterion is not "`Builtin` disappeared from the enum" — it's that Rust
code owning language behavior shrinks after each slice, not just moves
between files, and that swapping the Rust backend for CML/FPGA
wouldn't change my-lisp's own semantics or library behavior.

## Category (inventory step, done before picking a slice)

`abs`, `min`, `max`, `min-list`, `max-list` — category D (derived
operation): none touch OS/host capability, memory, sockets, or the
clock. Each is expressible entirely in terms of primitives the
language already has (`<`, `>`, `-`, `car`/`cdr`, `cond`, recursion).
Kept in `crates/my-lisp/src/eval/builtins.rs` as plain Rust closures
with no substrate reason — Rust-authority by default, not necessity.

## What moved

- **Deleted from `crates/my-lisp/src/eval/builtins.rs`**: five
  `define!(...)` blocks (the entire Rust implementation).
- **Deleted from `crates/my-lisp/src/eval/arithmetic.rs`**: `order_pair`
  — the shared helper only these five callers used, now genuinely dead,
  removed rather than left as unused code.
- **Deleted from `crates/my-lisp/src/language_items.rs`**: the five
  now-dead `builtin_metadata` match arms (they matched on
  `Value::Builtin` names that no longer exist for these five —
  `language_items()` only inspects `Value::Builtin`, so Lisp-defined
  functions were never going to be found there anyway, matching the
  pre-existing, already-accepted gap for `list`/`not`, migrated
  earlier).
- **Added to `lib/core.my`**: five real Lisp definitions.
- **Left untouched**: `lib/surface/semantic-registry.wsm`'s existing
  entries (`abs`=1004, `min`=1005, `max`=1006, `min-list`=1011,
  `max-list`=1012) — see "Real finding" below, these already existed
  and needed no change; `lib/surface/uk.my`'s existing peer bindings
  (`модуль`, `найменше`, `найбільше`, `найменше-у-списку`,
  `найбільше-у-списку`) — already pointed at these names, now resolve
  to the Lisp definitions automatically.

## Two real bugs found during migration (not assumed, live-tested)

1. **`eq` is not a safe empty-list check for a non-atom value.** First
   draft of `min-list`/`max-list` used `(eq items (quote ()))`; `eq`
   requires both operands to be atoms and raised `eq expects two
   atoms` on every non-empty-list recursive call. Fixed with `atom`,
   the idiom already used elsewhere in `lib/core.my` (e.g. `equal?`'s
   own `(atom rest)`).
2. **`atom` is not a safe "is this the recursion's base case" check for
   the *accumulator*.** `(atom rest-min)` was meant to detect "the
   recursive call bottomed out and returned the empty-list sentinel" —
   but a real numeric answer is *also* an atom, so this collapsed
   "sentinel" and "valid result" into the same branch, silently always
   taking the base case. `(min-list '(5 2 8 1))` returned `5` (the
   first element) instead of `1` until this was caught by testing
   real, non-trivial input rather than only the empty-list edge case.
   Fixed with `equal?` against the literal `(quote ())`, safe because
   the accumulator here is always either a number or the empty-list
   value, never ambiguous with a cons structure.

## Real finding: the registry already had these IDs; Rust never read them

`abs`/`min`/`max`/`min-list`/`max-list` already had real numeric
semantic identities in `lib/surface/semantic-registry.wsm` before this
migration — including ratified Sanskrit spellings for three of them
(`rūpa`, `alpatara`, `brhattara`). The deleted Rust code used the plain
`define!` macro (unconditional environment binding by literal name),
never `define_peer_builtin` (the registry-aware path other builtins
use) — so these numeric identities existed on paper but were never
consulted by the implementation that claimed to provide them. This
migration closes that second gap as a side effect of moving the
implementation, not as separate extra work.

**Process note, kept for the next agent doing a slice like this**: a
first draft of this migration nearly *re-added* these five as new IDs
(`1147`-`1151`), overwriting the real Sanskrit spellings with `missing`
placeholders, because the registry wasn't checked directly before
assuming the entries didn't exist. Caught before committing by
grepping the actual file, not from memory of what "should" be there —
worth remembering as its own lesson: verify the registry state
directly for every name in a slice, don't infer it from what the
deleted Rust code did or didn't reference.

## Verification (parity witness before deletion, not after)

`crates/my-lisp/tests/builtin_to_lisp_migration.rs`, 6 tests, all
passing:

- `abs` on integers, rationals, negatives, zero.
- `min`/`max` variadic behavior, including single-argument and
  all-equal cases.
- Zero-argument `min`/`max` still raise a named `Arity` error — via a
  required first parameter (dotted lambda-list `(first . rest)`, the
  same pattern `<=`/`>=` already use in `lib/core.my`), not a
  hand-written check.
- `min-list`/`max-list` on real lists and single-element lists.
- `min-list`/`max-list` on the empty list return `()` (Nil), distinct
  from `min`/`max`'s own zero-*argument* Arity error.
- The five Ukrainian peer spellings (`модуль`, `найменше`, `найбільше`,
  `найменше-у-списку`, `найбільше-у-списку`) resolve to the same
  migrated definitions, loaded through the real prerequisite chain
  `crates/my-lisp-cli/src/repl.rs` uses for the `uk` surface (not just
  `core.my` — a first draft of this test failed on an unresolved
  `map-empty` from `persistent-map.my` before the full chain was
  reproduced).

`cargo clippy -p my-lisp -p my-lisp-cli -p my-lisp-lsp --all-targets --
-D warnings`: clean.

## What this slice does NOT do (scope discipline, per the directive)

- Does not touch any host-capability builtin (`read-file`,
  `process-run-raw`, `tcp-*`, clock primitives) — those are the
  irreducible-mechanism category (A/B), not this slice's target.
- Does not attempt `Value::SemanticRef` or any `apply`-dispatch
  redesign — this slice proves the *pattern* (identify, verify parity,
  move, delete, keep registry honest) on the simplest safe candidates,
  not the full architecture change.
- Does not touch `Value::Builtin`'s `eq` semantics (the reverted work
  from the previous commit) — that redesign question is unrelated to
  whether any *specific* function belongs in Rust or Lisp.

## Candidates identified for a next slice, not started here

Per the same category-D reasoning: `abs`'s neighbors in
`builtins.rs` were the simplest already-isolated group. Other
`builtins.rs` entries worth auditing next (not analyzed in depth for
this document — flagged, not scoped): `string<?`, `string-append`,
`string-first`/`string-rest` (string manipulation with no host I/O,
similar shape to this slice) versus `sha256-hex`/`json-parse` (likely
irreducible or borderline — would need their own read before
classifying).

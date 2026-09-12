# Vertical slice 1 closure: three debts closed

Follow-up to `docs/VERTICAL-SLICE-1-ABS-MIN-MAX-2026-09-11.md`. Review
verdict: direction right (real Rust code deleted, not just moved), but
the slice wasn't fully closed — three tails of the old architecture
remained. New standing principle established for every future slice:

> **Semantic truth tests live with the language; Rust tests only
> mechanism.**

## Debt 1: stale docs/inventory (closed)

`docs/FUNCTIONS.md` still listed `abs`/`min`/`max`/`min-list`/`max-list`
under "Builtin'и ядра (live env)" (35 root builtins) instead of the
`core.my` library section. `lib/surface/uk-inventory.wsm` had the same
five names classified under `root-builtins` instead of `core-library`.

Fixed:
- `docs/FUNCTIONS.md`: moved the five names to the `core.my` list,
  corrected the root-builtin count (35 → 30, confirmed live via
  `scripts/gen-functions.my`'s own `#<builtin ...>` print-shape filter
  no longer matching them), corrected the `core.my` count (52 → 57),
  updated the refresh timestamp/base commit.
- `lib/surface/uk-inventory.wsm`: moved the five names from
  `root-builtins` to `core-library`, with a dated comment explaining
  why.

## Debt 2: Rust semantic tests duplicating language truth (closed)

`crates/my-lisp/tests/builtin_to_lisp_migration.rs` asserted the actual
*behavior* of `abs`/`min`/`max`/`min-list`/`max-list` in Rust
(`assert_eq!(eval_source("(abs -5)"), "5")` and five more like it) —
Rust owning knowledge of what the language's own behavior should be,
exactly the pattern this whole migration exists to move away from.

Fixed:
- Added 13 tier-3 fixtures to `tests/fixtures/conformance.my` covering
  the same behaviors (values, variadic arity, the zero-argument Arity
  error, empty-list handling) — this is now Lisp-owned data, not Rust
  assertions, consistent with `conformance.my`'s own stated role as
  "the implementation-independent contract."
- Added `lib/surface/peer-identity-acceptance.my`, a self-verifying
  Ukrainian program (same pattern as the existing
  `lib/surface/uk-acceptance.my`) proving `(тотожне? модуль abs)` etc.
  hold — this needed its own file rather than a `conformance.my` entry
  because it inherently compares bindings across two loaded surface
  files (`core.my` + `uk.my`), which `conformance.my`'s single-session
  execution model doesn't accommodate (checked directly: adding it
  there would have produced an `UnknownSymbol` result when
  `scripts/build-inventory.my`/`scripts/oracle-batch.wsm` regenerate
  against their own base-`core.my`-only session, contradicting a
  hardcoded `expected . "t"`).
- Added `peer_identity_acceptance_program_passes` to
  `crates/my-lisp/tests/uk_sa_surface.rs`, mirroring the existing
  `uk_acceptance_program_passes` exactly: a generic runner that
  evaluates the `.my` file and checks for the literal symbol `успіх` —
  it has no knowledge of which specific functions or values are being
  checked; that knowledge lives entirely in the Lisp file.
- Deleted `crates/my-lisp/tests/builtin_to_lisp_migration.rs` entirely
  — once its behavioral content moved to Lisp-owned data/programs and
  a generic runner, nothing mechanical remained to justify keeping a
  Rust file that knew these five functions by name.

`tests/fixtures/inventory.my` and `my-lisp-constitution.my`
(both `conformance.my` projections, GENERATED — do not hand-edit)
regenerated to match the grown fixture count (226 → 239 forms), twice
(once after the first edit, a second time after the 13 new fixtures
were added — caught by `crates/my-lisp/tests/mccarthy.rs`'s own
`constitution_my_stays_in_sync_with_conformance_my` test, which failed
with an exact count mismatch until this was done). `tests/fixtures/oracle-results.my`
(the third file in this generated chain) could **not** be regenerated
in this pass — it requires a live TCP oracle server at `:9999`, not
available in this environment. Two of its entries are now orphaned
(stale `F-<id>`s for the reframed `+`/`-` fixtures below), and the two
new `F-<id>`s those fixtures now have carry no oracle-results coverage
yet. Flagged here explicitly rather than silently left inconsistent —
whoever next has access to a live oracle server should re-run
`scripts/oracle-batch.wsm`.

## Debt 3: `Value::Builtin` print format pinned as contract (closed)

`tests/fixtures/conformance.my` had two tier-2, `since-contract (2 1)`
fixtures asserting `+` and `-` evaluate (unapplied) to the literal
string `"#<builtin +>"` / `"#<builtin ->"`. This is display
*representation* — explicitly the kind of thing
`docs/COMPILER-AUTHORITY-BOUNDARY.md` (#66) says a compiler backend is
free to vary — pinned as if it were the actual contract fact (that a
bare `+`/`-` reference is a first-class, callable value).

Fixed: reframed both fixtures to prove the same underlying contract
fact (first-classness) without coupling to the print format —
`((lambda (f) (f 3 4)) +)` → `"7"` passes the bare builtin as an
ordinary value and calls it, which only works if it really is
first-class, independent of how it prints. Verified live before
committing (not assumed): `7` and `6` respectively. Regenerated
`my-lisp-constitution.my`/`tests/fixtures/inventory.my` to match (their
own diffs confirm only these two fixtures' `expr`/`expected`/`note`
fields changed, `since-contract`/`requires`/`axioms` tags preserved
unchanged, so this remains evidence for the same ratified contract
clause, just via a representation-independent observation).

## Verification

- `cargo clippy -p my-lisp --all-targets -- -D warnings`: clean.
- `cargo test -p my-lisp --test mccarthy`: 76/76 (including the
  constitution-sync check that initially caught the drift from adding
  fixtures without regenerating).
- `cargo test -p my-lisp --test uk_sa_surface peer_identity_acceptance_program_passes`:
  passes.
- `cargo test -p my-lisp --test first_class_builtins`: 9/9 (unaffected
  by the `+`/`-` fixture reframing, since that file's own tests never
  depended on `#<builtin ...>`'s literal string).
- `cargo test -p my-lisp --test compiler_oracle_corpus` and `--lib ir::`:
  unaffected (neither fixture is tagged `compiler-corpus . t`, so
  #67/#68's frozen corpus is untouched by this closure).

## What remains genuinely open, not fixed here (named, not hidden)

- `tests/fixtures/oracle-results.my` needs a live oracle server to
  catch up, per Debt 2 above.
- The deeper architectural question the review raised — whether
  `(define модуль abs)`-style peer binding (verified live: `(eq модуль
  abs)` → `t`, real shared `Rc` identity, not a re-implementation) is
  sufficient, or whether ordinary (non-Canon) library functions need
  their own semantic-ID-first construction mechanism mirroring Canon's
  Rust-side `CANON_VALUES`/`materialize_value` — is **not** resolved
  here. The existing peer-binding already satisfies
  `docs/PLAN-FULL-LANGUAGE-PARITY.md`'s Etap D text literally (one
  value, all surfaces peer-bound to it, verified via `eq`) and building
  a new Lisp-level semantic-ID-keyed definition mechanism for ordinary
  library functions would be new infrastructure, not a slice-closure
  task — flagged for a future, deliberately scoped decision rather than
  built speculatively here.

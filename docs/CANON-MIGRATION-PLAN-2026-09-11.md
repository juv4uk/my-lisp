# Canon + function-table migration plan (2026-09-11)

Owner directive: a systematic plan (this document — no implementation
here) for how every consumer of `lib/surface/semantic-registry.wsm`
(my-lisp itself, wsm-my-lisp, cml, fpga-lisp) should dispatch/recognize
symbols through canonical numeric semantic IDs plus a generated
projection of the real registry, not hardcoded ASCII spellings or
hand-typed duplicate tables. This generalizes four already-found real
bugs, not a hypothetical concern:

1. `crates/my-lisp-lsp/src/analysis.rs` — `quote`/`cond` recognized by
   literal English string (fixed, commit `db825e7`).
2. wsm-my-lisp's `dll/src/eval.rs` — same class, English-only
   (fixed, commit `c3bf0f4`, then generator-ized in `55b3156`).
3. cml's `src/lower.rs` / `src/semantic.rs` — English-only recognition
   plus a hand-transcribed duplicate of the registry's Canon 0+7 rows
   (`semantic.rs` partially fixed via generator in `26e20e7`; `lower.rs`
   itself explicitly **not yet fixed**, per that commit's own note).
4. cml's `lower.rs` also merges case-different quoted symbols
   (`radio`/`RADIO` → the same value) during codegen — a distinct bug
   (exact-value preservation, not surface-spelling recognition), tracked
   as `cml#13`, P0, open.

## The principle, stated once so every consumer can point at it

`lib/surface/semantic-registry.wsm` is my-lisp's sole authority for
"which numeric semantic identity does a spelling belong to, across
which language surfaces (en/uk/sa/sym)." Any project that needs to
recognize a language form (quote, cond, lambda, define, defmacro, or
any admitted primitive/library identity) by name must derive that
recognition from the real registry file at build or run time — never
hand-type a spelling list, and never special-case only the English
surface. This is not a new rule; it is `docs/COMPILER-AUTHORITY-BOUNDARY.md`
(#66) applied to name-recognition specifically, and it already has two
independent, working implementations to point to (see below) — this
plan does not invent a new mechanism, it generalizes one already proven
twice.

**Explicit scope boundary, confirmed with fpga-lisp this session**: the
registry covers *language-owned forms and admitted callable identities*
only — `quote`, `cond`, `atom`, arithmetic, library functions. It does
**not**, and must never be asked to, cover arbitrary user-level data
symbols (a quoted `radio`, a user's own function name). Symbol-identity
preservation for *those* (cml#13) is a separate correctness property —
exact-value fidelity — not a semantic-ID coverage question. Keeping
these two questions distinct is itself part of this plan, since
conflating them already cost fpga-lisp real investigation time this
session before the distinction was made explicit.

## Precedent: two working implementations already exist

Both follow the same shape — read this bit before designing a third
variant from scratch:

- **wsm-my-lisp** (`dll/build.rs`, commit `55b3156`): a ~180-line
  minimal S-expression tokenizer/reader (not a general Lisp parser —
  the registry's own shape doesn't need one), reading
  `external/my-lisp/lib/surface/semantic-registry.wsm` (a git
  submodule) at build time, emitting `$OUT_DIR/canon_spellings.rs` as
  `const &[&str]` slices for exactly the IDs `eval.rs` special-cases
  (`0001` quote, `0007` cond — deliberately narrow, per the owner's own
  instruction not to import the whole registry speculatively). Fails
  the build (not a silent empty list) if the file is missing, an
  expected ID isn't found, or an ID resolves to zero surfaces.
- **cml** (`build.rs`, commit `26e20e7`): the same shape, reading
  my-lisp as a sibling checkout (already present in CI for conformance
  fixtures, so no CI change needed — contrast with wsm-my-lisp's
  submodule case, which *did* need a CI checkout fix, found and fixed
  before the first red build). Covers Canon `0001`-`0007` (all seven,
  not just quote/cond, since `semantic.rs`'s `is_reserved_canon_surface`
  needed all seven for shadowing checks). Explicitly does **not** cover
  `0010`/`0011`/`0012` (lambda/define/defmacro) yet, and explicitly does
  **not** fix `lower.rs`'s own English-only special-form recognition —
  both named as open in that commit's own message.

Neither implementation imported the whole registry generically "for
future use" — both scoped the generator to exactly the IDs the
consumer's existing code already special-cased. That restraint is
itself the pattern to keep, not an accident to fix later.

## Where my-lisp itself stands (no generator needed, and why)

my-lisp doesn't need a build.rs/generated-projection pattern for its
*own* internal code, because it holds the real registry directly, not
a copy — `crates/my-lisp/src/eval/canon.rs`'s `identity_for_surface`
and `crates/my-lisp/src/semantic_registry.rs`'s
`semantic_id_for_surface` already resolve any spelling to its numeric
identity at zero remove. The LSP bug (finding #1 above) wasn't a
missing-generator problem — it was code that bypassed those existing
resolvers with a hardcoded string check. The fix (`my_lisp::is_quote_surface_name`,
a thin public wrapper over the same internal resolver) is the pattern
**other in-repo/in-workspace consumers of this crate** should follow:
call the real resolver through a narrow public API, don't duplicate its
data. The generator pattern (wsm-my-lisp, cml) is specifically for
consumers that are a **separate repository/build**, where "just call
the function" isn't available and the registry file has to be read as
data instead.

## Concrete steps, in order

### Step 1 (cml, in progress) — finish `lower.rs`

`cml#9`'s own text names this as still open: `semantic.rs`'s
shadowing-check is generator-derived, but `lower.rs`'s actual
special-form dispatch (deciding "is this call a `cond`, a `lambda`, an
ordinary application") still recognizes forms by hardcoded English
spelling. Extend the *existing* `build.rs` (don't write a second
generator) to also emit `0010`/`0011`/`0012` (lambda/define/defmacro)
spellings, and change `lower.rs` to check membership in the generated
sets instead of literal strings. This is cml's own work; I'll confirm
correctness against the real registry/oracle if asked, per our
established pattern this session.

### Step 2 (cml, independent, already tracked) — fix `cml#13`

The `radio`/`RADIO` merge is a distinct bug from Step 1: it's about
preserving exact quoted-symbol *values* through codegen, not about
recognizing *form names*. Generating spellings from the registry does
not fix this — it needs its own fix in whatever code path uppercases
identifiers for the x86 target's own naming constraints, keeping a
reverse mapping (or avoiding the transform entirely for symbol *data*,
as opposed to symbol *names used as asm labels*) so `eq?` and printing
still observe the original case. Already correctly opened as its own
P0 issue; not blocked on, or blocking, Step 1.

### Step 3 (wsm-my-lisp, optional extension) — widen scope if new forms are special-cased

wsm-my-lisp's generator currently covers only quote/cond because
`eval.rs` only special-cases those two today. No action needed *unless*
a future change makes `eval.rs` recognize a third form by name (e.g.
if `lambda`/`define` ever need surface-spelling recognition outside
their own dedicated dispatch) — at that point, extend the existing
`TARGET_IDS`-style list by one entry, per the generator's own design
(already generic over "which numeric id").

### Step 4 (fpga-lisp, not yet started) — adopt the pattern proactively

Confirmed directly this session: `fpga-lisp/gen_symbol_table.py:32` has
`SPECIAL_FORMS = {50: "quote", 80: "cond"}` — hardcoded, English-only,
the same bug class as the other three, just not yet triggered by a
reported failure. Recommend fpga-lisp write its own minimal
registry-reading step (Python, not Rust — the same minimal-parser
approach, adapted to the language fpga-lisp's own tooling is written
in) that derives this dict from the real `semantic-registry.wsm` at
generation time, fail-closed the same way, scoped to exactly the two
IDs `gen_symbol_table.py` already special-cases (following the same
narrow-scope discipline both existing implementations used, not
importing the whole registry speculatively). This is proactive, not
reactive — fpga-lisp has not yet had a reported case where a non-English
spelling reached this dict incorrectly, but the shape of the bug is
identical to the three already found, and the fix is well-precedented
enough now that writing it before a bug report is cheap.

### Step 5 (all consumers, ongoing) — keep the scope boundary explicit

Whenever a consumer's generator or resolver is extended, re-state (in
its own commit message or doc, as the three existing precedents
already do) that arbitrary user-level data symbols remain outside the
registry's scope — this is not a limitation to work around, it's the
correct boundary, and the fpga-lisp/cml#13 exchange this session shows
how easily "does the registry cover X" gets asked about the wrong X
when this isn't stated plainly at each site.

## Coordination protocol going forward

- Each consumer's generator work is that consumer's own implementation
  — per the standing repo-boundary discipline (`docs/agent-doctrine.md`
  rule 4), I do not write wsm-my-lisp's/cml's/fpga-lisp's build.rs or
  Python scripts myself.
- I remain available to verify: (a) that a generated projection's
  output matches the real registry/oracle for the specific IDs it
  claims to cover, and (b) whether a given case is genuinely a
  semantic-ID question or, like cml#13, a different correctness
  property the registry was never meant to address.
- Each project should link back to this document (or its own
  precedent commit) rather than re-deriving the pattern independently,
  per the same anti-duplication principle the pattern itself is about.

# UKR Rebase and Public Inventory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the `uk`/`ukr` Ukrainian surface cleanly on the current canonical `.lisp` main branch, then add exhaustive public-API inventory infrastructure without a giant translation PR.

**Architecture:** First supersede conflict-heavy PR #101 with a fresh branch from current `main`, porting only its semantic/runtime/docs behavior to canonical `.lisp` paths. After that merge, add an exhaustive scanner that discovers every top-level `def`/`defmacro` under `lib/**/*.lisp` and reports coverage; only after an explicit baseline classification is reviewed do we turn `unclassified` into a hard CI failure. Translation proceeds by small domain PRs, each updating registry, runtime witness, generated API docs, and README/navigation in one slice.

**Tech Stack:** Rust workspace tests, Python 3 standard library, my-lisp `.lisp` generators/data, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-09-13-full-public-ukrainian-surface-design.md`

## Global Constraints

- `lib/surface/semantic-registry.lisp` remains the only spelling ↔ numeric semantic-ID authority.
- `uk` = compact and intuitively decodable Ukrainian spelling.
- `ukr` = full Ukrainian spelling; stable `ukr` identifiers contain no ASCII Latin letters.
- `full-uk` is not a runtime namespace or table column.
- Public Lisp library operations eventually receive numeric semantic IDs; internal helpers do not receive IDs merely for translation.
- C/Rust embedding symbols are not translated unless they are exposed as Lisp-callable user operations.
- README and `docs/ukrainian-api.md` are part of Definition of Done for every public translation slice.
- Existing numeric semantic IDs are append-only and never reused for a different meaning.
- No giant all-at-once translation rename; merge domain slices independently.

---

### Task 1: Supersede PR #101 on canonical `.lisp` main

**Files:**
- Create branch: `feat/ukr-surface-lisp-main` from current `main`
- Modify: `lib/surface/semantic-registry.lisp`
- Modify: `scripts/generate-function-table.lisp`
- Modify: `lib/generated/function-table.lisp`
- Modify: `lib/generated/meta-semantic-registry.lisp`
- Modify: `crates/my-lisp/src/semantic_registry.rs`
- Modify: `crates/my-lisp/src/lib.rs`
- Test: `crates/my-lisp/tests/ukr_acceptance.rs`
- Test: `crates/my-lisp/tests/uk_docs_surface_model.rs`
- Test: `crates/my-lisp/tests/macro_derivation.rs`
- Modify: `docs/ukrainian-api.md`
- Modify: `README.md`
- Modify: `.github/workflows/full-uk-surface-check.yml`

**Interfaces:**
- Consumes: current numeric semantic registry schema `sr/1`; current core-library bootstrap; current `.lisp` source layout.
- Produces: authoritative `ukr` surface rows; generated table columns `uk | ukr | en | sa | sym`; executable `ukr` bootstrap peers; read-only focused CI gate.

- [ ] **Step 1: Create a fresh branch from current `main` and record the exact base SHA.**

Expected branch name: `feat/ukr-surface-lisp-main`.

- [ ] **Step 2: Port the focused RED tests before runtime changes.**

The tests must assert all of the following:

```rust
assert!(registry_has_peer("1045", "uk", "текст-порожній?", "stable"));
assert!(registry_has_peer("1045", "ukr", "порожній-текст?", "stable"));
assert!(!function_table_header.contains("full-uk"));
assert!(function_table_header.contains("uk ukr"));
```

`ukr_acceptance.rs` must include `lib/surface/ukr-acceptance.lisp`, reject any ASCII Latin letter in that source, load the canonical core library, execute the program, and assert result `успіх`.

- [ ] **Step 3: Run focused tests and verify RED for the expected reason.**

Run:

```bash
cargo test -p my-lisp --test ukr_acceptance --test uk_docs_surface_model --test full_uk_surface
```

Expected: failure because current `main` has no authoritative `ukr` surface yet.

- [ ] **Step 4: Port registry/generator semantics from PR #101 to `.lisp` paths.**

Required invariants:

```text
same spelling + same semantic ID across uk/ukr => allowed
same spelling + different semantic IDs         => rejected
candidate spelling                              => documentation only, not admitted runtime peer
stable spelling                                 => admitted runtime peer
```

`generate-function-table.lisp` must emit `uk` then `ukr`; it must not synthesize `ukr` by copying `uk`.

- [ ] **Step 5: Port Lisp-owned stable peer bootstrap.**

After loading a Lisp-owned library value, bind missing stable peer spellings from the semantic registry to the same initial value. Do not rewrite already-existing bindings; later lexical shadowing of one peer must not mutate another peer.

- [ ] **Step 6: Regenerate projections using repository generators, never by hand.**

Run the canonical function-table and meta-registry generators and commit their exact output. Generated files must contain only canonical `.lisp` source references.

- [ ] **Step 7: Port README/API documentation coherently.**

README must state:

```text
uk  = коротка, інтуїтивно зрозуміла українська форма
ukr = повна українська форма тієї самої numeric semantic identity
```

`docs/ukrainian-api.md` must show `uk`, `ukr`, and `ukr status` for documented identities and must explicitly say `candidate` is not a stable executable promise.

- [ ] **Step 8: Run focused GREEN verification.**

Run:

```bash
cargo test -p my-lisp --test ukr_acceptance --test uk_docs_surface_model --test macro_derivation
python3 scripts/generate-ukrainian-api.py --check
```

Expected: PASS.

- [ ] **Step 9: Run full workspace and focused GitHub Actions.**

Run locally/CI:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: PASS. Focused Ukrainian gate must be read-only and generated projections must be zero-diff.

- [ ] **Step 10: Open replacement PR, mark #101 superseded, and merge only on fresh green head.**

Replacement PR body must name #101 and explain that it was ported to canonical `.lisp` paths. Merge with exact-head protection. After merge, close #101 as superseded rather than merging its stale branch.

---

### Task 2: Add exhaustive library-definition discovery without declaring false public classifications

**Files:**
- Create: `scripts/public_api_inventory.py`
- Create: `docs/generated/public-api-discovery.md`
- Test: `crates/my-lisp/tests/public_api_inventory_contract.rs`
- Modify: `docs/ukrainian-surface-inventory.md`
- Modify: `README.md`

**Interfaces:**
- Consumes: canonical `lib/**/*.lisp` tree and current runtime `language_items()`.
- Produces: deterministic discovery records `(source-file, kind, name)` and generated coverage counts; it does not choose translations and does not claim visibility yet.

- [ ] **Step 1: Write failing tests for recursive discovery.**

The test fixture must prove the scanner finds both functions and macros and ignores comments/strings:

```text
(def visible-fn (lambda (x) x))
(defmacro visible-macro args args)
; (def commented-out ...)
"(def text-only ...)"
```

Expected discovered names: exactly `visible-fn`, `visible-macro`.

- [ ] **Step 2: Run the test and verify RED because the scanner does not exist.**

- [ ] **Step 3: Implement `public_api_inventory.py` with a small Lisp-aware top-level scanner.**

Do not use a loose repository-wide grep as authority. The scanner tracks string/comment state and parenthesis depth and records only depth-0 `(def NAME ...)` / `(defmacro NAME ...)` forms. Recursively scan `lib/**/*.lisp`, excluding `lib/generated/**` and `lib/surface/**` governance/generated data.

CLI:

```bash
python3 scripts/public_api_inventory.py --check
python3 scripts/public_api_inventory.py --write-report
```

`--check` validates deterministic parsing; at this stage it reports unresolved visibility but does not fail merely because baseline classification is not yet complete.

- [ ] **Step 4: Generate `docs/generated/public-api-discovery.md`.**

The report must include generated counts, not hard-coded prose:

```text
library files scanned: N
top-level functions: F
top-level macros: M
total definitions: T
```

and a table with `source | kind | name | classification` where unresolved rows show `unreviewed`.

- [ ] **Step 5: Update README and Ukrainian inventory docs.**

Remove any wording that implies `140` is total Ukrainian surface coverage. Explain that `140` was a historical documented subset and link to the generated exhaustive discovery report.

- [ ] **Step 6: Verify zero-diff regeneration and commit.**

Run:

```bash
python3 scripts/public_api_inventory.py --write-report
git diff --exit-code docs/generated/public-api-discovery.md
```

Expected: zero diff on a second generation.

- [ ] **Step 7: Open and merge this infrastructure PR independently.**

Merge only when workspace CI is green. Do not combine domain translations into this PR.

---

### Task 3: Establish explicit visibility baseline and turn discovery into fail-closed governance

**Files:**
- Create: `lib/surface/public-api-inventory.lisp`
- Modify: `scripts/public_api_inventory.py`
- Test: `crates/my-lisp/tests/public_api_inventory_contract.rs`
- Modify: `.github/workflows/surface-drift-check.yml`
- Modify: `docs/generated/public-api-discovery.md`

**Interfaces:**
- Consumes: deterministic discovered definitions from Task 2.
- Produces: explicit `public/internal/compatibility` classification with source provenance; hard CI failure for any definition absent from the manifest.

- [ ] **Step 1: Seed the manifest from discovery output with no translations.**

Schema:

```lisp
(public-api-inventory/1
  (public logic-var lib/unify.lisp)
  (internal unify-walk lib/unify.lisp implementation-helper)
  (compatibility cadddr lib/core.lisp))
```

The manifest must never contain `uk`, `ukr`, `en`, `sa`, or `sym` spellings.

- [ ] **Step 2: Review every current discovered definition by domain and eliminate `unreviewed`.**

Each row must be one of `public`, `internal`, `compatibility`; duplicate `(source-file, name)` entries are invalid.

- [ ] **Step 3: Add RED tests for missing and duplicate classifications.**

Expected failures:

```text
UNCLASSIFIED public API candidate: lib/time.lisp::utc-now
DUPLICATE classification: lib/core.lisp::length
```

- [ ] **Step 4: Make `--check` fail closed.**

Validation rules:

```text
discovered - manifest = empty
manifest - discovered = empty, except explicitly supported runtime-only entries
public/internal overlap = empty
```

- [ ] **Step 5: Add the checker to surface-drift CI and verify GREEN.**

- [ ] **Step 6: Merge baseline classification before assigning new semantic IDs.**

This PR changes visibility governance only; it does not mass-translate names.

---

### Task 4: Translate domains as independent mergeable slices

**Files per domain:**
- Modify: `lib/surface/semantic-registry.lisp`
- Modify: `lib/surface/public-api-inventory.lisp` only if classification evidence changes
- Modify: docs metadata keyed by numeric semantic ID
- Regenerate: `lib/generated/function-table.lisp`
- Regenerate: `lib/generated/meta-semantic-registry.lisp`
- Regenerate: `docs/ukrainian-api.md`
- Modify: `README.md` only when navigation/examples/count wording changes
- Test: domain-specific executable Ukrainian witness

**Interfaces:**
- Consumes: public definitions from Task 3.
- Produces: numeric IDs and `uk`/`ukr` peer surfaces for one domain at a time.

Domain order:

```text
core/collections/text/vector
→ time
→ filesystem/process/host
→ TCP/network/serialization
→ UTF-8/text helpers
→ unify/reason/guard/knowledge
→ SI/quantity/scientific libraries
→ remaining public libraries
```

For each domain repeat this TDD cycle:

- [ ] **Step 1: Add a failing coverage test listing the exact public definitions in the domain that lack numeric semantic IDs or Ukrainian surfaces.**
- [ ] **Step 2: Verify RED.**
- [ ] **Step 3: Allocate append-only semantic IDs and add `ukr candidate` names; add `uk candidate` only when compact wording has review evidence.**
- [ ] **Step 4: Add executable peer binding/witness for stable names; do not mark candidates stable just to make tests green.**
- [ ] **Step 5: Regenerate function table and Ukrainian API reference.**
- [ ] **Step 6: Update README examples/count navigation when the domain materially changes user-facing coverage.**
- [ ] **Step 7: Run focused tests, full workspace CI, no-Latin, collision, docs zero-diff, and surface-drift gates.**
- [ ] **Step 8: Merge the domain PR before starting the next domain.**

---

### Task 5: Final completion gate

**Files:**
- Modify: `scripts/public_api_inventory.py`
- Modify: docs generators/checks
- Modify: README.md
- Modify: `docs/ukrainian-api.md`

**Interfaces:**
- Consumes: all merged domain slices.
- Produces: machine-verifiable claim that the complete public API is classified and represented in Ukrainian documentation.

- [ ] **Step 1: Require every `public` manifest row to resolve to exactly one numeric semantic identity.**
- [ ] **Step 2: Require every public identity to expose explicit `uk` and `ukr` statuses.**
- [ ] **Step 3: Require stable `ukr` to pass no-Latin, collision, executable witness, and docs-coherence checks.**
- [ ] **Step 4: Generate final counters from live data; never hard-code a total such as `140`.**
- [ ] **Step 5: Run full verification and merge only with fresh evidence on the exact head SHA.**

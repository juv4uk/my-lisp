# Full Ukrainian Cyrillic Surface Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `full-uk` an authoritative, executable, Cyrillic-only peer surface that can be typed without switching away from the Ukrainian keyboard layout.

**Architecture:** Keep numeric semantic IDs as the only identity and `semantic-registry.wsm` as the only spelling authority. Add `full-uk` triples to registry rows, have all existing generic registry consumers resolve them automatically, make the function-table generator read them instead of synthesizing them, and enforce the no-Latin rule with executable tests. Preserve current `uk` spellings unchanged.

**Tech Stack:** my-lisp/WASM-style source contracts, Rust integration tests/xtask checks, existing CLI/evaluator, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-09-13-full-uk-cyrillic-surface-design.md`

## Global Constraints

- `semantic ID` remains the only semantic identity.
- `lib/surface/semantic-registry.wsm` remains the only spelling authority.
- Existing stable `uk` spellings must not be removed or renamed.
- Admitted `full-uk` identifiers may contain Ukrainian Cyrillic letters, digits, and admitted Lisp identifier punctuation, but no ASCII Latin letters.
- No Latin technical allowlist exists for `full-uk`.
- Strings, comments, data, file paths, and foreign payloads are outside the Cyrillic-only identifier rule.
- Generated tables are projections and must not invent `full-uk` names.
- Work follows RED → GREEN → refactor and commits each independently reviewable slice.

---

### Task 1: Make the no-layout-switch rule executable

**Files:**
- Modify: `crates/my-lisp-cli/tests/uk_surface_audit_projection.rs`
- Modify after RED: `lib/surface/український-профіль-джерела.всм`

**Interfaces:**
- Consumes: `full_uk_candidate_rows(source) -> Vec<FullUkCandidateRow>` already present in the audit test.
- Produces: a regression test that rejects every staging `full-uk` candidate containing ASCII Latin letters.

- [ ] **Step 1: Write the failing test**

Add a test that collects every non-`—` `full_uk` candidate whose spelling contains `char::is_ascii_alphabetic()` and asserts the offender list is empty. Include semantic IDs and spellings in the failure message so one CI run reports the full cleanup set.

- [ ] **Step 2: Run the focused test and verify RED**

Run:

```bash
cargo test -p my-lisp-cli --test uk_surface_audit_projection full_uk_candidates_need_no_latin_keyboard_layout -- --nocapture
```

Expected: FAIL listing current Latin-bearing candidates such as `розібрати-json`, `sha256-у-шістнадцятковий-текст`, and TCP-bearing host candidates.

- [ ] **Step 3: Replace Latin-bearing staging candidates with reviewed Cyrillic spellings**

Update only the offending full-UK candidate field(s). Remove the `дозволені-міжнародні-позначення` policy that permits Latin identifier fragments for `full-uk`; replace it with the explicit no-layout-switch rule. Do not change current stable `uk` spellings.

- [ ] **Step 4: Run focused audit tests**

Run:

```bash
cargo test -p my-lisp-cli --test uk_surface_audit_projection -- --nocapture
```

Expected: all UK staging coverage, alias coherence, and Cyrillic-only tests PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/my-lisp-cli/tests/uk_surface_audit_projection.rs lib/surface/український-профіль-джерела.всм
git commit -m "test(uk): require full surface without Latin layout"
```

### Task 2: Admit `full-uk` as a real registry namespace

**Files:**
- Modify: `crates/my-lisp/src/semantic_registry.rs`
- Modify: `lib/surface/semantic-registry.wsm`

**Interfaces:**
- Consumes: generic `parse_rows`, `build_surface_index`, `admitted_semantic_id_for_surface`.
- Produces: admitted `(full-uk NAME stable)` triples that resolve to the same numeric semantic IDs as peer spellings.

- [ ] **Step 1: Write the failing registry test**

Add a synthetic parser test proving namespace `full-uk` is preserved and a live-registry test proving one ratified full-UK spelling resolves to the same semantic ID as its current `uk` peer.

- [ ] **Step 2: Run focused semantic-registry tests and verify RED**

Run:

```bash
cargo test -p my-lisp semantic_registry::tests -- --nocapture
```

Expected: the live full-UK assertion FAILS because the registry has no authoritative `full-uk` triples yet.

- [ ] **Step 3: Add authoritative `full-uk` triples without changing `uk`**

For the first ratified slice, add `(full-uk ... stable)` to semantically clear identities. Existing `uk stable` names that are already full-word Ukrainian may use the same spelling. For abbreviated/current names, use the reviewed staging full-word spelling only when it passes the Cyrillic-only rule.

The six stable host API identities 1147–1152 must use no-layout-switch spellings; TCP-oriented names must be behavioral Ukrainian names rather than Latin `tcp` fragments.

- [ ] **Step 4: Run semantic-registry tests**

Run the same command; expected PASS with peer identity preserved.

- [ ] **Step 5: Commit**

```bash
git add crates/my-lisp/src/semantic_registry.rs lib/surface/semantic-registry.wsm
git commit -m "feat(uk): admit full Ukrainian peer surface"
```

### Task 3: Stop generated tables from inventing `full-uk`

**Files:**
- Modify: `scripts/generate-function-table.my`
- Modify: `lib/generated/function-table.wsm`
- Modify: `docs/generated/function-table.md`
- Test: `crates/my-lisp-cli/tests/uk_surface_audit_projection.rs`

**Interfaces:**
- Consumes: registry `(full-uk WORD STATUS)` triples.
- Produces: generated `full-uk` columns exactly matching registry authority.

- [ ] **Step 1: Write the failing projection test**

Add a test that selects a registry row where `uk != full-uk` and asserts the generated function table contains the authoritative `full-uk` spelling/status rather than a mirror of `uk`.

- [ ] **Step 2: Run focused test and verify RED**

Run the UK audit integration test; expected FAIL because `full-uk-projection` currently mirrors `uk`.

- [ ] **Step 3: Change generator to read `full-uk` directly**

In `generate-function-table.my`, call `get-surface` for `full-uk`; remove the mirroring `full-uk-projection` policy. Preserve `— missing` when no full-UK row exists. Never infer a name.

- [ ] **Step 4: Regenerate projections**

Run:

```bash
cargo run -p my-lisp-cli --bin my-lisp -- scripts/generate-function-table.my
```

Then run the focused UK projection tests; expected PASS.

- [ ] **Step 5: Commit**

```bash
git add scripts/generate-function-table.my lib/generated/function-table.wsm docs/generated/function-table.md crates/my-lisp-cli/tests/uk_surface_audit_projection.rs
git commit -m "feat(canon): project authoritative full Ukrainian names"
```

### Task 4: Prove the meta/native runtime sees the same full-UK identity

**Files:**
- Modify: `crates/my-lisp/tests/uk_sa_surface.rs`
- Regenerate if needed: `lib/generated/meta-semantic-registry.my`

**Interfaces:**
- Consumes: admitted registry surfaces and generated meta semantic registry.
- Produces: native and meta-evaluator identity parity for a `full-uk` spelling.

- [ ] **Step 1: Add failing peer-identity test**

Choose a ratified full-UK spelling that differs from current `uk`, resolve/evaluate both spellings, and assert both materialize the same semantic callable identity.

- [ ] **Step 2: Verify RED**

Run:

```bash
cargo test -p my-lisp --test uk_sa_surface full_uk -- --nocapture
```

Expected: fail until generated meta registry includes the new admitted spelling.

- [ ] **Step 3: Regenerate meta semantic registry**

Run the existing generator/check path so admitted `full-uk` triples are projected generically. Do not add a separate full-UK dictionary.

- [ ] **Step 4: Verify GREEN**

Run native/meta peer tests; expected PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/my-lisp/tests/uk_sa_surface.rs lib/generated/meta-semantic-registry.my
git commit -m "test(uk): prove full surface semantic identity parity"
```

### Task 5: Add a no-layout-switch executable acceptance program

**Files:**
- Create: `lib/surface/full-uk-acceptance.lisp`
- Modify: `crates/my-lisp/tests/uk_sa_surface.rs`
- Modify: `crates/xtask/src/checks.rs`

**Interfaces:**
- Consumes: admitted `full-uk` registry names.
- Produces: a real CLI/evaluator witness whose executable identifiers contain zero ASCII Latin letters.

- [ ] **Step 1: Write the failing runner/policy tests**

Add a runner test that evaluates `full-uk-acceptance.lisp` and expects symbol `успіх`. Add an xtask policy check that strips comments/string contents and rejects ASCII Latin letters in executable code.

- [ ] **Step 2: Verify RED**

Run the focused acceptance and xtask checks; expected FAIL before the program/required names are complete.

- [ ] **Step 3: Write the acceptance program**

Exercise definition, function, condition, arithmetic, list processing, a higher-order operation, strings, and one non-core/host capability through ratified `full-uk` names. Keep all executable alphabetic identifiers Ukrainian Cyrillic.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
cargo test -p my-lisp --test uk_sa_surface -- --nocapture
cargo xtask verify
```

Expected: acceptance result `успіх`; no-layout-switch checks PASS.

- [ ] **Step 5: Commit**

```bash
git add lib/surface/full-uk-acceptance.lisp crates/my-lisp/tests/uk_sa_surface.rs crates/xtask/src/checks.rs
git commit -m "test(uk): execute full Cyrillic-only acceptance program"
```

### Task 6: Full verification and issue evidence

**Files:**
- Update only if evidence requires: `docs/ukrainian-surface-plan.uk.md`
- Update GitHub issues: #75 and #77 with concrete evidence; close only if every acceptance criterion is actually satisfied.

- [ ] **Step 1: Run repository verification**

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo build --workspace
cargo xtask verify
```

Also run the function-table and meta-registry generation checks used by CI.

- [ ] **Step 2: Confirm no accidental semantic/compatibility drift**

Review the diff: no semantic IDs changed; no existing stable `uk` spelling disappeared; generated files match authority; `full-uk` admitted identifiers contain no ASCII Latin letters.

- [ ] **Step 3: Open/update PR with exact evidence**

Record commit SHAs and exact commands/checks. Do not claim #75/#77 complete unless the acceptance program, authority projection, drift guards, and full CI are green.

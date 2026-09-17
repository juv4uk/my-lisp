# Canon Sculpture Audit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Before the 1.0 freeze, test which current Canon concepts are independently irreducible without changing language semantics during the audit.

**Architecture:** `knowledge/canon-sculpt-audit.lisp` is a non-authoritative evidence map. Research probes live under `evidence/canon-sculpt/` so they cannot accidentally enter the active semantic-fixture routing work. Any real semantic change is a separate follow-up only after preservation, circularity, and cross-backend gates are satisfied.

**Tech Stack:** my-lisp S-expressions, existing Lisp-owned fixture corpora, GitHub Actions, Rust evaluator only as observed mechanism.

**Spec:** GitHub issue #419 and `knowledge/canon-sculpt-audit.lisp`.

## Global Constraints

- Base audit integration on current `main`, not stale sibling branches.
- Do not edit `language-contract.lisp`, ADR-004, Canon registry, evaluator production code, or semantic expected values in this audit.
- Do not create a second semantic oracle: reference existing Lisp-owned witnesses instead of copying expected truth.
- Do not edit files owned by active #421, #422/#418/#410/#406/#396, #427, #328, #402/#403, #409/#424/#426/#428/#429, or #317.
- A machine/backend lowering is implementation evidence, never proof of semantic derivability.
- `insufficient-evidence` wins over a guessed classification.

---

### Task 1: Keep the integrated demolition map current

**Files:**
- Modify: `knowledge/canon-sculpt-audit.lisp`

**Interfaces:**
- Consumes: current `main` Canon implementation, semantic registry, existing Lisp-owned fixtures.
- Produces: one machine-readable evidence map; no semantic behavior.

- [ ] **Step 1: Compare audit branch to current main**

Run:
```bash
git diff --name-only main...audit/419-canon-sculpt-integrated
```
Expected: only `knowledge/canon-sculpt-audit.lisp`, `evidence/canon-sculpt/*`, and this plan until a separately approved audit-only file is added.

- [ ] **Step 2: Verify no production authority file changed**

Run:
```bash
git diff --name-only main...audit/419-canon-sculpt-integrated | grep -E '^(language-contract\.lisp|lib/|crates/|contracts/|tests/fixtures/)' && exit 1 || exit 0
```
Expected: exit 0.

- [ ] **Step 3: Commit only evidence-map reconciliation**

```bash
git add knowledge/canon-sculpt-audit.lisp
git commit -m "audit(#419): reconcile Canon evidence map"
```

### Task 2: Falsify ordinary-function `quote`

**Files:**
- Existing probe: `evidence/canon-sculpt/quote-ordinary-function-red.lisp`

**Interfaces:**
- Consumes: eager ordinary function application and current source symbol resolution.
- Produces: evidence only that ordinary eager application cannot explain quote's non-evaluation behavior.

- [ ] **Step 1: Run the research probe with the normal CLI/evaluator**

```bash
cargo run -q -p my-lisp-cli -- evidence/canon-sculpt/quote-ordinary-function-red.lisp
```
Expected: failure before the function body because `canon-sculpt-unbound-symbol` is evaluated as an unbound symbol.

- [ ] **Step 2: Record only the observed failure class**

If the failure is for a different reason, do not reinterpret it as support; fix or discard the probe.

- [ ] **Step 3: Do not GREEN this by adding host AST access or a quote alias**

A valid follow-up must explicitly name a strictly lower evaluation-control mechanism.

### Task 3: Isolate the selective-evaluation lower bound for `cond`

**Files:**
- Existing probe: `evidence/canon-sculpt/selective-evaluation-ordinary-function-red.lisp`
- Read-only authority: `tests/fixtures/control-dispatch-v1.lisp`

**Interfaces:**
- Consumes: eager ordinary application and canonical explicit-result `cond` witnesses.
- Produces: evidence only that an ordinary eager function cannot skip an unselected runtime branch.

- [ ] **Step 1: Run the ordinary-function selective-evaluation probe**

```bash
cargo run -q -p my-lisp-cli -- evidence/canon-sculpt/selective-evaluation-ordinary-function-red.lisp
```
Expected: failure while evaluating `canon-sculpt-unbound-branch` before `choose-first-eager` can return `selected`.

- [ ] **Step 2: Run the canonical control witness**

```bash
cargo test -p my-lisp control_dispatch -- --nocapture
```
Expected: canonical three-part rows in `tests/fixtures/control-dispatch-v1.lisp` pass, including explicit matching of `()`, non-truthiness of arbitrary data, and `()` on no match.

- [ ] **Step 3: Classify the result conservatively**

If Task 3.1 RED and Task 3.2 GREEN, record only: ordinary eager function application is insufficient for canonical selective evaluation. Do not promote `PRIM_COND` to proven irreducible.

### Task 4: Search for a strictly smaller runtime selector

**Files:**
- Modify only: `knowledge/canon-sculpt-audit.lisp`
- New probe allowed only under: `evidence/canon-sculpt/`

**Interfaces:**
- Consumes: semantic registry, macro substrate, necessary forms, callable value set.
- Produces: either a falsifiable lower-mechanism candidate or an explicit `insufficient-evidence` result.

- [ ] **Step 1: Inventory all non-`cond` mechanisms capable of delaying source evaluation**

Search:
```bash
rg -n 'SpecialForm|NecessaryForm|defmacro|macro|eval' crates/my-lisp/src lib knowledge tests/fixtures
```

- [ ] **Step 2: Reject fake reductions**

Reject any candidate that uses `cond` through another surface, registry round-trip, host representation inspection, backend branch opcode, or a stronger hidden selector.

- [ ] **Step 3: If no candidate survives, keep `syntax-rule-candidate`**

Absence of a current lower selector is evidence against easy reduction, not a proof of global irreducibility.

### Task 5: Cross-backend evidence check

**Files:**
- Read-only: `docs/cross-substrate-evidence-matrix.md`
- Read-only: current CML/fpga evidence referenced there.
- Modify only: `knowledge/canon-sculpt-audit.lisp`

**Interfaces:**
- Consumes: existing evidence classes.
- Produces: provenance labels, not copied expected values.

- [ ] **Step 1: Distinguish harness coverage from fresh execution**

Record `HARNESS-COVERED`, `MILESTONE-EQUIVALENT`, or other existing evidence classes literally; never rewrite them as “verified now”.

- [ ] **Step 2: Require fresh replay before using stale sibling PR #173 as current proof**

Do not treat old CML bridge work as merge-ready evidence while it is based on an old main.

### Task 6: Open audit PR only after fresh verification

**Files:** no new production files.

- [ ] **Step 1: Run diff hygiene**

```bash
git diff --check main...HEAD
```
Expected: exit 0.

- [ ] **Step 2: Run the two research probes and the canonical control witness**

Use Tasks 2 and 3 commands. Preserve RED as RED evidence; do not make the research probes part of steady-state semantic CI.

- [ ] **Step 3: Open a draft PR**

The PR must state exactly what was observed, which probes remain unexecuted, and that no primitive deletion/reclassification is authorized.

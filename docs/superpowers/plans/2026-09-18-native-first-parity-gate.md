# Native-First Differential Parity Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every admitted native-first island prove exact value parity against both the reference evaluator and independent Lisp-owned expected evidence, while also proving the route really executed natively.

**Architecture:** Keep the gate Lisp-owned. A parity corpus stores expression data, independent expected values, and explicit effect/error classes. The runner evaluates each row through the reference evaluator and through `native-first-execute-expression`; native success is accepted only when both agree with the independent expected value and provenance is exactly `execution-route native (status completed)`.

**Tech Stack:** my-lisp, existing native-first dispatcher/execution bridge, current x86-64 Lisp machine layer, semantic-slice CI.

**Spec:** `docs/superpowers/specs/2026-09-18-native-first-execution-design.md`

## Global Constraints

- No Rust semantic matcher.
- No change to Canon or semantic registry.
- Native regression after a native plan is chosen must stay visible; never retry through evaluator.
- Independent expected evidence must remain explicit; native and evaluator cannot be each other's sole oracle.
- Current native island is bounded `(car (cons <u64-literal> <u64-literal>))`.
- Current island is pure and has no admitted error outcome; encode these as explicit metadata rather than silently omitting them.
- The bridge can return exact integers only through `9007199254740991`; the first corpus includes that exact boundary.
- #507 owns compiled encoder work and #508 owns coverage ledger work; this task edits neither.

---

### Task 1: RED parity witness

**Files:**
- Create: `tests/fixtures/native-first-parity-witness.lisp`
- Modify: `scripts/test-current-semantic-slice.sh`

**Interfaces:**
- Consumes: `native-first-execute-expression`, evaluator `eval`, existing machine loads.
- Produces: named envelope `(native-first-parity-witness (status pass) (cases 4))`.

- [ ] **Step 1:** Define four independent corpus rows:
  - `(car (cons 0 1)) -> 0`
  - `(car (cons 2 3)) -> 2`
  - `(car (cons 42 99)) -> 42`
  - `(car (cons 9007199254740991 7)) -> 9007199254740991`
- [ ] **Step 2:** Each row records `effect pure` and `error not-applicable`.
- [ ] **Step 3:** Load a not-yet-existing `lib/machine/dispatch/native-first-parity.lisp`.
- [ ] **Step 4:** Add the witness to `test-current-semantic-slice.sh`.
- [ ] **Step 5:** Run the focused witness and observe RED specifically because the parity runner file is absent.

### Task 2: GREEN Lisp-owned parity runner

**Files:**
- Create: `lib/machine/dispatch/native-first-parity.lisp`

**Interfaces:**
- Produces:
  - `native-first-parity-case`
  - `native-first-parity-run`
  - `native-first-parity-all-pass?`

- [ ] **Step 1:** Evaluate each expression independently with `eval`.
- [ ] **Step 2:** Route the same expression through `native-first-execute-expression`.
- [ ] **Step 3:** Compare evaluator value to corpus expected value.
- [ ] **Step 4:** Compare native outcome to exact expected native provenance/value record.
- [ ] **Step 5:** Emit named failure kind for evaluator mismatch or native/route mismatch.
- [ ] **Step 6:** Run the focused witness to GREEN.

### Task 3: Integration and ownership

- [ ] Run diff hygiene.
- [ ] Run exact-head CI and bilingual gate.
- [ ] Open a fresh PR against `main`; do not merge old stacked #514.
- [ ] Close/supersede #514 after the replay PR is verified.
- [ ] Report to #509/#504 that future native islands must extend the corpus with value/error/effect/provenance evidence.

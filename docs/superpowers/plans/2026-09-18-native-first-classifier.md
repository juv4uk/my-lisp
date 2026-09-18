# Native-First Classifier Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a total Lisp-owned native-plan classifier with explicit evaluator fallback and one proven bounded native island.

**Architecture:** The classifier receives expression data, never source text and never host AST objects. It returns structured machine forms or the unchanged expression as fallback. It reuses existing Lisp-owned lowering/operand/layout layers and does not execute bytes.

**Tech Stack:** my-lisp, Rust test harness as observer only, existing x86-64 Lisp machine layer.

**Spec:** `docs/superpowers/specs/2026-09-18-native-first-execution-design.md`

## Global Constraints

- No semantic matcher in Rust.
- No changes to semantic registry/Canon.
- No raw native execution in #505.
- Do not edit #483/#484 machine-form files.
- Unsupported is fallback, not error.
- RED must precede production implementation.

### Task 1: RED contract for total classifier

**Files:**
- Create: `crates/my-lisp/tests/native_first_dispatch.rs`

- [ ] Write test harness that conditionally loads `lib/machine/dispatch/native-first.lisp` if present.
- [ ] Load core + existing encoder/layout/operands/lowering dependencies.
- [ ] Assert `(car (cons 2 3))` becomes a native plan with existing lowering forms and 16-byte arena.
- [ ] Assert ordinary addition, dynamic CONS operands, negative literal fields, atoms, and malformed data return exact evaluator fallback.
- [ ] Run focused test and observe RED because `native-first-plan` is absent.

### Task 2: Minimal Lisp-owned classifier

**Files:**
- Create: `lib/machine/dispatch/native-first.lisp`

- [ ] Add result constructors.
- [ ] Add exact-shape CAR(CONS u64 u64) recognition.
- [ ] Reuse `x86-as-u64-imm` and `x86-lower-cons-car-u64-forms`.
- [ ] Preserve original expression in every fallback.
- [ ] Do not encode bytes or invoke host capability.
- [ ] Run focused test to GREEN.

### Task 3: Integration gates

- [ ] Run diff hygiene.
- [ ] Run exact-head CI and bilingual docs gate.
- [ ] Open draft PR tied to #505.
- [ ] Post coordination note to #504/#506/#509 and #178 boundary.

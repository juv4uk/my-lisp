# Native-First Execution Bridge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Execute #505 native plans on the CPU and evaluator fallbacks through the reference evaluator, with route provenance preserved in Lisp.

**Architecture:** The bridge is a Lisp function over expression data. It calls `native-first-plan`; a `native-plan` goes through existing closed admission/native-call, while `evaluator-fallback` calls `eval` on the unchanged expression. Native execution errors propagate and are never silently converted to fallback.

**Tech Stack:** my-lisp, existing Lisp x86 lowering/encoding/admission, semantics-blind host native-call capability.

**Spec:** `docs/superpowers/specs/2026-09-18-native-first-execution-design.md`

## Constraints

- No Rust semantic matcher or new Rust code.
- No semantic registry/Canon changes.
- Native-plan execution must use existing closed admission.
- Fallback occurs only when classifier returns evaluator-fallback.
- Once classified native, machine/admission/runtime errors propagate.
- Result includes route provenance for tests: `(execution-route native VALUE)` or `(execution-route evaluator VALUE)`.

### Task 1 — RED

Create `tests/fixtures/native-first-execution-witness.lisp`.

Expected cases:
- `(car (cons 2 3))` -> `(execution-route native 2)`;
- `(+ 2 3)` -> `(execution-route evaluator 5)`;
- dynamic `(car (cons (+ 1 1) 3))` -> `(execution-route evaluator 2)`;
- `radio` quoted as expression data -> evaluator route returning `radio`.

Run through a verification-only workflow before the bridge file exists and record RED.

### Task 2 — GREEN

Create `lib/machine/dispatch/native-first-exec.lisp`:
- call `native-first-plan`;
- dispatch only on explicit plan tag;
- native: `x86-call-admitted-u64 forms arena`;
- fallback: `eval original-expression`;
- return route provenance;
- no catch-and-fallback around native errors.

Run the same witness to GREEN.

### Task 3 — Integration

Run exact-head CI+bilingual on the stacked PR, then feed evidence to #504/#506/#509.

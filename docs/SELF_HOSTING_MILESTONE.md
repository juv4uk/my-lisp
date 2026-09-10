# Self-hosting evaluator milestone

## Goal

Move the evaluator boundary from native implementation toward a Lisp-owned semantic layer.

## Current architecture

```
source
  ↓
canonical representation
  ↓
lib/meta-eval.my
  ↓
native primitive substrate
```

## Work items

### 1. Evaluator ownership

- Keep evaluation rules represented in My-Lisp data.
- Keep Rust limited to mechanism and primitive execution.

### 2. First-class semantic objects

Required witnesses:

- closure
- primitive identity
- macro identity
- environment lookup
- application

### 3. Proof boundary

Every new evaluator capability requires:

```
contract
  ↓
executable witness
  ↓
regression test
```

## Non-goal

Do not expand language features before the evaluator boundary is stable.

## Principle

Rust provides mechanism. My-Lisp defines meaning.

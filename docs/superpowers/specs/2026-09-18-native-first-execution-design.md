# Native-First Execution Design

**Status:** approved by owner request on 2026-09-18
**Parent:** #504
**First implementation slice:** #505

## Purpose

Make execution policy explicit for every Lisp expression:

```text
expression
   |
   v
Lisp-owned native-plan classifier
   |----------------------|
 native-plan       evaluator-fallback
   |                      |
 admitted forms          reference evaluator
   |
 bytes
   |
 CPU
```

The policy is total even while native coverage is partial. Lack of native lowering is a normal execution-routing outcome, not a language error.

## Authority

- Language meaning: semantic registry / Canon.
- Native-plan classification: Lisp-owned machine projection.
- ISA facts and encoding: existing `lib/machine/**`.
- Optimization: CML.
- Executable-memory mechanism: host.
- Evaluator: reference/fallback execution path.

No Rust semantic matcher is introduced.

## First slice (#505)

Create `lib/machine/dispatch/native-first.lisp` with:

```lisp
(native-first-plan expression-data)
```

Result algebra:

```text
(native-plan <structured-machine-forms> <arena-bytes>)
(evaluator-fallback <original-expression>)
```

The first admitted native island is the already-proven bounded literal form:

```lisp
(car (cons <u64-literal> <u64-literal>))
```

It reuses `x86-lower-cons-car-u64-forms` and `x86-pair-cell-bytes`. The classifier never produces raw bytes and never calls `native-call-u64-raw`.

Every other expression falls back unchanged, including dynamic operands such as:

```lisp
(car (cons (+ 1 1) 3))
```

## Fail-Closed Rules

- Exact shape match only.
- Both CONS fields must be admitted exact u64 values.
- Malformed/dotted source data falls back.
- Unsupported semantics falls back.
- Once a future execution bridge has accepted a `native-plan`, machine execution errors must not be silently converted to evaluator fallback.

## Growth Rule

Native coverage expands by adding independently witnessed classifier routes. Source programs do not opt into a backend and do not change when a new route is added.

## Verification

#505 proves classification only. #506 owns actual route execution. #509 owns differential parity for every admitted native island.

#483/#484 may expand machine atoms concurrently; #505 does not edit their files.

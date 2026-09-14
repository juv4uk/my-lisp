# VERTICAL-LISP-2 — Pair Layout Contract Slice

Status: active implementation note for issue #126.

## Purpose

Before CAR/CDR/CONS lowering exists, the pair representation must have one explicit machine-readable authority. The target lowerer must not invent field offsets locally.

## First TDD slice

1. RED: require the shared memory layout contract to state the pair storage and offsets.
2. GREEN: add the smallest pair-layout record to `memory-layout-contract.lisp`.
3. Only after this is green, add x86-64 memory addressing support and semantic lowering for `car`/`cdr`.
4. `cons` allocation remains a separate mechanism slice so Rust never learns Lisp semantics.

## Intended representation

A pair is a heap cell addressed by the 48-bit payload carried by the existing `cons` NaN-box tag. The cell is two consecutive 64-bit Lisp values:

```text
base + 0   car : 64-bit Lisp value
base + 8   cdr : 64-bit Lisp value
cell size  16 bytes
```

This document is explanatory only. `memory-layout-contract.lisp` is the machine-readable authority.

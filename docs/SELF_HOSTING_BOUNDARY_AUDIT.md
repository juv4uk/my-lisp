# Self-hosting boundary audit

## Purpose

Record the current boundary between the native evaluator and the metacircular evaluator.

## Current state

`lib/meta-eval.my` is a self-hosting witness. It owns:

- Canon-first name resolution
- lexical environment model
- closure representation
- macro expansion model
- recursive and mutually recursive closure data
- lambda-list validation

The native runtime still provides the execution substrate and the always-loaded evaluator path.

## Next migration boundary

The next reductions should preserve behavior while moving semantic ownership toward the language layer:

1. establish parity tests between native evaluation and `my-eval`;
2. reduce duplicated environment semantics;
3. keep native code as a minimal mechanism layer.

## Rule

Do not remove native code until the language-owned behavior has an executable witness and regression coverage.

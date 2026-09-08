# ADR-005 — Trilingual Surface Parity

**Status:** Accepted  
**Date:** 2026-09-08  
**Registry note:** the legacy `canonical/EN` registry interpretation below is a historical snapshot; ADR-008 supersedes it with numeric-only `semantic-registry.wsm` authority.

## Decision

`my-lisp` has one semantic core and three peer human programming surfaces:

- English (`en`)
- Ukrainian (`uk` / `ук`)
- Sanskrit (`sa`)

The core is **not English**. Canonical/core identifiers are semantic and historical identities. An English surface name may have the same spelling as its canonical identity, but that spelling coincidence gives English no higher semantic authority than Ukrainian or Sanskrit.

```text
                         semantic core
                              │
                    canonical identity
                              │
              ┌───────────────┼───────────────┐
              │               │               │
             EN              UK              SA
          human name      human name      human name
```

A public semantic identity is trilingually complete only when all three surfaces have a ratified stable name and equivalent executable behavior.

## Why

The earlier implementation grew from an English-facing runtime and then added Ukrainian and Sanskrit aliases. That history is an implementation fact, not the desired language model. Treating English as the implicit source surface would make the other two languages permanently derivative and would blur the distinction between semantic identity and human-facing spelling.

The project goal is stronger: a program should be expressible through any of the three public surfaces without changing its semantics.

## Surface registry interpretation

Until the legacy machine table is renamed, `lib/surface/uk-sa-coverage.wsm` is interpreted as the trilingual surface registry:

```text
category | canonical/EN | UK | SA | UK status | SA status | notes
```

For the English surface, the canonical/EN column is an explicit English surface name as well as the canonical identity. English status is derived as:

- `stable` for every translation-eligible public row;
- `compatibility-only` for rows outside the public translation denominator.

This derivation is intentionally executable and checked. A later schema migration may split `canonical` and `en` into separate fields even where their spellings are identical; that migration must not change semantic identity or surface behavior.

## Two different completion claims

### 1. Structural parity

Structural parity means every known public identity is classified for EN, UK and SA as one of:

- `stable`
- `candidate`
- `missing`
- `compatibility-only`

Structural parity is a property of the machine registry. It prevents silent gaps.

### 2. Release parity

Release parity is stronger. A human surface is release-complete only when all translation-eligible public identities are `stable` for that surface and the surface also has executable acceptance evidence.

`my-lisp` must not claim full trilingual release parity while any public SA row is `candidate` or `missing`.

At acceptance of this ADR, the machine table records:

| Surface | Stable | Candidate | Missing | Compatibility-only |
|---|---:|---:|---:|---:|
| EN | 140 | 0 | 0 | 21 |
| UK | 140 | 0 | 0 | 21 |
| SA | 36 | 88 | 16 | 21 |

Therefore the correct current statement is:

> EN and UK have complete stable name coverage for the selected 140-name public surface; SA is implemented broadly but is not yet fully ratified or complete.

## REPL contract

Interactive introspection is split deliberately:

- `(env)` / `(середовище)` — raw lexical truth: every binding visible to the evaluator;
- `:імена` / `:names` — human catalog for the current surface;
- `:ім'я NAME` / `:name NAME` — one semantic identity with its EN/UK/SA names and statuses;
- `:поверхні` / `:surfaces` — trilingual completion matrix.

The REPL commands are presentation policy, not Lisp semantics and not additions to `language-contract.my`.

## Completion gate

`scripts/check_trilingual_surface.py` is the executable registry gate. Its default mode validates structural parity and prints the current matrix without failing merely because SA is still open. Its `--require-complete` mode exits non-zero until EN, UK and SA are all fully stable across the public denominator.

This preserves the project's epistemic rule: **the label must not be stronger than the strongest executable evidence**.

## Consequences

1. New public operations must be classified for all three human surfaces in the same registry change.
2. A new English-facing operation is not considered surface-complete merely because its core identifier already exists.
3. Sanskrit `candidate` names stay candidates until separately audited and ratified; no bulk status promotion is allowed just to reach 100%.
4. Translation, REPL help, documentation and acceptance tests should consume the same registry rather than maintain private dictionaries.
5. `core` remains available as a canonical/debug surface and is not counted as a fourth human language.

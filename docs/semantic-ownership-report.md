# Semantic ownership report

> Generated deterministically from `knowledge/semantic-ownership.wsm`.
> This report counts **audited behaviors/responsibilities**, not LOC and not total language completeness.
> No number below is a “self-hosting percentage”.

## Summary

- Audited ownership rows: **34**
- Confirmed ownership rows: **32**
- Partial ownership rows: **2**
- Confirmed migration ledger entries: **8**
- Remaining host semantic-policy candidates: **1**
- Confirmed irreducible host mechanism/observation/authorization rows: **6**
- Unknown ownership rows: **0**

## Ownership classes

| category | audited rows |
|---|---:|
| `canon-ground` | 1 |
| `canon-operation` | 7 |
| `derived-tooling` | 1 |
| `host-authorization` | 3 |
| `host-mechanism` | 3 |
| `host-observation` | 1 |
| `lisp-owned` | 16 |
| `necessary-form` | 2 |

## Layers

| category | audited rows |
|---|---:|
| `bootstrap` | 3 |
| `canon` | 8 |
| `host-capability` | 6 |
| `knowledge` | 3 |
| `reasoning` | 5 |
| `self-hosting` | 1 |
| `stdlib` | 6 |
| `tooling` | 2 |

## Epistemic status

| category | audited rows |
|---|---:|
| `confirmed` | 32 |
| `partial` | 2 |

## Host-policy candidates

- `tcp-resource-representation` — Concrete TcpStream/TcpListener representation still appears in core Value; focused audit is issue 27 (`partial`)

## Confirmed migration ledger

- `canon-surface-authority` — `host-hardcoded` → `registry-data` at `668794caf6f3e2e1d0d6c8e740f218cc6ef04db9`: Canon stable surface routing moved to shared numeric semantic registry projection
- `core-host-capability-split` — `core-os-code` → `host-adapter` at `f565f6692c36a97f80afe0233f0bdb8dca506b81`: OS-touching filesystem/process/TCP operations moved from my-lisp core to my-lisp-host
- `defmacro-fallback-to-lisp` — `host-mechanism` → `lisp-owned` at `3fff9e9fbb7171a81ba128baedd68f093fc0c65b`: Rust defmacro evaluator fallback removed; language macro path owns behavior
- `list-rust-to-lisp` — `host-mechanism` → `lisp-owned` at `efdd9252fd4ca4af4503b219ab3ae79130ef0e64`: list special form removed from Rust and defined in lib/core.my
- `macro-peer-surface-authority` — `host-hardcoded` → `registry-data` at `baa03b7acf0793bec3184a48099c484231922bea`: 0012 stable and compatibility peer names moved from loader literals to registry admission
- `necessary-form-surface-authority` — `host-hardcoded` → `registry-data` at `3fa2ae1f5e5786cd5c0b41489648a23bb1f405f5`: LAMBDA/DEFINE stable surface routing moved from Rust spelling tables to numeric registry projection
- `peer-builtin-surface-authority` — `host-hardcoded` → `registry-data` at `dd4d9ae7d7bccbe465a0ae8ceb1b2f17f5f80f4e`: Arithmetic/comparison peer names moved from Rust arrays to registry-derived bindings
- `tooling-human-key-to-semantic-id` — `human-spelling-tooling` → `semantic-id-tooling` at `e8f60f659199686205376ca4fd1c570034c05de6`: Tooling syntax discovery moved from human spelling keys to semantic identities

## Audited behaviors

| key | semantic id | owner | layer | status | behavior |
|---|---|---|---|---|---|
| `backward-reasoning` | `—` | `lisp-owned` | `reasoning` | `confirmed` | Backward-chaining proof search and provenance construction |
| `canon-atom` | `0002` | `canon-operation` | `canon` | `confirmed` | ATOM first-class canonical operation |
| `canon-car` | `0005` | `canon-operation` | `canon` | `confirmed` | CAR first-class canonical projection operation |
| `canon-cdr` | `0006` | `canon-operation` | `canon` | `confirmed` | CDR first-class canonical remainder operation |
| `canon-cond` | `0007` | `canon-operation` | `canon` | `confirmed` | COND canonical short-circuit syntax |
| `canon-cons` | `0004` | `canon-operation` | `canon` | `confirmed` | CONS first-class canonical construction operation |
| `canon-empty-list` | `—` | `canon-ground` | `canon` | `confirmed` | Canon 0 empty-list ground object |
| `canon-eq` | `0003` | `canon-operation` | `canon` | `confirmed` | EQ first-class canonical identity operation |
| `canon-quote` | `0001` | `canon-operation` | `canon` | `confirmed` | QUOTE evaluator meaning and reserved surface resolution |
| `filesystem-authorization` | `—` | `host-authorization` | `host-capability` | `confirmed` | Per-session filesystem read/write scope and host canonicalization enforcement |
| `gensym` | `—` | `lisp-owned` | `stdlib` | `confirmed` | Fresh-symbol policy composed in Lisp from string operations and monotonic observation |
| `immutable-worlds` | `—` | `lisp-owned` | `knowledge` | `confirmed` | Immutable world snapshots and explicit world transitions |
| `knowledge-journal` | `—` | `lisp-owned` | `knowledge` | `confirmed` | Append-only knowledge journal and guarded knowledge admission |
| `list-constructor` | `—` | `lisp-owned` | `stdlib` | `confirmed` | Variadic list constructor derived from language lambda/rest semantics |
| `macro-definition` | `0012` | `lisp-owned` | `bootstrap` | `confirmed` | Macro-definition behavior derived in lib/macro.my over narrow make-macro substrate |
| `meta-evaluator` | `—` | `lisp-owned` | `self-hosting` | `partial` | Lisp-owned evaluator witness with known parity gaps |
| `monotonic-clock` | `—` | `host-observation` | `host-capability` | `confirmed` | Monotonic nanosecond observation exposed without calendar policy |
| `necessary-define` | `0011` | `necessary-form` | `bootstrap` | `confirmed` | DEFINE evaluator-controlled immutable binding form |
| `necessary-lambda` | `0010` | `necessary-form` | `bootstrap` | `confirmed` | LAMBDA evaluator-controlled closure construction |
| `outcome-narration` | `—` | `lisp-owned` | `reasoning` | `confirmed` | Human-facing narration over structured reasoning outcomes |
| `process-authorization` | `—` | `host-authorization` | `host-capability` | `confirmed` | Per-session process allowlist that Lisp code cannot self-grant |
| `process-public-result` | `—` | `lisp-owned` | `stdlib` | `confirmed` | Public process-run result interpretation over process-run-raw |
| `process-run-raw` | `—` | `host-mechanism` | `host-capability` | `confirmed` | OS process execution and captured raw bytes |
| `reason-index` | `—` | `lisp-owned` | `reasoning` | `confirmed` | Finite immutable predicate index with exact linear fallback |
| `reasoning-outcomes` | `—` | `lisp-owned` | `reasoning` | `confirmed` | Data-only proved/unknown/partial/blocked/disputed/invalid outcome algebra |
| `surface-registry-projection` | `—` | `host-mechanism` | `tooling` | `confirmed` | Rust projection/index mechanism reads numeric semantic surface authority without owning spellings |
| `tcp-authorization` | `—` | `host-authorization` | `host-capability` | `confirmed` | Per-session connect/listen allowlists enforced before OS operations |
| `tcp-resource-representation` | `—` | `host-mechanism` | `host-capability` | `partial` | Concrete TcpStream/TcpListener representation still appears in core Value; focused audit is issue 27 |
| `tcp-text-semantics` | `—` | `lisp-owned` | `stdlib` | `confirmed` | Public TCP text decoding over raw socket bytes |
| `time-semantics` | `—` | `lisp-owned` | `stdlib` | `confirmed` | UTC/calendar/deadline meaning derived in Lisp from raw clock observations |
| `tooling-syntax-discovery` | `—` | `derived-tooling` | `tooling` | `confirmed` | Tooling metadata keys syntax by semantic identity and derives spellings from registry |
| `translation-review` | `—` | `lisp-owned` | `knowledge` | `confirmed` | External translation candidate validation and admission review |
| `unification` | `—` | `lisp-owned` | `reasoning` | `confirmed` | Logic-variable unification and occurs-check |
| `utf8-interpretation` | `—` | `lisp-owned` | `stdlib` | `confirmed` | UTF-8 byte-to-text validation and interpretation |

## Interpretation rule

The denominator of every count is the checked-in audited inventory above. A larger `lisp-owned` count is not automatically progress, and a host-owned observation or authorization boundary is not automatically debt. Ownership changes are progress only when they remove duplicate semantic authority or move policy to the layer that can own it without weakening evidence.

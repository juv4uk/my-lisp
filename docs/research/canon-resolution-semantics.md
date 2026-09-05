# Research: Canon Resolution Semantics & Surface Spelling Projections
# Дослідження: Семантика резолюції канону та проекції поверхневих написань

**Status:** Research Investigation / Діагностичний аналіз  
**Date:** 2026-09-06  
**Context:** ADR-004 Closed McCarthy-7 Core (Canonical Semantic Identities)  
**Scope:** Architecture & Semantics only. Zero runtime/evaluator changes.

---

## 1. Problem Statement / Постановка проблеми

With the adoption of [ADR-004](file:///home/agents/GitHub/my-lisp/docs/adr/ADR-004-CLOSED-MCCARTHY7-CORE.md), `my-lisp` decoupled immutable core primitives from ASCII spellings:
```text
The closed McCarthy-7 core consists of exactly seven canonical semantic identities:
{ PRIM_QUOTE, PRIM_ATOM, PRIM_EQ, PRIM_CAR, PRIM_CDR, PRIM_CONS, PRIM_COND }
```
Surface signs in human languages (e.g. English `atom`, Ukrainian `атом`, future Sanskrit SLP1) are projections pointing toward these identities.

The critical architectural question is: **What is the exact operational relation between:**
1. Canonical Primitive Identity (`PRIM_*`)
2. Surface Spelling (source text token)
3. Symbol Interning & Reader Level Representation
4. Lexical Environment (`Environment`)
5. Local Rebinding / Lexical Shadowing (`def`, `let`, `lambda` parameters)
6. Evaluator Dispatch

Specifically, if a user writes:
```lisp
(def atom 42)
(atom x)
```
or
```lisp
(let ((атом (lambda (v) (quote custom))))
  (атом y))
```
**what must happen?** Where does Canon sit relative to the Environment?

---

## 2. The Three Architectural Models / Три архітектурні моделі

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ MODEL A: HARDWIRED RESERVED KEYWORDS (McCarthy 1960 / mccarthy-kernel.s)              │
│                                                                                        │
│   source spelling ──▶ [Hardwired Evaluator Check] ──▶ PRIM_ATOM                        │
│                                │ (no match)                                            │
│                                ▼                                                       │
│                         [Environment] (regular variables only)                         │
│                                                                                        │
│   Consequence: "atom" and "атом" are reserved keywords. Shadowing is forbidden or      │
│   silently ignored. User cannot write `(map car pts)` or `(let ((atom 1)) ...)`.      │
└────────────────────────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────────────────────────┐
│ MODEL B: PURE ENVIRONMENT MUTABILITY (Naive Contract 2.1)                             │
│                                                                                        │
│   source spelling ──▶ [Symbol: "atom"] ──▶ [Environment Lookup]                       │
│                                                  │                                     │
│                         ┌────────────────────────┴────────────────────────┐            │
│                         ▼                                                 ▼            │
│                   found binding                                     not found          │
│                (user value or builtin)                         UnknownSymbol error     │
│                                                                                        │
│   Consequence: Surface names are just mutable keys in a map. If a user defines         │
│   `(def atom 42)`, PRIM_ATOM is permanently lost in that scope. Furthermore, if        │
│   Ukrainian "атом" is unbound in the map, it fails as UnknownSymbol even if "atom" is  │
│   present. Multiple names require multiple redundant map entries.                      │
└────────────────────────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────────────────────────┐
│ MODEL C: CANONICAL IDENTITY + DUAL-PATH RESOLUTION ("The Third Way")                   │
│                                                                                        │
│                         SOURCE TEXT: "(atom x)" or "(атом x)"                          │
│                                           │                                            │
│                                           ▼                                            │
│                            [READER / RESOLVER LAYER]                                   │
│                                           │                                            │
│                 ┌─────────────────────────┴─────────────────────────┐                  │
│                 ▼                                                   ▼                  │
│         Ordinary Reference:                                Explicit Canon Path:        │
│             (atom x)                                         (core PRIM_ATOM x)        │
│                 │                                                   │                  │
│                 ▼                                                   ▼                  │
│      [LOCAL ENVIRONMENT]                                  [CANON TABLE (STATIC)]       │
│      Shadowable by user                                   Immutable root invariant     │
│      (e.g. def, let, lambda)                              Direct access to PRIM_ATOM   │
│                 │                                                   │                  │
│        ┌────────┴────────┐                                          │                  │
│        ▼                 ▼                                          │                  │
│   User Bound?     Unshadowed?                                       │                  │
│   ──▶ User Value  ──▶ Falls back to Canon ──────────────────────────┘                  │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Truth Tables for the 6 Canonical Resolution Cases

We analyze the observable outcome across the 6 test cases specified by the owner under each model:

### Case 1: Unshadowed Historical Spelling `"atom"`
- **Source:** `(atom 'foo)` without prior `(def atom ...)` or surrounding `(let ((atom ...)) ...)`.
- **Model A:** Hardwired match -> Dispatches `PRIM_ATOM` -> Returns `t`.
- **Model B:** Looked up in `Environment` -> Finds root `Value::Builtin("atom")` -> Returns `t`.
- **Model C:** Looked up in `Environment` (unshadowed) or resolved via Canon fallback -> Dispatches `PRIM_ATOM` -> Returns `t`.

### Case 2: Ukrainian Surface Spelling `"атом"` (Unshadowed)
- **Source:** `(атом 'foo)` without prior local definition.
- **Model A:** Hardwired check must contain UTF-8 string `"атом"` hardcoded in Rust/ASM -> Returns `t`.
- **Model B:** Looked up in `Environment` -> Unless a duplicate builtin was explicitly inserted for `"атом"`, fails with `UnknownSymbol: атом`. If duplicate builtin is inserted, it creates two disconnected symbols in the environment.
- **Model C:** Resolver maps `"атом"` to canonical identity `PRIM_ATOM`. Checks local environment for shadow of `"атом"`. If unshadowed -> Dispatches `PRIM_ATOM` -> Returns `t`. Both `"atom"` and `"атом"` invoke the identical underlying semantic identity without code duplication.

### Case 3: Local Binding Named `"atom"` (Shadowed)
- **Source:**
  ```lisp
  (def atom 42)
  (atom 10)
  ```
  or
  ```lisp
  (let ((atom 42)) atom)
  ```
- **Model A:** Either `(def atom 42)` is an error (reserved word), or `(atom 10)` ignores the local binding and executes `PRIM_ATOM 10` -> `t`. Contract 2.1 is violated; user cannot shadow.
- **Model B:** Local environment binds `"atom" -> 42`. Evaluating `(atom 10)` invokes 42 as operator -> `ErrorKind::Type ("not a callable function")`. Evaluating `atom` yields `42`. But `PRIM_ATOM` is completely inaccessible.
- **Model C:** Local environment binds `"atom" -> 42`. Evaluating `(atom 10)` yields `ErrorKind::Type` (honors local lexical scope). Evaluating `atom` yields `42`. **Crucially:** `(core atom)` (or `(primitive atom)`) remains available and continues to invoke `PRIM_ATOM` unconditionally!

### Case 4: Local Binding Named `"атом"` (Shadowed)
- **Source:**
  ```lisp
  (let ((атом (lambda (x) 99)))
    (list (атом 5) (atom 5)))
  ```
- **Model A:** Fails or ignores `"атом"` binding depending on whether hardwired parser permits keyword shadowing.
- **Model B:** If `"атом"` is bound, it evaluates to the lambda (99). But `"atom"` and `"атом"` are completely unrelated strings.
- **Model C:** Local frame shadows `"атом"` -> `(атом 5)` evaluates to `99`. The unshadowed `"atom"` in the same scope still evaluates to `PRIM_ATOM 5` -> `t`. And `(core atom)` / `(core атом)` bypasses the frame entirely to invoke `PRIM_ATOM`.

### Case 5: Direct Canonical Reference
- **Source:**
  ```lisp
  (let ((atom 42)
        (атом "hello"))
    (core atom (quote foo)))
  ```
- **Model A:** Not needed because keywords cannot be shadowed, but `core` would just be another reserved keyword.
- **Model B:** **Does not exist.** Once `"atom"` is rebound, the original primitive is lost in that scope.
- **Model C:** **Guaranteed escape hatch.** `core` is an inviolable syntax form that resolves its argument symbol against the **Static Canon Table**, completely bypassing the `Environment` chain. Returns `t`.

### Case 6: Unknown / Unregistered Spelling
- **Source:** `(parama-anu 'foo)` (unregistered surface sign).
- **Model A:** Fails hardwired check -> falls through to environment -> `UnknownSymbol`.
- **Model B:** Looked up in environment -> `UnknownSymbol`.
- **Model C:** Resolver checks Canon Table -> not a recognized canonical alias -> treated as ordinary dynamic symbol -> looked up in local `Environment` -> if not defined by user, raises named, observable `UnknownSymbol: parama-anu`. Fail-closed invariant preserved.

---

## 4. Comparison Matrix / Матриця порівняння моделей

| Property / Властивість | Model A (Hardwired Keywords) | Model B (Pure Environment) | Model C (Canonical Dual-Path / Third Way) |
| :--- | :--- | :--- | :--- |
| **Lexical Shadowing** (`let ((car ...))`) | ❌ Forbidden / Broken | ✅ Supported | ✅ Supported |
| **Multilingual Surface Synonyms** | ⚠️ Bloats hardwired match | ❌ Requires cloning map keys | ✅ Clean: Many spellings -> One Canon ID |
| **Inviolable Fallback (Wall Behind Cupboard)** | ⚠️ Absolute (no cupboard) | ❌ None (cupboard replaces wall) | ✅ Preserved: `(core ...)` always truth |
| **Epistemic Introspection** | ❌ Opaque | ⚠️ Weak (is it builtin or user?) | ✅ Strong (`symbol` vs `current` vs `canon`) |
| **Substrate Adaptability** (FPGA / C) | Easy for 1 language, hard for N | Requires full runtime hashmap | Trivial: Reader/Resolver emits canonical numeric ID |

---

## 5. Architectural Findings & The Exact Boundary / Архітектурні висновки

### Finding 1: Where Canon Sits Relative to Environment
Canon **does not** sit inside the `Environment` hash map, nor does it replace the `Environment`.
Canon sits **parallel to and behind** the `Environment`:
```text
           [EXPRESSION]
                │
         Is it (core sym)?
         ├── YES ──▶ CANON TABLE (direct dispatch, zero env lookup)
         └── NO  ──▶ ENVIRONMENT LOOKUP
                          │
                   Found in frames?
                   ├── YES ──▶ Return user binding
                   └── NO  ──▶ CANON FALLBACK TABLE ──▶ PRIM_*
```

### Finding 2: Reader / Resolver Stage
Translation from surface spelling (`"атом"`) to canonical identity (`PRIM_ATOM`) should happen during **symbol resolution**, not destructive reader replacement:
- If reader destructively turned the text `"атом"` into `"atom"`, `read` and `print` would violate the property of faithful homoiconicity (re-printing would mangle user language).
- Preserving the symbol's original print name while associating it with a `CanonicalId` preserves both homoiconicity and linguistic autonomy.

### Finding 3: The Role of `core` Form
The proposal in [`docs/PROPOSAL-INVIOLABLE-PRIMITIVES.md`](file:///home/agents/GitHub/my-lisp/docs/PROPOSAL-INVIOLABLE-PRIMITIVES.md) is empirically verified by this analysis:
- Without `(core ...)`, Model C degenerates into Model B whenever a user shadows a name.
- With `(core ...)`, the language achieves the true synthesis: **complete local expressive freedom without loss of semantic truth**.

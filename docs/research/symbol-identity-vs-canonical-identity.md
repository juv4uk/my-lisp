# Research: Surface Symbol Identity vs Canonical Primitive Identity & Canon Access
# Дослідження: Ідентичність поверхневого символу проти канонічної тотожності примітива

**Status:** Research Investigation / Діагностичний аналіз  
**Date:** 2026-09-06  
**Context:** ADR-004 Closed McCarthy-7 Core (Ontology vs Representation)  
**Scope:** Strictly research and design experiment. Zero runtime/evaluator changes.

---

## 1. The Core Homoiconic Dilemma / Головна дилема гомоіконічності

Consider the foundational expression:
```lisp
(eq (quote atom) (quote атом))
```
If `"atom"` and `"атом"` are two human-language surface spellings pointing to the same canonical semantic primitive (`PRIM_ATOM`), **what should this expression evaluate to?**

There are two mutually exclusive paradigms:

### Paradigm 1: Semantic Collapse (Symbols are Canonical Identities)
- The reader / symbol-interner recognizes `"атом"` as an alias for `PRIM_ATOM` and collapses it to the same symbol identity as `"atom"`.
- Outcome: `(eq (quote atom) (quote атом)) -> t`.
- **Fatal Defect (Homoiconicity & Round-Trip Loss):**
  - If a user writes `(print (quote (список атом)))`, the printer would output `(список atom)`.
  - Source text is mutated. The reader erases the user's natural language.
  - S-expressions cease to faithfully represent source text as literal data (homoiconicity is broken).

### Paradigm 2: Layered Separation (Symbol Identity ≠ Primitive Identity)
- Symbols are first-class lexical tokens of data in the L1 domain (`Value::Symbol(Rc<str>)`).
- Symbol equality via `eq` (`PRIM_EQ`) tests atomic identity of the symbol token itself.
- Outcome: `(eq (quote atom) (quote атом)) -> ()` (NIL/false).
- `(quote atom)` evaluates to the symbol `atom`.
- `(quote атом)` evaluates to the distinct symbol `атом`.
- **Both symbols independently map to `PRIM_ATOM` only when evaluated in operator position or resolved against the canonical semantic map.**
- **Preserved Invariant:** Full homoiconicity, lossless `read`/`print` roundtrip, non-destructive coexistence of languages.

```text
source text:
   "atom"                "атом"
     │                     │
     ▼                     ▼
[Value::Symbol("atom")] ≠ [Value::Symbol("атом")]    <-- L1 Data Domain (eq is false)
     │                     │
     └──────────┬──────────┘
                │
                ▼ (Operator Resolution / Canon Semantic Mapping)
            PRIM_ATOM                                 <-- L0 Primitive Identity
```

**Conclusion:** Canonical identity is **not** symbol identity. Canon is a **semantic mapping over symbols**, not an interning mechanism that destroys symbols.

---

## 2. Canon Access Decision Experiment / Експеримент доступу до Канону

How should canonical primitive identities be accessible to programs and users? We compare four candidate models across seven critical criteria.

### The 4 Candidate Models:

1. **Model 1: Invisible Internal Canon (No Explicit User Syntax)**
   - Canon exists exclusively as a specification invariant and external conformance harness contract.
   - Surface names (`atom`, `атом`) are bound in the initial root environment.
   - If a user shadows them (`(def atom 42)`), the original primitive is simply obscured in that scope like any other variable.
   - No `core` keyword, no special syntax.

2. **Model 2: Namespace / Module Access (`canon:atom`, `si:c` style)**
   - Follows the exact precedent already established in `my-lisp` for SI constants (`lib/si.my` -> `si:c`, `si:h`).
   - A dedicated static prefix (e.g. `canon:atom`, `canon:атом`) resolves directly to the immutable primitive identity.
   - Does not require a new special form.

3. **Model 3: Dedicated Special Form `(core sym)`**
   - Introduces an inviolable syntax form intercepted by the evaluator before environment lookup: `(core atom x)` or `(core атом x)`.
   - Cannot be shadowed or redefined.

4. **Model 4: First-Class Canonical Descriptor (`#<primitive PRIM_ATOM>`)**
   - Canonical primitives are distinct first-class values (or introspectable descriptors) obtainable via a standard library function or reader syntax.

---

## 3. Multi-Criteria Evaluation Matrix / Матриця оцінки моделей

| Evaluation Criterion / Критерій | Model 1: Invisible Canon | Model 2: Namespace (`canon:*`) | Model 3: `(core ...)` Form | Model 4: First-Class Value |
| :--- | :--- | :--- | :--- | :--- |
| **1. Shadowing Freedom** (`(let ((atom 42)) ...)`) | ✅ Clean (standard lexical scope) | ✅ Clean (shadows `atom`, not `canon:atom`) | ✅ Clean (shadows `atom`, not `core`) | ✅ Clean |
| **2. Homoiconicity** (data is code) | ✅ 100% pure (ordinary symbols) | ✅ Pure (ordinary qualified symbol) | ⚠️ Requires special reader/eval syntax | ⚠️ Opaque descriptor value |
| **3. Macro Behavior & Expansion** | ✅ Standard hygiene / expansion | ✅ Composes cleanly in macros | ⚠️ Macros must treat `core` specially | ⚠️ Embedding values in code |
| **4. Read / Print Roundtrip** | ✅ Perfect roundtrip | ✅ Perfect roundtrip | ⚠️ Preserved if list, but AST special | ❌ Descriptors cannot read-back safely |
| **5. Evaluator Complexity** | ✅ Zero evaluator changes | 🟢 Minimal (prefix resolution) | 🔴 High (new dispatch branch) | 🟡 Medium (new Value variant) |
| **6. C / FPGA Portability** | ✅ Trivial (symbols are numbers) | 🟢 Easy (static table entry) | 🔴 Requires extra hardware state | 🟡 Requires new hardware tag |
| **7. Risk of 8th Primitive / Magic Leak** | **🟢 ZERO RISK** (Strictly closed) | **🟢 ZERO RISK** (Naming convention) | **🔴 HIGH RISK** (`core` becomes pseudo-primitive #8) | **🟡 MEDIUM RISK** (New value domain) |

---

## 4. Key Diagnostic Insights / Ключові діагностичні висновки

### Insight 1: The Danger of `(core ...)` as an 8th Primitive
If `my-lisp` adopts `(core <op> ...)` as an inviolable evaluator form, `core` becomes an operation with observable semantics that cannot be expressed via McCarthy's seven primitives. It risks becoming an **8th primitive through the backdoor** (evaluator privilege).

### Insight 2: Model 1 vs Model 2 Synergy
- In **Model 1**, the language maintains absolute purity: exactly 7 primitives, ordinary lexical environments.
- In **Model 2**, canonical names (`canon:atom`, `canon:атом`) are simply pre-bound static symbols pointing to the immutable builtin operations, exactly like `si:c` in BIPM SI physics. If a user shadows `atom`, `canon:atom` remains untouched in the root frame. No special evaluator forms are needed.

### Insight 3: Symbol Identity Invariant
Under all valid models:
```lisp
(eq (quote atom) (quote атом)) -> () ; FALSE
```
Symbols are distinct tokens of human text. Their semantic convergence occurs at the **binding and invocation layer**, never by erasing the symbol's lexical identity.

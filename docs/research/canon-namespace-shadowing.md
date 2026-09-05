# Research: Canon Namespace Shadowing & Inviolability Tradeoffs
# Дослідження: Затінення простору імен Canon та компроміси непорушності

**Status:** Research Investigation / Діагностичний експеримент  
**Date:** 2026-09-06  
**Context:** ADR-004 Closed McCarthy-7 Core (Canon Access Models)  
**Scope:** Strictly conceptual & diagnostic. Zero runtime/evaluator changes.

---

## 1. The Core Tension / Головне протиріччя

If `canon:atom` (or Ukrainian `canon:атом`) is introduced as the canonical escape hatch to access the immutable primitive `PRIM_ATOM`:

```text
WHAT HAPPENS IF A USER WRITES:
(let ((canon:atom 42))
  (canon:atom x))
```

This presents a sharp dilemma between two language design ideals:
1. **Uniform Lexical Symmetry:** Every symbol that can be referred to can be locally shadowed (`(let ((x ...)) ...)` has zero exceptions).
2. **True Inviolability:** There exists an unconditional, unblockable path to the foundational primitives ("the wall behind the cupboard").

---

## 2. Comparison of the Three Approaches / Порівняння трьох підходів

### Variant A: `canon:atom` as an Ordinary Root Binding
- `canon:atom` is merely a standard `Value::Builtin` pre-installed in the root `Environment` alongside `atom`.
- **Can user bind it?** YES. `(let ((canon:atom 42)) ...)` binds `canon:atom` to `42`.
- **Evaluator changes:** **ZERO.** No special form, no parser hooks, no prefix interception.
- **Portability (C / FPGA):** Trivial. Identical to any standard environment binding.
- **Pseudo-primitive risk:** **ZERO.**
- **Consequence:** The access is **convenient and conventional, but NOT inviolable**. A user or misbehaved macro can shadow both `atom` and `canon:atom`.
- **Verdict:** Honest, minimal, and non-magical. But "the wall" can still be temporarily covered by a cupboard if someone explicitly names a cupboard `canon:atom`.

---

### Variant B: `canon:*` Namespace is Protected by Evaluator/Environment
- The environment or binder checks the symbol prefix: any attempt to bind, `def`, or `let`-bind a symbol starting with `canon:` is rejected at parse/eval time (`ErrorKind::InvalidForm` or `ErrorKind::ProtectedNamespace`).
- **Can user bind it?** NO. Explicitly prohibited.
- **Evaluator changes:** MODERATE. `Environment::define`, `create_lambda` parameter validator, and `let` macro must inspect symbol prefixes.
- **Portability (C / FPGA):** Lowers portability. Requires hardware or minimal C runtime to understand string prefix semantics and enforce runtime permission barriers.
- **Pseudo-primitive risk:** **LOW-TO-MEDIUM.** Does not add an 8th primitive, but creates a **privileged lexical category** (reserved prefix namespace).
- **Consequence:** Provides true inviolability, but introduces asymmetrical rules: some valid symbol literals cannot be used as lexical variable names.

---

### Variant C: `canon:atom` Resolved via Separate Canonical Map
- `canon:*` is not looked up in the `Environment` chain at all.
- During evaluation, when an operator or variable has the prefix `canon:`, the evaluator routes it directly to a static, compile-time/ROM table (`CANON_TABLE`).
- **Can user bind it?** User can bind it in `let`, but the evaluator **ignores the shadow** when resolving `canon:*`, or raises a static error.
- **Evaluator changes:** HIGH. Evaluator symbol resolution must split into two distinct lookup engines.
- **Portability (C / FPGA):** Splits execution into two address spaces (Environment vs ROM dispatch).
- **Pseudo-primitive risk:** **HIGH.** Essentially reinvents `(core ...)` syntax form disguised as a symbol prefix naming convention.

---

## 3. Four Probe Expressions / Чотири контрольні вирази

We examine the observable outcomes across the four decisive expressions identified by the owner:

### Probe 1: `(eq 'atom 'атом)`
- **Candidate Status:** **Proposed Semantic Invariant** (Under review).
- **Behavior:**
  - Evaluates to `()` (NIL / false) across all three variants.
  - Reader interns two distinct symbols: `Symbol("atom")` and `Symbol("атом")`.
  - Preserves lossless `read`/`print` roundtrip without mutating human language.
  - Convergence to `PRIM_ATOM` occurs strictly at the operator/evaluation mapping stage.

### Probe 2: `(let ((atom 42)) atom)`
- **Behavior:**
  - Evaluates to `42` across all three variants.
  - Lexical shadowing of historical names remains 100% legal, valid, and unhindered (Contract 2.1 preserved).

### Probe 3: `(let ((canon:atom 42)) canon:atom)`
- **Variant A:** Evaluates to `42`. `canon:atom` is treated symmetrically with all other symbols.
- **Variant B:** Raises structured error (`ProtectedNamespace` / `InvalidForm`): user is forbidden from shadowing the `canon:` namespace.
- **Variant C:** Evaluates either to an error or silently ignores the binding and returns `PRIM_ATOM` (violating lexical transparency).

### Probe 4: `(let ((atom 42)) (canon:atom 'x))`
- **Variant A:** Evaluates to `t` (calls `PRIM_ATOM 'x'`). In ordinary idiomatic usage, this provides the clean escape hatch to the canonical primitive without requiring any evaluator privileges.
- **Variant B:** Evaluates to `t` (calls `PRIM_ATOM 'x'`), with an enforced guarantee that `canon:atom` could not have been shadowed.
- **Variant C:** Evaluates to `t` (calls `PRIM_ATOM 'x'`) via direct ROM dispatch.

---

## 4. Multi-Criteria Tradeoff Matrix / Матриця компромісів

| Criterion / Критерій | Variant A: Ordinary Root Binding | Variant B: Protected Namespace | Variant C: Separate Map Dispatch |
| :--- | :--- | :--- | :--- |
| **1. Lexical Symmetry** | ✅ 100% symmetric | ⚠️ Asymmetric (reserved prefix) | ❌ Broken (shadows ignored) |
| **2. Evaluator Simplicity** | 🟢 ZERO new evaluator logic | 🟡 Prefix check in binders | 🔴 Dual resolution paths |
| **3. True Inviolability** | ⚠️ Conventional, not absolute | ✅ Absolute | ✅ Absolute |
| **4. C / FPGA Feasibility** | 🟢 Trivial | 🟡 String inspection in hardware | 🔴 Dual lookup engines |
| **5. Escape Hatch Utility** | ✅ Works in 99.9% real scenarios | ✅ Works 100% | ✅ Works 100% |
| **6. Absence of Magic / 8th Prim** | **🟢 ZERO MAGIC** | 🟡 Policy rule in runtime | 🔴 Reinvents `core` form secretly |

---

## 5. Synthesis & Architectural Recommendation / Синтез і рекомендація

1. **Why Variant C is Rejected:**
   Variant C is merely the high-risk `(core ...)` form wearing the mask of a colon prefix. It creates two parallel evaluators.

2. **The Real Choice: Variant A vs Variant B:**
   - If `my-lisp` prioritizes **absolute semantic minimalism and zero evaluator magic**, **Variant A** is superior: `canon:atom` and `canon:атом` are simply standard aliases in the root environment. In practical code, shadowing `atom` leaves `canon:atom` completely intact. If a programmer deliberately writes `(let ((canon:atom ...)))`, they shoot themselves in the foot by choice, just as someone in Common Lisp can shadow standard names in a local scope.
   - If `my-lisp` prioritizes **absolute inviolability** above all else, **Variant B** is the only coherent way, but it pays the cost of introducing a non-uniform lexical rule (a reserved namespace).

3. **Recommended Research State:**
   - Keep `canon:*` as the **leading candidate for canonical access**.
   - Keep `canon:*` under **Variant A (ordinary root binding)** until empirical evidence demonstrates a concrete need for the protective restriction of Variant B.
   - Retain `(eq 'atom 'атом) -> ()` as a **Proposed Semantic Invariant**.

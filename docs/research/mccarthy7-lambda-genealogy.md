# Research: McCarthy-7 and Lambda Genealogy in my-lisp
# Дослідження: McCarthy-7 та генеалогія Lambda в my-lisp

**Status:** Research finding (Not a normative rule modification)  
**Date:** 2026-09-05  
**Context:** ADR-004 Closed McCarthy-7 Core Diagnostic Pass

---

## 1. Question / Дослідницьке питання

What is the actual semantic status of `lambda` in `my-lisp`? Specifically:
- Can the currently observable contract of `lambda` be derived strictly from the closed L0 semantic primitives (`quote`, `atom`, `eq`, `car`, `cdr`, `cons`, `cond`) and L1 admitted value domains (symbols, pairs)?
- If not, what exact capabilities are required, and where does evaluator support enter?

Яким є реальний семантичний статус `lambda` в `my-lisp`? Зокрема:
- Чи можна вивести спостережуваний контракт `lambda` суто із замкнених семантичних примітивів L0 (`quote`, `atom`, `eq`, `car`, `cdr`, `cons`, `cond`) та допущених доменів значень L1 (символи, пари)?
- Якщо ні, які саме спроможності є необхідними, і в якій точці виникає підтримка з боку обчислювача (evaluator)?

---

## 2. Observable Lambda Contract / Спостережуваний контракт Lambda

In the current `my-lisp` reference engine (`crates/my-lisp`), `lambda` provides five observable properties:

1. **First-Class Value Status:** `(lambda (x) x)` evaluates to a distinct callable value (`Value::Closure`) that can be bound, passed as an argument, stored in pairs, and returned.
2. **Lexical Environment Capture:** At evaluation time, `(lambda ...)` captures the active lexical bindings (`closure.environment = environment.clone()`). Outer variables remain lexically scoped, immune to dynamic re-binding by callers.
3. **Lambda-List Arity Protocols:**
   - Fixed parameters: `(lambda (x y) ...)` enforces exact arity.
   - Variadic / Rest parameter: `(lambda (x . rest) ...)` or `(lambda args ...)` packs remaining evaluated arguments into a list.
4. **Call Frame Isolation:** Applying a closure creates an isolated child frame (`local_environment = closure.environment.child()`). Parameter bindings do not mutate or leak into the caller's environment.
5. **Operator Position Application:** When an expression `(f arg1 ...)` has a closure value in operator position `f`, arguments are evaluated in the calling environment, bound to parameters in the child frame, and the body expressions are evaluated in sequence.

---

## 3. Current Rust Mechanisms / Поточні механізми в Rust

In `crates/my-lisp/src/eval/`:
- **Special Form Interception:** `eval/mod.rs` intercepts `Some("lambda")` in `evaluate_list` to inhibit argument evaluation.
- **Dedicated Data Model Variant:** `Value::Closure(Rc<Closure>)` exists alongside atomic and pair types in `value.rs`.
- **Environment Tree Machinery:** `Environment` is implemented as an explicit tree of heap-allocated reference-counted frames (`Rc<RefCell<Frame>>`) with `.child()` and `.define()`.
- **Application Dispatch Hook:** Operator application in `eval/mod.rs` explicitly branches on `Value::Closure` and routes to `closures::apply`.

---

## 4. Attempted Reduction to L0 + L1 / Спроба редукції до L0 + L1

Can this be constructed using only McCarthy's 7 primitives + pairs/symbols?

### What can be represented in L0 + L1:
1. **Closure representation as data:** A closure record can be represented as a tagged list using `cons` and `quote`:
   ```lisp
   (cons (quote closure) (cons params (cons body (cons env (quote ())))))
   ```
2. **Environment representation as an alist:** Bindings can be structured as an association list of pairs `((var . val) ...)` using `cons`.
3. **Parameter-argument binding:** An associative pairing function (`bind-params`) can be written purely using `car`, `cdr`, `cons`, `cond`, and `eq`.
4. **Environment lookup:** `env-lookup` can be implemented recursively using `car`, `cdr`, `eq`, and `cond`.

### Where reduction encounters boundaries:
Even if a macro or transformation synthesizes a `(closure params body env)` structure:
- **Lexical Capture Boundary:** L0 primitives contain no operator to inspect, capture, or reify the currently executing environment. McCarthy's 1960 `eval[e; a]` bypassed this by taking the environment `a` as an explicit external formal parameter. In surface syntax, `(lambda ...)` does not take `a`; the evaluator must implicitly supply it.
- **Operator Application Hook Boundary:** In a standard Lisp evaluator, evaluating `((closure ...) arg)` treats `(closure ...)` as an ordinary form. Without an evaluator hook that recognizes closures in operator position, the evaluator would attempt to call the symbol `closure` as a function.

---

## 5. Decomposing the "Lambda" Phenomenon / Декомпозиція феномену Lambda

Rather than treating `lambda` as a monolithic capability, the investigation reveals distinct sub-mechanisms:

```text
┌─────────────────────────┬────────────────────────────────────────────────────────┐
│ Sub-mechanism           │ Classification Candidate                               │
├─────────────────────────┼────────────────────────────────────────────────────────┤
│ 1. Surface syntax       │ Derived / Syntactic sugar over closure construction    │
│ 2. Parameter binding    │ Derived list algebra (reproducible via cons/car/cdr)   │
│ 3. Closure value record │ Admitted Domain representation (tagged pair structure) │
│ 4. Lexical capture      │ Admitted Evaluator Capability (AEC)                    │
│ 5. Call-frame isolation │ Admitted Evaluator Capability (AEC)                    │
│ 6. Operator application │ Admitted Evaluator Capability (AEC)                    │
└─────────────────────────┴────────────────────────────────────────────────────────┘
```

---

## 6. Current Diagnostic Verdict / Поточний діагностичний вердикт

```text
MCCARTHY7-LAMBDA-STATUS:
REQUIRES ADMITTED EVALUATOR CAPABILITY (AEC)
```

**Supported by:**
- Analysis of current Rust evaluator semantics (`crates/my-lisp/src/eval/`).
- Concrete metacircular evidence in `lib/meta-eval.my`, where closure creation and application require explicit branches in `my-eval` and `my-apply`.
- Inability to reify the caller's active environment or register operator-position call hooks using only L0 primitives.

**Claims explicitly NOT made:**
- **No mathematical impossibility theorem:** We do not claim that no mathematical encoding (e.g. combinatory logic / SKI combinators or explicit continuation-passing transforms) could ever simulate function abstraction under pure rewriting.
- **No hasty alteration of ADR-004:** In ADR-004, `lambda` remains classified as `? UNRESOLVED` pending independent confirmation on a second witness substrate.

---

## 7. Next Steps & Independent Witness / Наступні кроки та незалежний свідок

1. **Independent Witness Pass:** Construct an isolated executable demonstration in `meta-eval.my` or Guile that tests whether closure capture and operator application can be isolated from the evaluator's primitive set.
2. **Decomposition Verification:** Verify whether the surface syntax `(lambda (x) ...)` can be formally desugared once environment capture and application hooks are explicitly admitted.
3. **Classification Transition:** Once the independent witness is recorded, propose an update to ADR-004 transitioning `lambda` from `? UNRESOLVED` to `AEC: Lexical-Closure & Application Capability`.

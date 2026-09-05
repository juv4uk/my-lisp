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

## 7. Independent Witness B: Metacircular Evaluator (`lib/meta-eval.my`)
## Незалежний свідок B: Метациркулярний обчислювач (`lib/meta-eval.my`)

To establish whether the necessity of evaluator intervention is merely an artifact of the Rust runtime (`crates/my-lisp/src/eval/`) or an intrinsic property of the observable semantics, a second independent witness was executed against `lib/meta-eval.my` via test `meta_eval_lambda_witness_env_capture_and_application` in `crates/my-lisp/tests/mccarthy.rs`.

Щоб перевірити, чи потреба у втручанні обчислювача є лише артефактом Rust-рантайму (`crates/my-lisp/src/eval/`), чи іманентною властивістю спостережуваної семантики, виконано другого незалежного свідка над `lib/meta-eval.my` через тест `meta_eval_lambda_witness_env_capture_and_application` у `crates/my-lisp/tests/mccarthy.rs`.

### Witness A (Explicit Lexical Environment Capture):
In `lib/meta-eval.my`, `my-eval` receives the lexical environment `env` as an explicit evaluation parameter:
```lisp
((eq (car expr) (quote lambda))
 (list (quote closure) (second expr) (cdr (cdr expr)) env))
```
When evaluating `(lambda (x) outer)` under `witness-env = ((outer . 7))`:
- The surface expression does **not** mention `witness-env`.
- The evaluation produces the observable tagged list data:
  ```lisp
  (closure (x) (outer) ((outer . 7)))
  ```
- **Conclusion:** Constructing the tagged list is ordinary list algebra (`cons`, `quote`), but obtaining the implicit lexical environment `env` requires the evaluator's internal context.

### Witness B (Operator Application & Frame Extension):
In `lib/meta-eval.my`, `my-apply` detects the closure tag and binds arguments:
```lisp
((eq (car fn) (quote closure))
 (my-eval-body (third fn)
   (bind-params (second fn) args (car (cdr (cdr (cdr fn)))))))
```
When evaluating `(my-apply closure-val (cons 42 (quote ())))`:
- `bind-params` creates an extended association list: `((x . 42) (outer . 7))`.
- Parameter binding itself is pure list algebra (`cons`, `car`, `cdr`).
- However, ordinary expression application `((lambda (x) outer) 42)` requires the evaluator to recognize the closure value in operator position and invoke the application protocol instead of treating `closure` as an undefined function name.

Both independent implementations (Rust host evaluator and `meta-eval.my` in Lisp) exhibit the identical boundary:
1. **List algebra** handles parameter binding, alist representation, and closure records.
2. **Evaluator capability** is required for implicit lexical environment capture and operator-position application dispatch.

---

## 8. Refined Decomposed Status / Уточнений декомпонований статус

```text
┌────────────────────────────────────────┬───────────────────────────────────────────┐
│ Component / Компонент                  │ Proven Semantic Status                    │
├────────────────────────────────────────┼───────────────────────────────────────────┤
│ Surface syntax `(lambda (x) ...)`      │ DERIVED (syntactic sugar over capture)    │
│ Parameter binding (`bind-params`)      │ DERIVED (pure McCarthy-7 list algebra)    │
│ Closure data structure                 │ ADMITTED DATA (tagged pair/list domain)   │
│ Lexical environment capture            │ ADMITTED EVALUATOR CAPABILITY (AEC)       │
│ Operator-position application hook     │ ADMITTED EVALUATOR CAPABILITY (AEC)       │
└────────────────────────────────────────┴───────────────────────────────────────────┘
```

**Normative State (ADR-004):**
In ADR-004 (`docs/adr/ADR-004-CLOSED-MCCARTHY7-CORE.md`), `lambda` remains cataloged as `? UNRESOLVED` until the owner decides whether to formally graduate it as an Admitted Evaluator Capability (AEC) or keep it in the diagnostic tier.

---

## 9. Next Steps / Наступні кроки

1. Present the two independent witnesses (Rust evaluator + `meta-eval.my`) and the decomposed taxonomy to the owner.
2. If accepted, formulate the precise wording for ADR-004 to categorize `lambda` not as an 8th primitive, but as an Admitted Evaluator Capability (AEC) over closure values.


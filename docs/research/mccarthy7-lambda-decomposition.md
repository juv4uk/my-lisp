# Research: McCarthy-7 Lambda Decomposition
# Дослідження: Декомпозиція lambda в McCarthy-7

**Status:** Research Investigation — `MCCARTHY7-LAMBDA-DECOMPOSITION`  
**Date:** 2026-09-06  
**Context:** ADR-004 §5 — `lambda` status `? UNRESOLVED`  
**Scope:** Classify each observable component independently. No runtime changes. No ADR classification updates yet. No impossibility claims.

---

## Goal / Мета

`lambda` in the current Rust implementation is a single special form (`closures.rs`). The goal of this decomposition is **not** to give `lambda` a single status — but to ask, component by component, whether each observable part of lambda behavior is:

- constructible purely from L0 primitives + L1 admitted domains
- or requires an evaluator capability that cannot be expressed in L0+L1 without circularity

```text
Do not change runtime.
Do not update ADR classification yet.
Do not claim impossibility.
Record status per component: DERIVED / ADMITTED / PARTIAL / UNRESOLVED
```

---

## Component Map / Карта компонентів

```text
(lambda (x y) body)
    │
    ├── C1: Syntax Recognition
    │        "lambda" as head symbol triggers special form path
    │
    ├── C2: Parameter List Parsing
    │        (x y) extracted as formal argument list
    │
    ├── C3: Body Preservation
    │        body expression captured without evaluation
    │
    ├── C4: Environment Capture
    │        current lexical environment captured at lambda creation site
    │
    ├── C5: Closure Data Construction
    │        closure = (closure-tag params body captured-env)
    │        stored as a first-class value
    │
    ├── C6: Operator Detection at Application
    │        ((lambda ...) arg) — evaluator identifies operator as closure
    │
    ├── C7: Argument Binding
    │        formal params x <- actual arg 42, y <- actual arg ...
    │
    ├── C8: Environment Extension
    │        captured-env extended with new frame {x->42, y->...}
    │
    └── C9: Body Evaluation in Extended Environment
             body evaluated in new extended env
```

---

## Component Analysis / Аналіз компонентів

### C1 — Syntax Recognition

**Observable contract:**
When the head of a list is the symbol `lambda`, evaluation does not evaluate
sub-expressions in the normal applicative-order way.

**Current Rust mechanism:**
`eval/mod.rs` -> `evaluate_list` -> `match items[0].kind.as_symbol()` ->
`Some("lambda")` branch. Hardwired string match before environment lookup,
identical to `quote` and `cond`.

**Meta-eval witness:**
`lib/meta-eval.my` -> `my-eval` -> `(cond ... ((eq (car expr) (quote lambda)) ...))`.
Head recognized using `eq` + `quote` + `car` — all L0.

**Constructible from L0+L1?**
Recognition via `eq`/`quote`/`car`: yes, fully L0.
The *interception* (preventing subexpression evaluation) requires the evaluator
dispatch to exist — a bootstrap dependency, not an L0 gap.

**Status:** `PARTIAL`
Recognition is L0-derivable. Special-form interception requires host evaluator
as bootstrap. Metacircular evaluator resolves this at the self-hosting level.

---

### C2 — Parameter List Parsing

**Observable contract:**
`(lambda (x y) body)` -> parameter list `(x y)` extracted as list of symbols.

**Current Rust mechanism:**
`closures.rs` -> `items[1]` = second sub-expression = `Value::List` of symbols.

**Meta-eval witness:**
`(second expr)` = `(car (cdr expr))` — pure L0 composition.

**Constructible from L0+L1?**
Yes. `car`/`cdr` navigate to the parameter list. The list itself is L1 pairs of L1 symbols.

**Hidden evaluator capability?** None.

**Status:** `DERIVED`

---

### C3 — Body Preservation

**Observable contract:**
`body` is NOT evaluated at lambda creation time. Stored as data for later.

**Current Rust mechanism:**
`closures.rs` captures `items[2]` without calling `evaluate`.

**Meta-eval witness:**
`(cdr (cdr expr))` gives body as unevaluated pair chain — pure L0 list navigation.

**Constructible from L0+L1?**
`cdr`/`car` for extraction (L0). Storing as pair chain (L1). Non-evaluation is
equivalent to quoting — `PRIM_QUOTE` is L0. Requires being inside C1 dispatch path.

**Hidden evaluator capability?** Only the C1 bootstrap dependency.

**Status:** `DERIVED`

---

### C4 — Environment Capture

**Observable contract:**
At `(lambda ...)` creation time, the **current lexical environment** is bound
into the closure — definition-time env, not call-time env.

**Current Rust mechanism:**
`closures.rs` -> `Value::Lambda { env: Rc<Environment>, ... }` — current `env`
reference cloned into closure struct.

**Meta-eval witness:**
`(list (quote closure) (second expr) (cdr (cdr expr)) env)` — `env` is the
explicit parameter to `my-eval`, stored as fourth element via `cons`. Construction
is pure L0+L1 once `env` is available.

**Constructible from L0+L1?**
The *data construction* (consing env into closure): yes, L0+L1.
The *availability of env*: requires the evaluator to thread env as an explicit
argument through all recursive calls. This is an admitted evaluator capability.

**Status:** `PARTIAL`
Construction is L0+L1. Having `env` as a first-class threaded value requires
the evaluator to make it available — an admitted evaluator invariant.

---

### C5 — Closure Data Construction

**Observable contract:**
Result of `(lambda (x) body)` is a first-class closure value:
`{tag, params, body, captured-env}`.

**Current Rust mechanism:**
`Value::Lambda { params, body, env }` — dedicated Rust enum variant.

**Meta-eval witness:**
`(list (quote closure) params body env)` — a tagged L1 pair list.

**Constructible from L0+L1?**
Yes. `list` = nested `cons` = L0. `(quote closure)` = L0. The closure as a data
structure is a tagged L1 pair chain.

**Hidden evaluator capability?** None, once `env` is available (C4).

**Status:** `DERIVED`
Closure data structure is fully representable as a tagged L1 list using L0.

---

### C6 — Operator Detection at Application

**Observable contract:**
`((lambda (x) x) 42)` — evaluator detects the operator is a closure, not a builtin.

**Current Rust mechanism:**
`evaluate_list` -> operator evaluated -> `Value::Lambda` variant matched.

**Meta-eval witness:**
`my-apply` -> `(cond ((eq (car fn) (quote closure)) ...) ...)` — detects tagged
list via `car`/`eq`/`quote`, all L0.

**Constructible from L0+L1?**
Yes. Tagged-list detection is pure L0 pattern matching (given C5 closure representation).

**Status:** `DERIVED`

---

### C7 — Argument Binding

**Observable contract:**
Formal params `(x y)` bound one-to-one to actual args `(42 ...)`.

**Current Rust mechanism:**
`apply_lambda` -> zips params and args into new `Environment::extend`.

**Meta-eval witness:**
`bind-params` in `meta-eval.my` — recursive `car`/`cdr` zip of two lists into an
association list. Pure L0 list operations.

**Constructible from L0+L1?**
Yes. Parallel list traversal with `car`/`cdr`/`cons` — all L0.

**Status:** `DERIVED`

---

### C8 — Environment Extension

**Observable contract:**
Captured environment extended with new binding frame `{x->42, y->...}`.

**Current Rust mechanism:**
`Environment::extend(parent_env, bindings)` — new frame over captured env.

**Meta-eval witness:**
`(extend-env bindings captured-env)` in `meta-eval.my` — new association list
layer prepended to existing env via `cons`.

**Constructible from L0+L1?**
Environment as pair chain = L1. Extension = `cons` new binding layer = L0. Yes.

**Hidden evaluator capability?**
The env's *semantic role* (governing symbol resolution) is an admitted evaluator
invariant. The data structure representing it is fully L1.

**Status:** `DERIVED`
Extension as list prepend is L0+L1. Semantic role is an admitted invariant.

---

### C9 — Body Evaluation in Extended Environment

**Observable contract:**
`body` evaluated in the newly extended environment; result is the return value
of the lambda application.

**Current Rust mechanism:**
`evaluate(body_expr, extended_env)` — recursive call to `evaluate` with new env.

**Meta-eval witness:**
`(my-eval body (extend-env ...))` — recursive call to `my-eval` with extended env.

**Constructible from L0+L1?**
`my-eval` is the evaluator. This is the evaluator calling itself — metacircular
self-application. At the metacircular level, derivable. At the bootstrap level,
the host evaluator must provide this capability.

**Hidden evaluator capability?**
**YES — and this is the fundamental one.** Evaluation itself (reducing an expression
to a value) is the admitted capability that L0 primitives *define the contract for*
but cannot *provide* without circularity. The evaluator must exist to evaluate.

**Status:** `ADMITTED`
Recursive evaluation in an environment is an admitted evaluator capability —
the core semantic engine that L0 specifies but does not derive.

---

## Summary Table / Зведена таблиця

| # | Component | L0+L1? | Hidden Capability | Status |
| :- | :--- | :--- | :--- | :--- |
| C1 | Syntax recognition | Partial (recognition yes, interception needs host) | Bootstrap dispatch | `PARTIAL` |
| C2 | Parameter list parsing | Yes (`car`/`cdr`) | None | `DERIVED` |
| C3 | Body preservation | Yes (quote equivalence + C1 path) | C1 bootstrap only | `DERIVED` |
| C4 | Environment capture | Data construct yes; `env` availability needs host | Evaluator must thread `env` | `PARTIAL` |
| C5 | Closure data construction | Yes (`cons`/`quote`) | None, once `env` available | `DERIVED` |
| C6 | Operator detection | Yes (`car`/`eq`/`quote`) | None beyond C1 | `DERIVED` |
| C7 | Argument binding | Yes (L0 list ops) | None | `DERIVED` |
| C8 | Environment extension | Yes (`cons` over pair chain) | Env semantic role = admitted invariant | `DERIVED` |
| C9 | Body evaluation | Metacircular only; bootstrap = host | Recursive eval-in-environment | `ADMITTED` |

---

## Architectural Findings / Архітектурні висновки

### Finding 1: Lambda decomposes asymmetrically

Six of nine components (C2, C3, C5, C6, C7, C8) are fully derivable from L0+L1,
given:
- Closures represented as tagged L1 lists
- Environments represented as L1 pair chains

The remaining three split into two structurally distinct categories:

```text
C1, C4 — PARTIAL (bootstrap dependency):
  Not an L0 gap. The evaluator must exist and thread env.
  Resolved at the metacircular level (meta-eval.my witnesses this).
  In a self-hosting my-lisp, these become derivable via meta-eval.

C9 — ADMITTED (fundamental):
  Evaluation itself cannot be derived from L0 without circularity.
  L0 defines what evaluation must do; it does not provide evaluation.
  This is the essential bootstrap — shared by all L0 primitives too.
```

### Finding 2: The essential admitted capability is already implied by L0

The single non-derivable core of `lambda` is:

```text
The ability to evaluate an expression in a given environment.
```

But this is not a new capability. Consider:

```text
(atom x)  ->  evaluator applies PRIM_ATOM to the value of x
```

Every L0 primitive already requires the evaluator to *apply* it to *values* — which
means the evaluator's ability to reduce expressions is already an admitted capability
of L0 itself. Lambda makes this recursive structure *visible and reusable*, but does
not add to it.

Consequence: if the evaluator's recursive capability is already admitted by L0,
then `lambda` introduces **no new admitted capability** beyond what L0 already requires.

### Finding 3: Proposed classification (candidate — NOT yet ADR-ratified)

```text
lambda = L2 DERIVED FORM (candidate)

Dependency declaration:
  - L0: quote, car, cdr, cons, eq, cond
  - L1: symbols, pairs
  - Admitted evaluator capability: recursive eval-in-environment
    (already admitted by L0 semantics; lambda does not add to it)
  - No new primitive beyond the closed McCarthy-7 set
```

This is a **research candidate**, not a normative decision. Owner ratification
required before updating ADR-004 §5 table.

---

## Open Questions / Відкриті питання

**Q1: Is `eval-in-environment` a new admitted capability, or already implied by L0?**
L0 primitives require an evaluator to apply them to values. If C9's capability
is already entailed by L0's own semantics, `lambda` introduces zero new admitted
capabilities. This would strengthen the L2 candidate significantly.

**Q2: Does `meta-eval.my` constitute sufficient derivation proof?**
Or is a separate algebraic reduction required — showing the closure representation
satisfies all observable contracts via L0 axioms alone, without relying on the
host evaluator's existence as a premise?

**Q3: Frame isolation — derivable from L0 or admitted constraint?**
The guarantee that inner frames do not bleed into outer is demonstrated empirically
(test `C` in `meta_eval_lambda_witness_env_capture_and_application`). Is it
derivable from L0 list properties alone, or does it require an admitted constraint
on environment structure (e.g., "environments are immutable association lists")?
Tracing this formally would close Q2 as well.

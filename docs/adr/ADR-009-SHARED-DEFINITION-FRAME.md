# ADR-009: Shared Definition Frame Semantics
# ADR-009: Семантика спільного frame визначень

**Status:** Accepted / Прийнято  
**Date:** 2026-09-10  
**Scope:** top-level `def` / `define` closures and the metacircular parity model

## 1. Decision / Рішення

`my-lisp` uses a **shared lexical definition frame**, not a frozen lexical snapshot, for closures created in a top-level definition frame.

A closure captures the identity of its lexical frame. Bindings added or replaced later in that same frame are therefore visible when the closure is eventually called.

Canonical witness:

```lisp
(def f (lambda () g))
(def g 42)
(f)
```

must evaluate to:

```text
42
```

This is **not dynamic scope**. Parameters and bindings introduced in lexical child frames continue to shadow names from the shared definition frame, and unrelated caller-local bindings must not leak into a closure.

## 2. Observable rules / Спостережувані правила

1. **Later binding visibility**

   ```lisp
   (def f (lambda () g))
   (def g 42)
   (f)
   ```

   returns `42`.

2. **Later replacement in the same frame is visible**

   ```lisp
   (def g 1)
   (def f (lambda () g))
   (def g 2)
   (f)
   ```

   returns `2`.

3. **Call-before-binding still fails**

   ```lisp
   (def f (lambda () g))
   (f)
   (def g 42)
   ```

   The call occurs before `g` exists in the frame and therefore observes an unresolved name. The existence of a later source form does not grant retroactive visibility.

4. **Lexical shadowing wins**

   ```lisp
   (def g 1)
   (def f (lambda (g) g))
   (def g 2)
   (f 9)
   ```

   returns `9`.

5. **Nested lexical capture remains lexical**

   A closure created inside another closure keeps its child-frame bindings while still sharing the top-level parent frame. Later top-level updates do not overwrite captured parameters or local bindings.

## 3. Why Variant B / Чому варіант B

The reference evaluator already implements this rule deliberately:

```text
Environment = Rc<RefCell<Frame>>
closure.environment = environment.clone()
def -> environment.define(...)
```

`Environment::clone()` preserves frame identity rather than copying the frame's `HashMap`. `evaluate_definition` then mutates that same frame after evaluating the definition value. The reference implementation even documents this as the mechanism that makes recursive definitions visible after binding.

Changing the reference evaluator to snapshot semantics would therefore not be a neutral cleanup. It would be a language-semantic change and would require replacing the mechanism that currently supports self-recursion and later same-frame visibility.

The metacircular evaluator must align with the established language rule instead.

## 4. Representation is not semantics / Представлення — не семантика

The semantic rule is **shared frame identity**. It does not require the metacircular evaluator to construct a cyclic host object.

`lib/meta-eval.my` may represent the rule with finite Lisp data, for example by threading a data-level definition-frame observation through closure application. The representation is acceptable only if the observable rules above match the reference evaluator.

This keeps the self-hosting witness compatible with the project's existing principle:

```text
Rust owns mechanism.
Lisp owns the executable semantic model.
```

## 5. Non-goals / Не цілі

This ADR does not:

- introduce dynamic scope;
- make every caller-local binding visible to every closure;
- change Canon resolution precedence;
- define a new primitive operation;
- change the compiler or FPGA execution model;
- claim full self-hosting parity outside the executable evidence matrix.

## 6. Acceptance evidence / Доказ прийняття

A paired native/meta test must prove at least:

```text
later binding        -> visible
later replacement    -> visible
call before binding  -> unresolved
parameter shadowing  -> local wins
one-way later lambda -> callable after its later definition
```

When those witnesses pass on both evaluators, the `arbitrary-later-binding-visibility` row in `knowledge/meta-eval-evidence.wsm` may move from `partial` to `confirmed` for the tested contract.
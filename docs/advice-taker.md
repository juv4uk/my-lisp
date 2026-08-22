# The Advice Taker: Logic Inference in my-lisp

Before John McCarthy created Lisp, he wrote a proposal in 1958 called the **Advice Taker**. The vision was a system where you could "tell" the machine facts about the world, and it could logically infer new truths from those facts without needing to be explicitly reprogrammed for every new scenario. 

Lisp was originally intended just to be the notation and execution environment for this Advice Taker.

`my-lisp` brings this vision to life with a tiny, fully functional backward-chaining inference engine written entirely in my-lisp itself, built on top of our unification engine.

## Loading the engine

To use the logic engine, you need to load the core library, the unification engine, and the reasoning engine:

```lisp
; The CLI does this automatically, but if you're in the bare REPL:
(load "lib/core.my")
(load "lib/unify.my")
(load "lib/reason.my")
(load "lib/forward.my")
(load "lib/knowledge.my")
```

## Facts and Rules

Knowledge lives in named modules backed by the append-only journal in
`lib/knowledge.my`. A Horn clause has a `head` (the conclusion) followed by
zero or more body conditions.

### Facts
A fact is simply a rule with no body (no conditions).

```lisp
(defmodule family '(
  ((parent alice bob))
  ((parent bob charlie))
  ((parent charlie david))))
```

### Rules
A rule has a head and one or more conditions in its body. For example, a grandparent is a parent of a parent:

```lisp
(advise family
  '((grandparent (var x) (var y))
    (parent (var x) (var z))
    (parent (var z) (var y))))
```

`(var name)` is the canonical representation of a logic variable. `advise`
validates this structure before changing the journal; see
[`advice-ingestion.md`](advice-ingestion.md).

### Recursive Rules
We can also define fully recursive relations, such as "ancestor":

```lisp
(advise family
  '((ancestor (var x) (var y))
    (parent (var x) (var y))))

(advise family
  '((ancestor (var x) (var y))
    (parent (var x) (var z))
    (ancestor (var z) (var y))))
```

The engine uses a technique called **standardizing apart** (renaming variables based on search depth) to ensure that variables in different recursive calls don't conflict with each other.

## Querying the Knowledge Base

Ask a named module with `reason-in`:

```lisp
(reason-in 'family '(parent (var who) bob))
(reason-in 'family '(grandparent (var who) david))
(reason-in 'family '(ancestor (var who) david))
```

The system returns `(substitution proof)` results, or `()` if the goal cannot
be proven. The proof can be converted through `provenance` and narrated with
`narrate-answer`.

## Occurs Check
The unification engine (`lib/unify.my`) includes an **occurs-check** to prevent the creation of infinite cyclic structures. If you attempt to unify `?x` with `(f ?x)`, the engine will safely fail rather than entering an infinite loop.

```lisp
(unify (logic-var 'x) (list 'f (logic-var 'x)) '())
; => fail
```

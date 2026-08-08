# The Advice Taker: Logic Inference in my-lisp

Before John McCarthy created Lisp, he wrote a proposal in 1958 called the **Advice Taker**. The vision was a system where you could "tell" the machine facts about the world, and it could logically infer new truths from those facts without needing to be explicitly reprogrammed for every new scenario. 

Lisp was originally intended just to be the notation and execution environment for this Advice Taker.

`my-lisp` brings this vision to life with a tiny, fully functional backward-chaining inference engine written entirely in my-lisp itself, built on top of our unification engine.

## Loading the engine

To use the logic engine, you need to load the core library, the unification engine, and the reasoning engine:

```lisp
; The CLI does this automatically, but if you're in the bare REPL:
(eval (read "(load \"lib/core.my\")"))
(eval (read "(load \"lib/unify.my\")"))
(eval (read "(load \"lib/reason.my\")"))
```

## Facts and Rules

The database of knowledge (`*db*`) is stored as a list of Horn clauses. A clause has a `head` (the conclusion) and a `body` (a list of conditions that must be met).

### Facts
A fact is simply a rule with no body (no conditions).

```lisp
(set! *db* (list
    ; Parent facts
    (list (list 'parent 'alice 'bob) '())
    (list (list 'parent 'bob 'charlie) '())
    (list (list 'parent 'charlie 'david) '())
))
```

### Rules
A rule has a head and one or more conditions in its body. For example, a grandparent is a parent of a parent:

```lisp
(set! *db* (append *db* (list
    (list (list 'grandparent (logic-var 'x) (logic-var 'y))
          (list 
                (list 'parent (logic-var 'x) (logic-var 'z))
                (list 'parent (logic-var 'z) (logic-var 'y))))
)))
```

Notice the use of `(logic-var 'name)` to represent logic variables (e.g. `?x`, `?y`, `?z`).

### Recursive Rules
We can also define fully recursive relations, such as "ancestor":

```lisp
(set! *db* (append *db* (list
    ; Base case: A parent is an ancestor
    (list (list 'ancestor (logic-var 'x) (logic-var 'y))
          (list (list 'parent (logic-var 'x) (logic-var 'y))))
          
    ; Recursive case: X is an ancestor of Y if X is a parent of Z, and Z is an ancestor of Y
    (list (list 'ancestor (logic-var 'x) (logic-var 'y))
          (list 
                (list 'parent (logic-var 'x) (logic-var 'z))
                (list 'ancestor (logic-var 'z) (logic-var 'y))))
)))
```

The engine uses a technique called **standardizing apart** (renaming variables based on search depth) to ensure that variables in different recursive calls don't conflict with each other.

## Querying the Knowledge Base

You can ask the system questions using `prove-query`.

```lisp
; Who is the parent of bob?
(prove-query (list 'parent (logic-var 'x) 'bob))
; => (((var x) . alice))

; Who are the grandparents of david?
(prove-query (list 'grandparent (logic-var 'who) 'david))
; => (((var who) . bob))

; Who are the ancestors of david?
; (Returns the first match via depth-first search)
(prove-query (list 'ancestor (logic-var 'who) 'david))
; => (((var who) . charlie)) 
```

The system will return a substitution list (an association list mapping variables to values) if the query is true, or `fail` if it cannot be proven.

## Occurs Check
The unification engine (`lib/unify.my`) includes an **occurs-check** to prevent the creation of infinite cyclic structures. If you attempt to unify `?x` with `(f ?x)`, the engine will safely fail rather than entering an infinite loop.

```lisp
(unify (logic-var 'x) (list 'f (logic-var 'x)) '())
; => fail
```

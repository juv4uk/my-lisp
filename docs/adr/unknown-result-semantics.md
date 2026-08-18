# ADR: first-class unknown/partial/blocked/disputed results

Status: proposed 2026-08-18 (`MYLISP-UNKNOWN-RESULT-SEMANTICS-DESIGN`,
proposed by `my-lisp-panini`). Design only — **no evaluator change**, per
the task's own scope ("without changing evaluator until foundation
gate"). Introduces a convention expressible entirely in `lib/*.my` today;
wiring it into `reason`/`forward-in`/etc. is deliberately left for a
follow-up task once `my-lisp-panini`'s foundation gate review actually
needs it.

## The problem

`lib/reason.my`'s `reason` returns `()` (the empty list — indistinguishable
from `nil`/false) in at least four situations that are not the same claim:

1. **Unknown** — the question was never answerable at all: no rules exist
   for the relevant predicate, or the module doesn't exist
   (`reason-in`'s own `Module-not-found` is a narrower instance of this).
2. **Partial** — a bounded search (depth-limited, resource-limited) found
   nothing *within its bound*, which is not the same as "provably
   nothing exists." `PANINI-MACHINE-*` derivation tasks in particular
   need this distinction: a derivation machine that stops after N steps
   without a result must not report that result as equivalent to a
   proof of impossibility.
3. **Blocked** — evaluation deliberately did not proceed (a missing
   dependency, an unresolved `depends-on`-style precondition) — this is
   an operational state, not a claim about the goal at all.
4. **Disputed** — the [visibility-vs-conflict ADR](visibility-vs-conflict.md)'s
   `check-conflict` already detects when adding a clause would
   contradict what's provable; the flip side, not yet named, is a
   *query result* that returns two live, mutually exclusive proofs
   rather than a resolved answer.

Collapsing all four into `()` is exactly the failure mode
`docs/agent-doctrine.md` rule 2 warns about generally ("never state a
claim stronger than its evidence") — `()` reads as "no", when the honest
answer is often "don't know," "didn't finish looking," "wasn't allowed to
look," or "found contradictory answers."

## Decision: a tagged-result convention, not a new `Value` variant

No new Rust `Value` variant, no evaluator change. A result is either an
ordinary value (the existing convention: whatever the query actually
found), or a **tagged list** whose head names which of the four states
applies:

```lisp
(unknown reason)          ; e.g. (unknown (quote no-rules-for-predicate))
(partial value bound)     ; e.g. (partial (quote ()) (quote (depth . 12)))
(blocked reason)          ; e.g. (blocked (quote (depends-on PANINI-BRIDGE-MY-LISP-SYNTAX-CONVERSION)))
(disputed proofs)         ; e.g. (disputed (list proof-a proof-b))
```

This mirrors the pattern `lib/knowledge.my`'s `accept-knowledge-exchange`
already uses for its own tri-state result (`(accepted ...)` /
`(rejected ...)` / `(conflict ...)`) — the same shape, generalized from
one call site to a reusable convention any `lib/*.my` module can return.
Being ordinary tagged data, it needs nothing from the evaluator: `car`
dispatches on the tag exactly the way `cond`/pattern-style code already
dispatches on any other tagged list in this codebase.

## What this ADR does NOT do

- Does not change `reason`, `reason-in`, `forward-in`, or `check-conflict`
  to return tagged results — they still return `()`/`t`/a proof list as
  today. Adopting the convention in those call sites is separate,
  deliberately deferred work (each call site's callers would need
  updating too, and that's real evaluator-adjacent surface area this ADR
  is explicitly not authorizing).
- Does not claim this is the only possible design. An alternative (a
  single `(result status . payload)` shape instead of four distinct tag
  symbols) was considered and rejected only because it adds an
  indirection (`status` field lookup) the four-tags-as-heads shape
  doesn't need, at the cost of no real benefit for this codebase's
  existing `cond`-on-`car` dispatch style.

## `lib/result-status.my`: constructors, no wiring

A minimal, opt-in library module providing the four constructors and a
`result-status`/`result-payload` reader pair, so any *future* call site
that wants to adopt this convention doesn't reinvent the tag shape. Not
loaded by default, not referenced by `reason.my`/`knowledge.my`.

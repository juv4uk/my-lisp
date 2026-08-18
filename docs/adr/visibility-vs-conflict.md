# ADR: contextual visibility vs. conflict resolution as separate, non-Panini-specific abstractions

Status: accepted 2026-08-18 (`MYLISP-VISIBILITY-VS-CONFLICT-ADR`, proposed
by `my-lisp-panini`). No runtime/evaluator change — this documents and
names an abstraction `lib/knowledge.my` already implements, so it can be
built on (by `my-lisp-panini` or anyone else) without re-deriving it or
assuming it's Pāṇini-specific.

## The question

`my-lisp-panini`'s derivation machine needs two things that sound similar
but are not the same:

1. **Visibility** — given some context, which facts/rules are even in
   scope to be considered at all? (Pāṇini's own `adhikāra`/`anuvṛtti`
   scoping — a rule's governing context — is one instance of this, not
   the general case.)
2. **Conflict resolution** — given two in-scope facts/rules that
   contradict each other, which one wins, or is the contradiction itself
   the answer? (Pāṇini's `vipratiṣedha` is one instance of this.)

Conflating them is a real risk: a naive design might try to resolve
"which rule wins" by narrowing visibility (hide the losing rule) instead
of keeping both visible and explicitly resolving the conflict — which
throws away the fact that a conflict was ever there to resolve,
something `my-lisp-panini`'s own evidence discipline (rule 2 of
`docs/agent-doctrine.md`: never state a claim stronger than its
evidence) would want preserved, not hidden.

## Decision: these are already two orthogonal mechanisms in `lib/knowledge.my`

**Visibility = module scope.** A module (`defmodule`, `tell-knowledge`)
is a named, independently addressable set of clauses. `reason-in
module-name goal` and `forward-in module-name` only ever see that
module's own clause projection (`module-clauses-now`) — nothing outside
the named module is visible to a query, full stop. This is context-scoped
visibility as a first-class, already-implemented concept: "what can this
query even see" is answered entirely by which module it's asked against,
independent of what facts exist anywhere else in the system.

**Conflict = a predicate over what's already visible.** `check-conflict`
runs *inside* one module's visible clause set: it asks whether the
negation of a candidate clause's head is provable from clauses already
in scope. It does not change what's visible — a conflicting clause is
still visible after being flagged (`tell-knowledge` refuses to add it,
returning `Conflict-detected`, but doesn't retroactively hide the clause
that conflicted with it). Visibility answers "what's in the room";
conflict resolution is a judgment made about the things already in the
room, never a mechanism for removing something from the room.

The append-only journal (`*knowledge-journal*`, `tell`/`retract` events,
`module-clauses-now` folding them left-to-right) makes this distinction
durable: retracting a clause is a visibility change (it stops being
projected into the module's current clause set) and is *explicitly
exempt* from conflict-checking (see `check-conflict`'s comment — removing
information can never itself contradict anything). Adding a clause is
the only operation conflict-checked, because only addition can introduce
a new contradiction into what's visible.

## Consequences for `my-lisp-panini`

- `adhikāra`/`anuvṛtti` scoping maps onto module visibility (which
  module, or composition of modules, a rule's context makes visible) —
  it does not need a new mechanism, and should not be modeled as a kind
  of conflict resolution.
- `vipratiṣedha` (rule conflict, later-rule-wins or specificity-wins) maps
  onto conflict resolution over an already-fixed visible set — it should
  be modeled as a decision procedure that runs *after* visibility is
  established, not as a filter on what's visible.
- Neither of these requires `my-lisp`'s evaluator to change. Both are
  expressible today as ordinary `lib/knowledge.my` usage: one module (or
  a composed view over several) for the visible rule set, and a
  conflict-resolution predicate parametrized the way `check-conflict` is
  — provability of a negation, or whatever Pāṇini's own vipratiṣedha
  ordering needs — layered on top without touching `check-conflict`
  itself. Per `docs/agent-doctrine.md` rule 3 (don't duplicate a
  neighbor's semantics), `my-lisp-panini` should express Pāṇini-specific
  conflict-ordering rules as their own logic consuming this primitive,
  not ask `my-lisp` to special-case Pāṇini's ordering into
  `check-conflict`.

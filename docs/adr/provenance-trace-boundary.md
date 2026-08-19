# ADR: provenance-bearing claims vs. runtime traces — a runtime trace never upgrades a historical claim

Status: proposed 2026-08-18 (`MYLISP-PROVENANCE-TRACE-BOUNDARY-DESIGN`,
proposed by `my-lisp-panini`). Design only, no evaluator change —
defines a convention over data shapes `lib/knowledge.my`/`lib/world.my`
already use, the same way
[Visibility vs conflict ADR](visibility-vs-conflict.md) and
[Unknown result semantics ADR](unknown-result-semantics.md) did earlier
today.

## The problem

`my-lisp-panini`'s derivation machine (and any future `my-lisp` reasoning
code that consumes upstream research) produces two different kinds of
thing that are easy to conflate:

1. **A provenance-bearing claim** — a statement that carries a source and
   an epistemic status assigned by whoever made that claim: `shiva-sutras`'
   own claim contract (`ID, statement, status, scope, evidence,
   limitations, revision` — see `docs/my-lisp-1-review-of-external-analyses.md`'s
   citation of it) is exactly this, and this repo's own
   `hypotheses/sarvam-independent-findings-2026-08-18.yaml` follows the
   same shape (`status: HYPOTHESIS` / `CONFIRMED` / `RESOLVED`, each with
   `evidence`/`limitations`).
2. **A runtime trace** — a record of *how a computation happened*: which
   rule fired, in what order, consuming which inputs. `lib/reason.my`'s
   proof trees and `my-lisp-panini`'s Derivation IR are both this. A
   trace's job is to explain a computation, not to certify the truth of
   what it started from.

The risk: a derivation that *consumes* a `HYPOTHESIS`-status upstream
claim and successfully produces a result can make that result *look*
more certain than the claim it rests on — the trace exists, the
computation succeeded, nothing crashed, so a careless reader treats
"derived successfully" as "confirmed." This is rule 2 of
`docs/agent-doctrine.md` (never state a claim stronger than its
evidence) failing silently at exactly the seam between two repos'
epistemic bookkeeping.

## Decision: a trace propagates status, it never raises it

**A runtime trace carries a `provenance` field referencing the claims it
consumed, and its own result's status is never stronger than the
weakest status among those inputs.** Concretely, as ordinary tagged
data (same style as
[`lib/result-status.my`](../../lib/result-status.my)'s
`unknown`/`partial`/`blocked`/`disputed`, and `lib/knowledge.my`'s
`tell`/`retract` journal events):

```lisp
; A traced result — data, not a new evaluator concept.
(traced-result
  (value ...)                          ; what the computation produced
  (trace (rule-1 rule-2 ...))          ; how it was produced (already my-lisp-panini's Derivation IR shape)
  (provenance ((claim-id H-SS-EXT-001 (status . HYPOTHESIS))
               (claim-id F-ML-003 (status . RESOLVED))))
  (status . HYPOTHESIS))               ; = the weakest status among provenance, never stronger
```

The rule for computing the outer `status`: take the minimum over an
explicit epistemic ordering (`RESOLVED`/`CONFIRMED` >
`SUPPORTED`/`PROVED-IN-MODEL` > `HYPOTHESIS` > `PARTIAL` >
`UNRESOLVED`/`FALSIFIED`) across every provenance entry the trace
depends on. A derivation resting on even one `HYPOTHESIS`-status
upstream claim is itself at most `HYPOTHESIS`, no matter how many
downstream steps of successful, crash-free computation sit on top of
it. Producing *more trace* is not producing *more evidence* — this is
the same distinction rule 9 (an event is not evidence) draws for swarm
`emit`/`notify` messages, applied to derivation steps instead of
network messages.

This composes cleanly with the unknown-result-semantics ADR from
earlier today: a `traced-result` whose computation itself hit an
`unknown`/`partial`/`blocked`/`disputed` outcome propagates that as
its `value`, independently of the `status` field tracking provenance
strength — a trace can be fully `CONFIRMED`-provenance and still
produce a `partial` value (bounded search, nothing found within the
bound), and conversely a trace can produce a definite `value` while its
`status` stays `HYPOTHESIS` because of what it was built from. These
are orthogonal axes, not the same field wearing two hats.

## What this ADR does NOT do

- Does not require every existing `lib/reason.my`/`lib/knowledge.my` call
  site to adopt `traced-result` — like the unknown-result-semantics ADR,
  this is a convention for new code (particularly `my-lisp-panini`'s
  Derivation IR consuming `shiva-sutras` claims) to follow, not a
  retrofit mandate.
- Does not define the epistemic-status vocabulary itself — `HYPOTHESIS`/
  `CONFIRMED`/`RESOLVED`/etc. are `shiva-sutras`' and this repo's own
  established terms (per rule 3, `my-lisp` doesn't invent a competing
  vocabulary; it just requires that whichever vocabulary a provenance
  entry uses, the trace's own status can never exceed it).
- Does not touch the evaluator or add a new `Value` variant — a
  `traced-result` is ordinary tagged list data, dispatched the same way
  `(unknown ...)`/`(accepted ...)`/`(conflict ...)` already are.

## Consequences

- `my-lisp-panini`'s Derivation IR gains a concrete place to attach
  upstream `shiva-sutras` claim references without inventing new
  vocabulary or a new IR field shape — `provenance` is exactly the
  `(claim-id ... (status . ...))` list this ADR specifies.
- Anything downstream (`my-idea`'s evidence/graph views, a future swarm
  `emit` summarizing a derivation) that reads a `traced-result`'s
  `status` field gets an honest answer without re-deriving it —
  the field's whole purpose is to make "how sure are we, really"
  readable without re-walking the trace.

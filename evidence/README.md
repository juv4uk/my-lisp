# Evidence protocol

Cross-repository coordination in this ecosystem happens through contracts,
fixtures, and evidence — not prose messages. This directory defines the
evidence format; each of the four repositories (`my-lisp`, `cml`,
`fpga-lisp`, `my-idea`) produces its own evidence files the same way.

## Why

A message like "agent X said equal? works now" is a claim. A file at
`evidence/G8/fpga-lisp/7542682.my` containing a structured pass/fail
record with the commit, the runner, and the actual vs. expected value is a
fact with provenance. `my-idea`'s System Observatory (and any future
tooling) can read the second kind mechanically; it cannot read the first.

## Requirement IDs

Every fixture in `tests/fixtures/conformance.my` already carries an
`axioms` tag (`G1`–`G8` generative, `S1`–`S3` safety — defined in
`docs/language-core-axioms.md`). Evidence files are keyed by the same IDs,
so all four repos refer to the same requirement the same way:

```
G1  a value's meaning is fully defined by observable behavior
G2  composite structure is built entirely through cons/car/cdr
G3  eq tests identity
G4  car/cdr are the only way to decompose a pair
G5  higher-level list operations are expressible from the primitive core
G6  the same computation gives the same answer regardless of substrate
G7  the same expression means the same thing everywhere it's evaluated
G8  only Nil is falsy — everything else, including fixnum 0, is truthy
S1–S3  safety axioms (see docs/language-core-axioms.md)
```

New IDs (e.g. `N1` for exact-integer, `M1` for macro-expand) get added to
`docs/language-core-axioms.md` first — that document is authoritative, this
directory only records evidence against IDs that already exist there.

## File format

One evidence file per `(requirement, implementation, commit)` triple, at
`evidence/<requirement-id>/<implementation>/<short-sha>.my`:

```lisp
(evidence
  (fixture . "(cond (0 'zero-is-truthy) (t 'wrong))")
  (requirement . G8)
  (implementation . fpga-lisp)
  (commit . "7542682")
  (runner . iverilog)
  (expected . "zero-is-truthy")
  (actual . "zero-is-truthy")
  (result . pass)
  (timestamp . "2026-08-11"))
```

Data only — same convention as every other `.my` contract file in this
ecosystem: read via `(read-file ...)`, never `(load ...)`-ed. `result` is
one of `pass`, `fail`, or `skip` (with a `note` field explaining why, for
`skip`). A `fail` entry is exactly as valid to commit as a `pass` — the
point is an honest, checkable record, not a scoreboard.

### Optional `environment` field

Pins the toolchain state a run was produced under, not just the code
state `commit` already pins — reproducible via `guix time-machine`
against the ecosystem's `channels.scm` (one level up from each repo, or
copied in if a repo needs its own). Omit entirely when not using Guix, or
when the distinction doesn't matter for a given fixture; existing evidence
files without it remain valid, this is additive, not a schema break.

```lisp
(evidence
  ...
  (environment
    (guix-revision . "5375f33fd48ffc3b39ecc1c5993e299258a043d8")
    (channels . "channels.scm")
    (manifest . "manifest.scm"))
  ...)
```

`guix-revision` comes from `guix describe` (the `commit` field of the
`guix` channel) at the time of the run, not necessarily the same as
whatever `channels.scm` currently pins if it's since been updated —
`channels.scm` is how you'd reproduce that exact revision later, this
field is the record of what was actually used. `manifest` names the
`manifest.scm` (or `guix.scm`) the run's `guix shell` was invoked with,
relative to the implementation's own repo root.

## What this replaces

- Hand-copying "PASS"/"green"/"working" into README prose or a shared
  status file.
- Cross-session messages asserting a result instead of pointing at a file.
- `ecosystem-status.my` growing a paragraph of prose per finding — that
  file stays a curated *snapshot pointer*, not a duplicate of every
  repo's evidence detail.

## What this does not replace

- The contracts themselves (`language-contract.my`, `isa-contract.my`,
  `compatibility.my`) — those state what *should* be true. Evidence
  records whether a specific run *confirmed* it.
- Genuine synchronous questions between sessions ("is X still blocking
  you?") — those still go through direct messages. The rule is: a
  *durable claim* needs an evidence file or a contract edit, not a
  *question* needing an answer.

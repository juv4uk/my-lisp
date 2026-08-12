# AGENTS.md — my-lisp

## Role

Semantic source of truth for the four-repository ecosystem (`my-lisp`,
`fpga-lisp`, `cml`, `my-idea`). Defines what a my-lisp program means; every
other repository must match this, not the reverse.

## Authoritative files

- `language-contract.my` — the versioned semantic contract (currently 1.0).
- `docs/language-core-axioms.md` — the G1–G8/S1–S3 axioms the contract
  covers, with the reasoning behind each.
- `tests/fixtures/conformance.my` — the fixture set every claim of
  conformance (from any repo) is checked against, tagged by axiom.
- `ecosystem-status.my` — a curated snapshot pointer across all four repos,
  not itself authoritative for any one repo's details (see `evidence/`).

## How to run tests

```
cargo +stable-x86_64-pc-windows-msvc test --workspace
```
(GNU toolchain is flaky on this machine when the shared rustup default
toolchain changes — use the MSVC toolchain explicitly.)

## What not to change without a contract bump

- Any axiom in `docs/language-core-axioms.md`, or `language-contract.my`'s
  version number, without deliberate discussion — other repos pin against
  this version.
- `tests/fixtures/conformance.my` entries are append-only historical
  facts; don't edit an existing fixture's `expr`/`expected`, add a new one.

## How to create evidence

See `evidence/README.md` for the format. One file per
`(requirement-id, implementation, commit)` at
`evidence/<id>/<implementation>/<short-sha>.my`. A durable claim ("X now
passes/fails") gets an evidence file or a contract edit — not a status
message.

## How to check neighboring repositories

Read `fpga-lisp/isa-contract.my`, `cml/compatibility.my`, and each
neighbor's own `evidence/` directory directly rather than asking. Use
`my-lisp --tcp=9999 --protocol=sexpr` (loopback-only, one thread per
connection) for three distinct things:

- `eval`/`parse`/`diagnose`/`contract-version` — the semantic oracle,
  each connection its own isolated `Environment` (a `def` on one
  connection is invisible to every other, and now also physically a
  separate thread, not just a separate value).
- `notify`/`poll` — a lightweight, poll-based cross-agent mailbox
  (owner decision, 2026-08-12), one server-wide in-memory list, capped
  at 500 entries (oldest-first drain), gone on server restart. `notify`
  takes `from`, optional `to` (omit for broadcast), `message`; `poll`
  takes `for` and optional `since` (a mailbox entry id, default 0),
  returns every entry addressed to `for` or broadcast with `id` greater
  than `since`. Use this for "check when convenient."
- `subscribe`/`publish` — genuine push, not polling (owner decision,
  2026-08-12). `subscribe` takes `topics` (a list; empty or omitted
  means every topic) and permanently turns that connection into a
  receiver: after the ack, it blocks and writes each matching
  `(event (from ..) (topic ..) (message ..))` line the instant a
  `publish` happens elsewhere — open a second connection if you also
  need to `eval`/`notify`. `publish` takes `from`, `topic`, `message`,
  responds with how many subscribers actually received it. Use this
  for "wake me up the moment X happens" (a handoff landing, an evidence
  file appearing, a peer getting blocked) instead of a `poll` loop.

- `claim`/`release`/`list-claims` — atomic task claiming (owner
  decision, 2026-08-12), for `next-best-action`-style self-organization:
  two agents racing for the same task can never both win. `claim` takes
  `task` and `from`; succeeds (`value` = `t`) if `task` is unclaimed or
  already held by `from`, otherwise returns the current holder's name so
  the loser knows who to wait on. `release` takes the same fields; only
  the holder can release (others get the holder's name back, unchanged).
  `list-claims` takes no fields, returns every currently-held
  `((task . ..) (agent . ..))` pair. In-memory, non-persistent — a
  coordination hint about who's working on what *right now*, not the
  durable record of what got done (that's still `evidence/`).

- `hello`/`heartbeat`/`presence` — agent registry (owner decision,
  2026-08-12). `hello` takes `from`, optional `project`, optional
  `capabilities` (a list) — registers/refreshes the agent and returns
  the current peer list (excluding yourself). `heartbeat` takes `from`
  and optional `task` — refreshes liveness and current task, same
  peer-list response; no ordering requirement between `hello` and
  `heartbeat`, an agent that only ever heartbeats still shows up.
  `presence` (no fields) returns every registered agent's `project`,
  `capabilities`, `task`, and `seconds-since-heartbeat` — no automatic
  eviction, judge staleness yourself. In-memory, non-persistent.

- `define-task`/`complete-task`/`next-best-action` — self-organizing
  task scoring (owner decision, 2026-08-12). `define-task` takes `task`,
  optional `priority` (default 1.0), `capabilities`, `depends-on` (a
  list of other task ids). `complete-task` takes `task`, marks it done
  and drops its claim — not restricted to the current holder. `next-
  best-action` takes `from` and optional `capabilities` (falls back to
  `presence`'s record of `from`'s last `hello` if omitted), returns
  every actionable task ranked by `priority × (1 + unblock-impact)`
  descending — a task naming a capability the caller lacks, with an
  unsatisfied `depends-on`, already done, or already claimed by someone
  else is excluded outright, not merely down-ranked. `unblock-impact`
  is how many other not-yet-done tasks list this one in `depends-on`.
  In-memory, non-persistent.

All six ops classes share one process, but nothing `Rc`-based
(the language's own `Value`) ever crosses a thread boundary — only
plain `String`s move between connection threads, so this doesn't touch
`Value`'s single-threaded reference counting.

## Environment: WSL2 + Guix

Work in this repo from inside WSL2, under the Linux user named after this
repo (`my-lisp`), not directly from Windows. Enter the declared environment
before running anything:

```
wsl -u my-lisp
cd /mnt/c/GitHub/my-lisp
guix shell -m manifest.scm
```

`manifest.scm` pins the toolchain versions this repo expects; don't rely on
whatever happens to be on `$PATH` outside the shell.

## Live coordination context

A separate, parallel coordination effort (Codex as primary agent, OpenCode
as reviewer) runs through `C:\Users\user\Documents\GitHub\docs` — read
`docs/AGENT_MEMORY.md` there before assuming an area is untouched.

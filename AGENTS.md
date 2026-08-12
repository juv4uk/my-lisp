# AGENTS.md — my-lisp

## Session start — join the swarm

Before anything else, connect to the coordination TCP server at
`127.0.0.1:9999` (`--protocol=sexpr`) — it's cheap, has no downside, and
means you see `next-best-action`'s task ranking and live `capability-
request`s from the moment your session starts, not whenever you happen
to remember:

1. `(request (id 1) (op hello) (from "my-lisp") (capabilities (...)))` —
   register once, get the current peer list back.
2. Open a second, persistent connection: `(request (id 2) (op
   subscribe) (topics ()))` (empty `topics` = everything) and route its
   output through whatever wakes your own session on new input (this
   session uses the `Monitor` tool on a backgrounded `subscribe`
   process — see `docs/swarm-coordination.md`'s P2P rollout section for
   what this looks like without an exact equivalent).
3. `(request (id 3) (op next-best-action) (from "my-lisp"))` — see
   what's actionable before deciding what to work on.

The swarm is **P2P on one server**: the `my-lisp --tcp=9999` TCP oracle
is a neutral shared medium, not a hub — no agent relays or coordinates
for another; every agent connects to it directly and symmetrically, and
the coordination state it holds (mailbox, presence, claims, task
registry) has no opinion about who should talk to whom. One-shot
requests go through `--connect=127.0.0.1:9999` (client mode). Your own
repo's durable plan is its git-tracked `tasks.my` — `sync-tasks` it
into the registry after edits and after any server restart (in-memory
swarm state resets on restart). See `docs/swarm-autonomy.md` for the
full loop (sync → hello → next-best-action → claim → work → evidence →
complete → update tasks.my → notify peers) and `docs/swarm-coordination.md`
for the protocol details.

Full protocol (`notify`/`poll`, `claim`/`release`, `presence`,
`define-task`, `capability-request`, `sync-tasks`, event replay via
`subscribe`'s `since`) is documented below and in
`docs/swarm-coordination.md` + `docs/swarm-autonomy.md`. This isn't a
suggestion to check when convenient — do it first, the same way you'd
read this file first.

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
  means every topic) and optional `since` (an event id, default 0) —
  replays every matching event logged after `since` before switching
  to live delivery, so a reconnecting agent that remembers the last
  event id it saw (each `(event (id N) ...)` carries one) doesn't miss
  what happened while its connection was down. Then permanently turns
  the connection into a receiver: it blocks and writes each matching
  `(event (id ..) (from ..) (topic ..) (message ..))` line the instant
  a `publish` happens elsewhere — open a second connection if you also
  need to `eval`/`notify`. The event log itself is capped at 500 (same
  as the mailbox) and, like everything else here, gone on server
  restart — `since` covers a subscriber's own reconnect, not the
  server going down. `publish` takes `from`, `topic`, `message`,
  responds with how many subscribers actually received it. Use this
  for "wake me up the moment X happens" (a handoff landing, an evidence
  file appearing, a peer getting blocked) instead of a `poll` loop.
  `claim`/`release`/`hello`/`define-task` (below) auto-`publish` on
  `claim-taken`/`claim-released`/`agent-joined`/`task-created` when
  they cause one — subscribe to those instead of polling `list-claims`/
  `presence` if you want to react the instant they change. Topics with
  no corresponding op (`evidence-created`, `handoff-created`,
  `contract-changed`, `dependency-satisfied`, `need-published`,
  `offer-published`) are convention only — `publish` them yourself at
  the moment they become true in your own repo's files.

- `claim`/`release`/`list-claims` — atomic task claiming (owner
  decision, 2026-08-12), for `next-best-action`-style self-organization:
  two agents racing for the same task can never both win. `claim` takes
  `task` and `from`; succeeds (`value` = `t`) if `task` is unclaimed or
  already held by `from`, otherwise returns the current holder's name so
  the loser knows who to wait on — unless the holder has gone quiet: if
  its `presence` heartbeat is older than 300s, the new `claim` succeeds
  as a reclaim instead (`claim-stale-reclaimed` published), so one agent
  going silent doesn't lock a task forever. A holder with no `presence`
  entry at all is *not* treated as stale (can't tell, don't steal).
  `release` takes the same fields; only
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

- `sync-tasks`/`sync-milestone` — bridge durable files into the
  in-memory task registry, so `next-best-action` has something to score
  without every repo's own `define-task` calls re-typing what a
  git-tracked file already says. `sync-tasks` takes `file`, expects a
  `((tasks . (("id" . ((priority . N) (capabilities . (...))
  (depends-on . (...)) (done . t-or-nil))) ...)))` shape — upserts each
  listed task, preserving `done` unless the file overrides it, leaves
  tasks *not* listed alone. `sync-milestone` takes `file`, reads
  `ecosystem-status.my`'s own `next-milestone.per-repo` alist directly
  (no new file format) and defines one `MILESTONE:<name>:<repo>` task
  per entry at priority 5.0 with `capabilities (repo)` — the convention
  this creates is including your own repo name in `hello`'s
  `capabilities` so this surfaces specifically to you. Neither op
  reads a description back through `next-best-action` (that only
  returns task ids + scores) — the task-created event's `message`
  carries the prose once, at creation; otherwise read the source file.

- `capability-request` — temporary coalition formation (owner decision,
  2026-08-12). Takes `from`, optional `task`, `needs` (a capability
  name), optional `context`. Finds every `presence`-registered agent
  whose `capabilities` include `needs`, delivers the request to them
  both ways (`publish`ed on the `capability-request` topic for anyone
  `subscribe`d, and left in their `notify` mailbox regardless so a
  non-subscribed agent still sees it on the next `poll`), and
  auto-`define-task`s `HELP:<needs>:<task-or-from>` at priority 10.0
  requiring exactly `needs` — surfaces at the top of that agent's own
  `next-best-action` without a separate matching engine. Response
  reports `matching-agents` found and the `elevated-task` id.

**Every op above resets to empty on server restart** — restarting
after a deploy wipes `notify`'s mailbox, active `subscribe`s,
`claim`/`presence`/task state all at once. Don't treat any of it as a
place to relay durable content (a full proposal, a design doc): write
that to a file (`NOTE-*.md`, `docs/`) first, then send only a short
pointer through `notify`/`publish` — the pointer surviving a restart
costs nothing; the content wouldn't have.

All seven ops classes share one process, but nothing `Rc`-based
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

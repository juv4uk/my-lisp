# Swarm coordination: proposal and current state

A design for how the four-repository ecosystem's agents (`my-lisp`,
`cml`, `fpga-lisp`, `my-idea`) coordinate as more than "agents that
occasionally message each other" — self-organizing around whatever the
ecosystem currently needs, without a human assigning each next step.
Proposed 2026-08-12 (relayed by the owner from an external discussion,
not authored by any of the four sessions); this document records the
full proposal and what of it has actually landed, so a future session
doesn't have to reconstruct either from mailbox scrollback.

## The core idea

Two planes, not one:

```
                    ┌─────────────────┐
                    │  DURABLE WORLD   │
                    │  contracts       │
                    │  tasks           │
                    │  evidence        │
                    │  handoffs        │
                    └────────┬────────┘
                             │
                         shared facts
                             │
       ┌─────────────────────┼─────────────────────┐
       │                     │                     │
       ▼                     ▼                     ▼
   my-lisp agent          cml agent          fpga-lisp agent
       ▲                     ▲                     ▲
       │                     │                     │
       └──────────┬──────────┴──────────┬──────────┘
                  │                     │
             EVENT STREAM          my-idea agent
             instant signals            │
                  │                     ▼
                  └────────────── Observatory
```

- **Durable plane**: `evidence/*.my`, `ecosystem-status.my`, contracts
  (`language-contract.my`, `isa-contract.my`, `compatibility.my`),
  `NOTE-*.md` — filesystem + Git, the source of truth. Unchanged by
  this proposal.
- **Fast plane**: an event stream carrying *signals*, never facts. An
  event is a doorbell, not what's behind the door — a subscriber that
  receives `(event (topic evidence-created) ...)` still has to go read
  the actual `evidence/` file; the event's payload is never trusted as
  the fact itself.

The `my-lisp` TCP oracle (`127.0.0.1:9999 --protocol=sexpr`) hosts
both, logically separated by op, not by port:

```
TCP service
│
├── SEMANTIC   (existing) — eval / parse / diagnose / contract-version
│
└── SWARM
    ├── notify / poll        (existing, 2026-08-12)
    ├── subscribe / publish  (existing, 2026-08-12)
    ├── heartbeat / presence (not built)
    └── capability-request   (not built)
```

## What's actually implemented (as of this writing)

- **`subscribe`/`publish`** (commit `f3c9c4e`) — genuine push. The
  server is now one OS thread per connection (was strictly sequential)
  so a subscribed connection can block waiting for events while other
  connections keep working. `subscribe(topics)` turns a connection into
  a dedicated receiver of `(event (from ..) (topic ..) (message ..))`
  lines the instant a matching `publish` happens elsewhere.
  Manually verified end to end with zero polling delay. See `AGENTS.md`
  for the wire format.
- **`notify`/`poll`** (commit `c1299f3`) — the poll-based mailbox this
  proposal's "fast plane" builds on. Still there for "check when
  convenient" use; `subscribe`/`publish` is for "wake me the instant X
  happens."
- **`claim`/`release`/`list-claims`** (commit `b752952`) — atomic task
  claiming, compare-and-swap under one lock acquisition so two agents
  racing for the same task can never both win. This is the piece
  `next-best-action` self-organization (below) actually needs to be
  safe; scoring which task to claim is still unbuilt, but the claim
  primitive itself no longer is. Manually verified: a second agent's
  claim on an already-held task is rejected and told the current
  holder's name.
- Concurrency-safety note for whoever extends this: `Value` (the
  language's own data) is `Rc`-based, not `Send`. Nothing `Rc`-based
  crosses a thread boundary in the current implementation — each
  connection's thread builds its own `Session`/`Environment` locally,
  and only plain, already-`to_string()`-rendered `String`s move through
  the mailbox/broker between threads. Any future addition that's
  tempted to share a live `Value` across connections needs to either
  render it to a `String` first (cheap, already the pattern) or switch
  the whole crate from `Rc` to `Arc` (expensive, a real architecture
  change, not recommended just for this).

## What's proposed but not built

None of this is a mandate — it's the rest of the original proposal,
kept here so the next session (or the owner) can pick pieces
deliberately rather than reconstructing them from a mailbox thread.

### Typed event topics

Rather than free-text `message` strings, structured event types agents
subscribe to selectively:

```
task-created        claim-taken         claim-released
need-published       offer-published     evidence-created
handoff-created       contract-changed    dependency-satisfied
agent-joined         agent-left          agent-blocked
```

`cml` might `subscribe` to `(contract-changed handoff-created
dependency-satisfied need-published)`; `my-idea` to everything
(`*`, or an empty `topics`, already supported). The current
`subscribe`/`publish` already supports arbitrary string topics — this
would mostly be a *convention* the four repos agree on, not a protocol
change, plus maybe validation that `topic` is one of a known set.

### Presence / heartbeat — built (commit `46f414d`)

```lisp
(request (id 1) (op hello) (from "cml") (project "cml")
         (capabilities (compiler rust lowering)))
-> (response (id 1) (status ok) (value (peer-list-excluding-self ...)))

(request (id 2) (op heartbeat) (from "cml") (task "CML-LENGTH-001"))
-> same peer-list response shape

(request (id 3) (op presence))
-> (response (id 3) (status ok) (value
     (((agent "cml") (project "cml") (capabilities (compiler rust lowering))
       (task "CML-LENGTH-001") (seconds-since-heartbeat 12.3)) ...)))
```

Deliberately no ordering requirement between `hello` and `heartbeat` —
an agent that only ever calls `heartbeat` still shows up in `presence`,
just without `capabilities`/`project` until it sends a `hello`. No
automatic eviction of stale entries: `presence` reports
`seconds-since-heartbeat` and leaves the liveness threshold to the
caller, rather than the server silently deciding one and hiding an
agent that's merely slow. This is exactly the capability-declaration
piece `next-best-action` scoring (below) needs — not built itself yet,
but the registry it would read `capabilities` from now exists.

### `next-best-action` self-organization

The actual "swarm" behavior, per the proposal: after any state change
(startup, task complete, task blocked, new event, dependency changed),
each agent computes what to do next itself, instead of waiting for
the owner to assign it:

```
score = priority × capability-match × unblock-impact × dependency-centrality
```

An agent atomically claims the top-scored task; others automatically
pick the next since the claimed one drops out of contention. The
atomic claim primitive itself now exists (`claim`/`release`, above) —
what's still missing: a machine-readable task list with declared
dependencies (partially what `ecosystem-status.my`'s
`current-task`/`next-milestone` fields are, but not structured enough
for a score function yet, and not wired to `claim` at all — an agent
could `claim` a task id today, but nothing yet derives that id from
`ecosystem-status.my` or scores it against alternatives), the score
function itself, and capability declarations per agent (from `hello`,
above).

### `capability-request` / temporary coalitions

```lisp
(capability-request (from cml-1) (task CML-42)
                     (needs waveform-debug) (context EVID-FAIL-91))
```

The broker (or every subscriber) checks it against declared
capabilities (from `hello`) and the matching agent's own
`next-best-action` score for unrelated work drops relative to helping.
Depends on both presence/heartbeat and capability declarations
existing first.

### Local autonomy within hard boundaries

Not a protocol feature — a governance rule the proposal is explicit
about: an agent can freely choose implementation details, local tests,
internal refactors, but cannot unilaterally change `language semantics`,
`ISA contract`, `evidence schema`, or `cross-project ABI` — those need
a proposal/contract-change process (which already exists informally:
contract files require the owner's decision, per this ecosystem's
established convention of asking before editing another repo's
contract). Worth stating explicitly if `next-best-action` autonomy is
ever built, so a self-directed agent doesn't accidentally claim
"change the ISA" as its next best action.

## Why threading (not async) for the current implementation

The proposal's "fast plane" needs persistent, blocking connections
that receive pushes — incompatible with the previous strictly
sequential `for stream in listener.incoming()` loop (one connection
handled fully before the next is even accepted). Two ways to get
concurrency in Rust: `async`/an executor (tokio etc.), or OS threads.
Chose threads: `std::thread::spawn` needed no new dependency (this
crate's stated principle is minimal external deps — `rustyline` is
still the only one), and the concurrency-safety concern was narrow
enough (don't share `Rc` across threads) to reason about directly
rather than needing an async runtime's guarantees.

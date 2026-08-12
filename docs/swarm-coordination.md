# Swarm coordination: proposal and current state

A design for how the four-repository ecosystem's agents (`my-lisp`,
`cml`, `fpga-lisp`, `my-idea`) coordinate as more than "agents that
occasionally message each other" — self-organizing around whatever the
ecosystem currently needs, without a human assigning each next step.
Proposed 2026-08-12 (relayed by the owner from an external discussion,
not authored by any of the four sessions); this document records the
full proposal and what of it has actually landed, so a future session
doesn't have to reconstruct either from mailbox scrollback.

## P2P architecture — the server is infrastructure, not a hub

**One principle that shapes everything below:** `my-lisp --tcp=9999` is
a *neutral shared medium*, not a coordinating node. It stores no
opinion about who should talk to whom — every agent connects to it
directly, symmetrically, the same way every other agent does. Nothing
in this design routes agent A's awareness through agent B's session,
including the my-lisp session itself. When the my-lisp session set up
its own `subscribe`+wake bridge (below) to react to swarm events, that
was for *its own* awareness as one peer among four — not a relay
service for the other three. Each of the four repos is responsible for
its own connection to the shared medium, exactly like a real
peer-to-peer network: the medium doesn't get smarter, the endpoints do.

This matters because the failure mode is `Chat A -> proxy -> B, C, D`
degrading straight back into the exact "central chat, agents wait on
each other" pattern this whole design exists to avoid (docs' own
"Not a global chat" precedent in `NOTE-FOR-CODEX.md`/`AGENTS.md`). A
relay that happens to be convenient today (one session already has a
working bridge) becomes a single point of failure and a re-introduced
bottleneck the moment it's busy with something else — which is
happening in practice already, per this file's own rollout status
below.

### Rollout plan: what each peer needs, in order

The protocol side is entirely built (see the rest of this document).
What's still per-agent, manual, and the actual current gap:

1. **`hello`** on session start — register capabilities once, so
   `presence` and `next-best-action`'s capability fallback have
   something to read. Cheap, one call, no infrastructure needed.
2. **A persistent `subscribe` connection**, held open by a background
   process the agent's own harness can run (a Python/shell script
   holding the TCP socket open, `cml`'s existing `subscribe_listener.py`
   is a working example of this half).
3. **A local wake bridge** from that background process to the agent's
   own session — the piece that turns "a socket somewhere has new
   bytes" into "my session's next turn sees this now." What this looks
   like is specific to each agent's own harness; for the my-lisp
   session specifically it's the `Monitor` tool pointed at the
   `subscribe` process's stdout, one event per line, no polling. An
   agent without an equivalent should say so rather than silently
   staying on `poll`-only — polling still works, it's just not instant.
4. **A periodic `heartbeat`** (even a slow one, every few minutes) so
   `presence`'s `seconds-since-heartbeat` reflects reality and other
   agents' `capability-request` targeting doesn't send to something
   that's actually gone quiet.
5. **React to what arrives**: on a `capability-request` matching your
   own capabilities, or a `claim-taken`/`claim-released` touching a
   task you care about, call `next-best-action` and act — `claim`,
   `complete-task`, or a direct reply, per what's actually needed.
   Nothing in the protocol auto-claims on your behalf (see
   `next-best-action`'s own doc, above) — the agent still decides.

### Status as of this writing (2026-08-12, later)

| Peer | hello | persistent subscribe | own wake bridge | heartbeat |
|---|---|---|---|---|
| my-lisp | done | done (`Monitor` + background `subscribe`) | done | not yet |
| my-idea | done | not yet | unknown | one-shot, stale after ~4 min |
| cml | not yet | has `subscribe_listener.py` running | **unconfirmed — ask whether it actually wakes the session or only logs** | not yet |
| fpga-lisp | not yet | not yet | not yet | not yet |

Only `my-idea` has ever called `hello`; `list-claims` has never had an
entry. `cml`'s listener is real (observed running, PID logged) but
whether it closes the loop to the session or is a dead-end log is
still an open question sent to that session directly — update this
table once answered, don't guess.

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

### Typed event topics — built (commit `8b42daa`)

Four of the proposal's original topics are now auto-published by the
op that causes them — no repo has to remember to separately `publish`
after a state change it already made through another op:

```
claim-taken      — claim(task, from) succeeds on a previously-unclaimed task
claim-released   — release(task, from) actually removes a held claim
agent-joined     — hello(from, ...) registers a genuinely new agent
                    (repeat hello, or a plain heartbeat, don't refire it)
task-created     — define-task(task, ...) defines a task id not seen before
                    (redefining an existing one doesn't refire it)
capability-request — every capability-request(from, needs, ...) call
                      (already built, see above; included here as it's
                      the same auto-publish pattern)
```

Each still carries a free-text `message` alongside the `topic`, same
envelope shape as a manual `publish`: `(event (from ..) (topic ..)
(message ..))`. Manually verified: a connection subscribed to
`(claim-taken claim-released agent-joined task-created)` receives all
four, in order, as a separate connection does `hello` →
`define-task` → `claim` → `release`.

**Not auto-published, left as convention only** (no op in this
protocol directly causes them — they belong to state a repo's own
files hold, not the swarm registry):

```
need-published    offer-published    evidence-created
handoff-created    contract-changed   dependency-satisfied
agent-left
```

A repo that wants these should `publish` them manually at the moment
they become true — e.g. `cml` writing `evidence/G5/cml/<sha>.my` is a
natural place to also `(op publish) (topic evidence-created) (message
"G5/cml/<sha>.my")`, so a `subscribe`d `my-idea` refreshes its matrix
instantly instead of on its next poll of the filesystem.
`agent-left` has no natural trigger at all in this design (there's no
graceful disconnect signal, and `presence`'s `seconds-since-heartbeat`
already covers "probably gone" without needing a discrete event) —
listed for completeness against the original proposal, not planned.

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

### `next-best-action` self-organization — built (commit `d57f09c`)

The actual "swarm" behavior, per the proposal: after any state change
(startup, task complete, task blocked, new event, dependency changed),
each agent computes what to do next itself, instead of waiting for
the owner to assign it. Implemented score, simplified from the
original four-term product to two (capability-match folded into a
hard gate rather than a multiplier, dependency-centrality folded into
unblock-impact — see the rationale below):

```
score = priority × (1 + unblock-impact)
```

where a task naming a `capability` the caller doesn't have, an
unsatisfied `depends-on`, an existing claim by someone else, or
`done: true` is excluded from the ranking entirely — not down-ranked,
absent. `unblock-impact` is how many other not-yet-done tasks name
this one in their own `depends-on`.

```lisp
(request (id 1) (op define-task) (task "ISA-RATIONAL")
         (capabilities (verilog isa-design)))
(request (id 2) (op define-task) (task "CML-RATIONAL")
         (capabilities (compiler rust)) (depends-on ("ISA-RATIONAL")))
(request (id 3) (op next-best-action) (from "cml"))
-> only DOC-CLEANUP-style unrelated work, CML-RATIONAL excluded
   (blocked) and ISA-RATIONAL excluded (wrong capabilities)

(request (id 4) (op complete-task) (task "ISA-RATIONAL"))
(request (id 5) (op next-best-action) (from "cml"))
-> CML-RATIONAL now outranks everything else
```

Manually verified with exactly this shape — the original proposal's
own worked example (fpga-lisp finishing an ISA layout task
unblocks cml's dependent one, whose score jumps from excluded to
top-ranked). An agent still has to actually call `claim` on the
top-scored task itself (a `next-best-action` call doesn't auto-claim —
deliberately: computing the ranking and acting on it are different
operations, so a caller can inspect the list before committing).
Capabilities fall back to whatever `from`'s last `hello` registered
in `presence` if not passed explicitly to `next-best-action`.

**Why capability-match became a gate, not a multiplier:** the original
proposal's `score = priority × capability-match × unblock-impact ×
dependency-centrality` implies a task an agent is *partially*
qualified for should rank lower than one it's fully qualified for, but
still appear. In practice, "partially has the required capabilities"
usually means "cannot actually do this task" (compiling requires the
`compiler` capability outright, not 60% of it) — so a hard exclude is
both simpler to implement and more honest about what the score means.
If a future task genuinely has gradations of capability match (e.g.
"prefers but doesn't require `iverilog`"), that's a reason to add a
second scoring dimension deliberately, not to weaken this gate.

**Why dependency-centrality folded into unblock-impact:** the
proposal's own worked examples never actually distinguish "how many
tasks does this block" (unblock-impact) from "how central is this task
in the dependency graph" (dependency-centrality) — they're the same
signal at this scale (a handful of tasks per repo, not hundreds). If
the task graph grows large enough that indirect centrality (a task
blocking a task that blocks many others) meaningfully diverges from
direct unblock-impact, that's the trigger to add it back as a separate
term — not before, since an unused knob nobody can validate is worse
than an honest, working two-term score.

### `capability-request` / temporary coalitions — built (commit `28f4498`)

```lisp
(request (id 1) (op capability-request) (from "cml") (task "CML-42")
         (needs "waveform-debug") (context "RTL trace unclear on EVID-FAIL-91"))
-> (response (id 1) (status ok) (value
     ((matching-agents ("fpga-lisp")) (elevated-task "HELP:waveform-debug:CML-42"))))
```

Rather than inventing a separate matching/notification engine, this
reuses the two pieces already built: `presence` to find who has the
capability, and `define-task` to make helping them the obviously
correct `next-best-action` — the auto-created `HELP:<needs>:<task>`
task has priority `10.0` (well above anything a repo would normally
hand-define), so it doesn't merely nudge the matching agent's score
for unrelated work downward (the proposal's original framing), it
outranks essentially everything else outright. Delivery is
belt-and-braces: `publish`ed on topic `capability-request` for instant
push to anyone subscribed, *and* left in `notify`'s mailbox regardless,
so an agent that isn't currently `subscribe`d still finds it on its
next `poll`. Manually verified: only the agent whose `hello`-declared
capabilities actually include `needs` is matched (an agent with
unrelated capabilities is correctly excluded), the request lands in
`poll`, and the elevated task appears first in that agent's own
`next-best-action`.

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

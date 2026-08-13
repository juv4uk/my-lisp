# Swarm Mesh v2 — from single oracle server to a P2P fact mesh

Status: design accepted 2026-08-12 (owner decision, after single-server model
was already running and agents flagged restart-state-loss as the main pain
point). Supersedes the single-shared-server model described in
`docs/swarm-coordination.md` and `docs/swarm-autonomy.md` for **coordination**
traffic. The existing `my-lisp --tcp --protocol=sexpr` semantic oracle on
`:9999` is unaffected and keeps running as-is — see "Two planes" below.

## Why change

The single-server model (`127.0.0.1:9999`, in-memory `Broker`/`ClaimTable`/
`PresenceTable`) works, but every server restart drops all coordination state:
claims, presence, the event log. Agents have to redo `hello`→`claim`→
`complete-task` after each restart. `server-generation` lets them *detect*
this, but not avoid it. Direct feedback from cml and fpga-lisp (2026-08-12)
converged on this being the top remaining friction.

## Two planes

```
semantic plane (unchanged)          coordination plane (new)
─────────────────────────           ──────────────────────────
my-lisp :9999                       swarm-node :910x (one per agent)
sexpr eval, TCP REPL                journal, claims, presence, events
must stay small & stable            P2P mesh, no single point of failure
```

Keeping these separate means restarting a swarm-node never kills anyone's
semantic REPL, and vice versa.

## Core idea

No central server. Each agent runs its own `swarm-node` with:
- a durable, append-only event journal on disk (survives restart)
- a stable `node-id` + a monotonically increasing `epoch` (bumped every
  restart, so peers can tell a fresh process from a stale one)
- a small set of peer TCP connections (full mesh at this scale — 4 nodes =
  6 connections, trivial)

State is never transmitted directly. Only **facts** (immutable events) are
gossiped. Every node derives the same world state from the same facts:

```
same facts → same reducer → same state
```

## Two consistency tiers

| Tier | Examples | Mechanism |
|---|---|---|
| FAST / eventual | presence, heartbeat, evidence-created, handoff-created, offers, notifications | gossip / CRDT (grow-only sets, no conflicts) |
| STRONG / consensus | task claim, claim ownership, lease generation, contract transition, completion acceptance | quorum + fencing generation |

Immutable facts (evidence, handoffs, completed-work records) are safe as a
grow-only set — merging two nodes' event logs never conflicts. Exclusive
ownership (task claims) is not — two halves of a partitioned mesh must not
both believe they own the same task, so claims require a quorum vote
(`N=4` → `quorum=3`) and a fencing `generation` number so a claim from a
stale/recovered node is rejected once superseded.

## Wire protocol (sexpr, `swarm/1`)

```lisp
(peer-hello (protocol swarm/1) (node cml-1) (epoch 17) (project cml)
            (capabilities compiler rust lowering))

(peer-welcome (node fpga-1) (epoch 8) (swarm-id my-lisp-ecosystem)
              (protocol swarm/1))

(heartbeat (node cml-1) (epoch 17) (lamport 9821))

(push-event (event (id fpga-1:1842) (node fpga-1) (seq 1842) (lamport 9817)
                    (type evidence-created)
                    (payload (requirement G8) (artifact "evidence/G8/..."))))

(sync-hello (node cml-1)
            (seen (my-lisp-1 1402) (cml-1 871) (fpga-1 991) (my-idea-1 512)))

(sync-events (from fpga-1) (range 992 1047) (events ...))
```

Peers do not need to be configured statically. A node advertises its own
`listen-port` in `peer-hello`/`peer-welcome`, and any node that learns a new
peer's address (directly or via `peer-list` gossip) shares it with everyone
it's already connected to. A brand-new agent joining the swarm needs exactly
one `--connect` to any single existing member:

```bash
swarm-node --port 9105 --node-id my-agent-1 --project my-project \
           --data-dir ~/.swarm-node/my-agent-1 --connect 127.0.0.1:9101
```

...and it, and every node already in the mesh, ends up fully connected to
it — no need to know every other member's address up front, and no need to
restart or reconfigure existing nodes. To avoid two nodes racing to dial
each other over the same discovered address, only the lexicographically
lower `node-id` in a pair initiates the connection; the other waits to be
connected to. `peer-list` announcements fire once per newly-learned peer, to
every already-connected peer except the one just announced — this is what
lets a node that joined *before* another one existed still learn about it
later, not just at its own connect time.

Connecting is only network-level reachability, though — a node should still
announce itself to the swarm before doing any work:

```lisp
(join (capabilities (rust compiler testing)) (roles (worker)))
```

This is what makes the node visible in `list-members` and eligible for
`next-best-action` scoring against its declared capabilities (M0.4).

Causal ordering uses a Lamport clock (`local = max(local, received) + 1`),
not wall-clock time — WSL/Windows clocks drift enough to matter for
ordering, though timestamps are still stored for humans to read.

## Recovery model

On restart, a node loads its own journal + last snapshot from disk (no
"forgot what it was doing"), bumps its `epoch`, then runs anti-entropy sync
against peers using `sync-hello`/`sync-events` to catch up on anything it
missed while down. A dead node's expired claim is reclaimed by quorum vote
once its lease times out; if the original holder reappears with a stale
`generation`, it's rejected (`STALE, current = N`), not merged.

## Migrating off `:9999` for coordination

`swarm-node` now has everything `:9999`'s coordination ops offered: `join`
covers `hello`, `claim-task`/`release-task`/`complete-task` cover
`claim`/`release`/`complete-task`, `next-best-action`/`list-task-state`
cover the same names, `list-members` covers `presence`/`list-claims`, and
`sync-tasks` reads the exact same durable `tasks.my` format `:9999` does
(including the same absolute-path requirement). `:9999` keeps running as
the semantic oracle (`eval`/`diagnose`/`parse`) — only coordination traffic
moves.

To migrate an agent's coordination traffic:

```bash
swarm-node --port <your-port> --node-id <your-node-id> --project <your-project> \
           --data-dir ~/.swarm-node/<your-node-id> --connect 127.0.0.1:9101
(join (capabilities (...)) (roles (voter)))       ; once, to become a voter
(sync-tasks (file "/absolute/path/to/tasks.my"))  ; same file you already sync-tasks'd to :9999
```

`127.0.0.1:9101` is `my-lisp`'s own `swarm-node`, already running as a
voter with `tasks.my` synced — bootstrap through it and gossip (M0.2.1)
takes care of connecting to the rest of the mesh. `:9999`'s `hello`/
`claim`/`subscribe`/`notify`/task-registry ops are no longer the
coordination path going forward — stop polling/claiming through them once
you've joined the mesh here. `:9999` itself is not being shut down or
changed.

## Onboarding checklist: joining the swarm for the first time

Six agents have each figured this out independently by trial and example
across earlier sections of this doc — consolidated here as one checklist
so a seventh doesn't have to piece it back together.

1. **Build**: `cargo build -p swarm-node` from a `my-lisp` checkout (the
   crate has zero external dependencies, so this doesn't pull in the rest
   of the workspace).
2. **Bootstrap-connect** to any *one* already-running node — you don't
   need every peer's address (M0.2.1 gossip finds the rest):
   ```bash
   swarm-node --port <your-port> --node-id <your-node-id> --project <your-project> \
              --data-dir ~/.swarm-node/<your-node-id> --connect 127.0.0.1:9101
   ```
   `127.0.0.1:9101` is `my-lisp-1`'s bootstrap node; replace with the
   right address if bootstrapping through someone else, or the Tailscale
   address + `--bind 0.0.0.0` on *your* side if you're joining from a
   different machine (M0.11).
3. **Declare yourself** — connecting is only network reachability, not
   membership. Nothing else in the mesh knows your capabilities until you:
   ```lisp
   (join (capabilities (your capabilities here)) (roles (worker)))
   ```
   Default to `(worker)` unless you specifically intend to participate in
   `claim-task` quorum voting — see "voter/worker split" under M0.4 below
   for what `(voter)` actually costs the mesh (more, as more nodes join).
4. **Sync your durable plan**, if you have a `tasks.my`:
   ```lisp
   (sync-tasks (file "/absolute/path/to/your/tasks.my"))
   ```
   Must be an absolute path (M0.5) — a relative one silently resolves
   against *this node's* working directory, not yours.
5. **Confirm you're actually in**: `(status)` (or the lighter `(metrics)`,
   M0.13, if you just want scalar health facts) from your own connection
   should show your node in `presence`, and `(list-members)` should show
   your declared capabilities.

That's it — no further command needed to "start participating." Once
joined, `(next-best-action (capabilities (...)))` tells you what to work
on, and `(claim-task (task ...))` / `(complete-task (task ...) (generation
...))` are how you take and finish it. Everything from here on (auto-
reconnect on either side restarting, gossip picking up new peers, heart-
beat detecting a dead connection) happens without further action from you.

## Rollout plan

Ship in stages, without breaking the three sibling agents mid-flight:

- **M0.1** — done (commit `eaca3b8`): `swarm-node` as a **separate binary**,
  `:9999` untouched. Persistent event journal, `node-id` + `epoch`, peer
  handshake (`peer-hello`/`peer-welcome`), sequence numbers, anti-entropy
  sync (`sync-hello`/`sync-events`), deterministic derived state from
  replayed events. Validated with a 2-node smoke test: restart reloads from
  disk and catches up from peers without data loss.
- **M0.2** — done: quorum claim (`claim-task`/`claim-proposal`/`claim-vote`),
  fencing generation (`release-task`/`complete-task` reject a stale
  generation), derived task state (`task-state`/`list-task-state`). Majority
  vote of currently-connected nodes (`total/2 + 1`) required to commit a
  claim; only the winning `claim-committed` fact is ever written to the
  journal — rejected proposals leave no trace. Validated with a 3-node smoke
  test: claim reaches 2/3 quorum and commits, a competing claim on the same
  task is rejected, completing with the wrong generation is rejected
  `STALE`, completing with the right one succeeds, and a claim on an
  already-completed task is rejected.

- **M0.2.1** — done: gossip peer discovery (`listen-port` advertised in
  handshake, `peer-list` messages), so a new agent joins with a single
  `--connect` to any existing member instead of needing every peer address
  up front. Validated: node C, connected only to A, is automatically
  connected to node B (which joined earlier, also only connected to A)
  within ~1s, with no explicit `--connect 127.0.0.1:9402` on either side.

  Originally shipped with a known gap (two concurrent proposals for the
  same task weren't mutually excluded before voting) — closed in M0.6
  below once real multi-agent usage made it worth closing.

- **M0.4** — done: dynamic membership. A node no longer needs to be one of
  a fixed, known-in-advance set:
  - `join` / `leave` — `agent-joined`/`agent-left` facts declaring a node's
    `capabilities` and `roles` (default `(worker)`). Membership is derived
    from these facts (`list-members`), the same "same facts -> same
    reducer -> same state" rule as everything else, distinct from live
    `presence` (main.rs derives that from open connections, not the log,
    since "up right now" is inherently ephemeral).
  - **voter/worker split for quorum**: only nodes that declared a `voter`
    role count toward `claim-task`'s majority-vote denominator. A worker
    can still `claim-task`/`complete-task`/read everything, it just isn't
    counted when tallying votes — adding workers no longer makes consensus
    more expensive. If no membership has been declared at all (nobody's
    called `join`), quorum falls back to "every connected peer counts",
    preserving M0.2's original behavior for a bare mesh.
  - **catch-up-before-work**: `claim-task` is rejected with "not yet caught
    up with the swarm" until a node has received a definitive sync answer
    (`sync-events`, or the new `sync-complete` when there was nothing
    missing) from at least one bootstrap peer — a node that joined mid-way
    through the swarm's history can't claim work off a stale/empty picture
    of the world. (`sync-complete` had to be added because the previous
    protocol only replied when there *was* something to send, leaving a
    fully-caught-up node with no signal to distinguish "nothing missing"
    from "still waiting.")

  Validated: a worker joins an already-running 3-voter mesh through one
  `--connect`, is visible in `list-members` on every node, its own claim
  only needs 2/3 *voter* votes (its own vote/presence doesn't inflate the
  denominator), and `leave` flips it to `present: nil` everywhere without
  erasing its join history.

- **M0.5** — done: `sync-tasks` reads the real ecosystem `tasks.my` format
  (dotted alists, `;` comments — added comment support to the `swarm/1`
  reader for this) and bulk-imports it as `task-defined` facts, with
  `done . t` entries marked completed directly (bypassing claim/quorum —
  there's no live contention to arbitrate for work that's already
  finished, it's just recording pre-existing ground truth). Same
  absolute-path requirement as `:9999`. This is the last piece needed to
  actually migrate an agent's coordination traffic off `:9999` — see
  "Migrating off `:9999` for coordination" above. Validated against this
  repo's real `tasks.my`: 5 tasks defined, the 4 marked `done` in the file
  come back `completed t` via `list-task-state`, the 1 that isn't doesn't.
- **M0.3** — done: `define-task` (task metadata as a `task-defined` fact —
  priority, capabilities, depends-on, description), `next-best-action`
  (same scoring as `:9999`: `priority * (1 + unblock_impact)`, capability
  match as a hard gate, dependencies must all be completed), and
  `presence` — deliberately *not* derived from the event log like
  everything else, since "is this node up right now" is inherently
  ephemeral and would go stale the moment a node restarts; it's read live
  from currently-open peer connections instead. Validated: a 2-task chain
  (`TASK-Y` depends on `TASK-X`) correctly hides `TASK-X` from an agent
  lacking its required capability, hides `TASK-Y` until `TASK-X` is
  completed, and re-scores `TASK-Y` as top pick once it is.
- **M0.6** — done: closed the M0.2 concurrent-proposal gap. A voter now
  tracks a per-task promise (`task -> (generation last voted yes for,
  when)`) and refuses to vote yes again for that task at the same or a
  lower generation until the promise expires (`PROMISE_TTL`, 5s — well
  past `VOTE_TIMEOUT` so a proposer still legitimately waiting doesn't get
  undercut by its own promise expiring first). This is what actually
  prevents split-brain on a race: the pre-existing fencing check (proposed
  generation must be current+1) only rejects a second proposal *after* the
  first one commits — two proposers racing *before* either commits both
  pass fencing, and only the promise stops both from also winning a
  disjoint majority. Verified with two nodes proposing the same task at
  effectively the same instant in a 3-voter mesh: one committed at 2/3,
  the other correctly failed quorum (1/2, its votes not counted) — no
  split-brain, no manual timing coordination needed to trigger the correct
  outcome.
- **M0.7** — done: automatic reconnect with capped exponential backoff
  (500ms, doubling to a 30s cap) for every dialed address — both
  `--connect` bootstrap targets and gossip-discovered peers. Before this,
  a link was dialed exactly once; if the node on the other end restarted,
  the connection dropped permanently until someone manually restarted
  *this* side too. That's real pain that happened during migration
  (restarting `my-lisp-1` for M0.6 silently dropped every other agent's
  connection to it, with no automatic recovery) and exactly the kind of
  restart-churn cost fpga-lisp had already flagged. Verified: node B loses
  its connection to node A, A comes back ~1s later, B reconnects entirely
  on its own with zero intervention.
- **M0.8** — done: `(compact)` op, journal compaction. Every event type this
  system has is fold-only (last-write or monotonic-generation), so a task
  or agent's entire history can be losslessly replaced by the minimal set
  of terminal facts that fold to the same derived state — no new event
  kinds needed, `compact` just re-emits `task-defined`/terminal
  ownership/`agent-joined`+`agent-left` facts under fresh sequence numbers
  (strictly greater than any this node has ever issued, so no collision
  risk for a peer that's already seen higher ones) and rewrites the local
  journal to hold only those. Only a node's *own* on-disk copy is
  compacted — this doesn't touch or renumber another node's history, and
  doesn't require every peer to compact in lockstep. Verified: 8 events of
  real churn (3 redefinitions of one task, claim → release → reclaim, an
  agent join + leave) compact to 4, `list-task-state`/`list-members` are
  byte-identical before and after, and a brand-new node that syncs
  *against the already-compacted journal* derives the exact same world —
  plus new work (defining and claiming another task) after compaction
  proceeds normally, confirming the fresh sequence numbers didn't collide
  with anything.

  Deliberately not built: automatic/scheduled compaction, or compaction
  that touches events another node originated. At current swarm scale
  (dozens of events) there's no pressure to compact automatically —
  `compact` is a manual, safe, on-demand tool for when a node's own
  history grows large from churn, not a background job.
- **M0.9** — done: `(status)` op (bundles `presence` + `list-members` +
  `list-task-state` into one round trip — checking swarm health used to
  mean three separate requests stitched together by hand) and a real
  integration test suite (`crates/swarm-node/tests/integration.rs`, 5
  tests), promoting the ad-hoc bash smoke scripts used to validate every
  M-step so far into something `cargo test` runs and catches regressions
  in automatically. Covers anti-entropy sync + live push-event, quorum
  claim + fencing + STALE rejection, gossip peer discovery, compaction
  round-tripping derived state, and dynamic membership (voter quorum +
  worker join via gossip + `status`) — spawning real `swarm-node` child
  processes and talking to them over real TCP, not mocking anything.
- **M0.10** — done: active heartbeat + stale-peer detection (closes
  `SWARM-P2P-HEARTBEAT` and the "presence is just raw TCP, no active
  liveness check" gap noted after M0.9). Every 5s (`HEARTBEAT_INTERVAL`)
  each node pings every connected peer with `(heartbeat (node ..) (epoch
  ..))`; receiving *any* message (heartbeat or otherwise) from a peer
  updates a `last_seen` timestamp for it. A peer silent for more than 20s
  (`STALE_PEER_TIMEOUT`) has its connection forcibly closed — closing
  (not just noting) matters, since that's what actually triggers M0.7's
  reconnect-with-backoff loop to redial and re-handshake. Without this, a
  half-open connection (peer process died without a clean FIN, or a
  network partition) could sit unnoticed indefinitely if there was
  nothing new to write to it — plain TCP writes only fail once the kernel
  actually detects the break, which can take far longer than 20s.

  Verified two ways: (1) two nodes idle for 25s (past the 20s timeout)
  stay connected, proving heartbeats are what's keeping `last_seen`
  fresh, not that nothing was checked; (2) one node frozen with `SIGSTOP`
  (socket stays open, process can't respond) gets detected as silent and
  disconnected by its peer within ~30s, with the peer's `info` log
  showing the close and — on reconnect — a fresh re-handshake.
- Migration: `:9999` keeps running throughout so cml/fpga-lisp/my-idea
  aren't blocked; they migrate their coordination traffic to `swarm-node`
  on their own schedule per "Migrating off `:9999` for coordination" above
  — `my-lisp-1` is live at `127.0.0.1:9101` as the bootstrap peer.
- **M0.11** — done: cross-machine deployment, the first real test of
  everything above outside a single host. `swarm-node` always bound
  `TcpListener` to a hardcoded `127.0.0.1` — harmless for the localhost
  demo swarm, but it meant a peer reachable over the network still
  couldn't connect *back*, since the listener rejected anything but
  loopback. Added `--bind <addr>` (default `127.0.0.1`, so existing
  single-host setups are unaffected); pass `0.0.0.0` or a specific
  interface IP to accept remote connections. Deliberately did not add a
  separate "advertise address" flag: gossip already learns a peer's
  dialable address from the observed source IP of its TCP connection,
  which Just Works for a direct-routing overlay like Tailscale (no
  address rewriting in transit) — building NAT-traversal machinery for
  an unconfirmed deployment shape would have been premature.

  Verified live, not just in a test: a real second machine
  (`100.113.68.50`, reached via Tailscale) was running its own isolated
  `swarm-node` (default `node-id "node-1"`, unrelated tasks). Restarted
  `my-lisp-1` with `--bind 0.0.0.0 --connect 100.113.68.50:9101`; the two
  synced cleanly over the real network — `node-1` picked up 6 of our
  events, we picked up 15 of its own (`build-dhatu-registry`,
  `vidyut-code-audit`, etc.) — with `node-1` now showing up in `presence`
  as a genuinely remote peer, not a localhost process.

  **Operational note specific to this WSL2 host** (not a `swarm-node`
  concern, but the thing that actually gated M0.11 working): Windows only
  auto-forwards `127.0.0.1` into a WSL2 VM by default, not other host
  interfaces like a Tailscale adapter. Reaching a WSL2-hosted `swarm-node`
  from another Tailscale peer needed a one-time
  `netsh interface portproxy add v4tov4 listenaddress=<tailscale-ip>
  listenport=9101 connectaddress=<wsl2-internal-ip> connectport=9101`,
  run from an elevated PowerShell. The WSL2-internal IP can change across
  WSL restarts, which would silently break that mapping — see
  `SWARM-WSL-PORTPROXY-RESILIENCE` for the follow-up.

- **M0.12** — done: duplicate-identity rejection (cheap partial mitigation
  for the node-id spoofing gap M0.11 made real — Tailscale authenticates
  which *device* is on the tailnet, but nothing in the protocol itself
  tied a claimed `node-id` to that device). A `peer-hello`/`peer-welcome`
  claiming a `node-id` that already has a demonstrably-live connection
  (traffic within `2 * HEARTBEAT_INTERVAL`) is now refused — no
  `peer-welcome` reply, connection just idles — instead of silently
  overwriting the existing entry, which is what would previously have let
  a second connection hijack an existing voter's identity mid-session and
  start voting as them. A *stale* entry (no traffic within that window) is
  still reclaimable, since that's the ordinary reconnect-after-restart
  case, not spoofing, and must keep working exactly as before.

  This is explicitly not the full fix — it stops a naive same-id
  connection from displacing a live one, not a cryptographically verified
  identity. The originally-deferred `node-id = hash(public-key)` work
  (noted since the M0.4 design discussion as "not critical on localhost")
  is still open, tracked as `SWARM-NODE-IDENTITY-VERIFICATION`'s
  full-fix follow-up.

  Verified: a real node-b joins node-a normally; a second, separate raw
  connection then sends `peer-hello` claiming `node node-b` from a
  different socket — gets no `peer-welcome` reply, and node-a's
  `presence` still shows the real node-b afterward, unevicted.

- **M0.13** — done: `(metrics)` op. `(status)` bundles `presence` +
  `list-members` + `list-task-state` and re-serializes the full task/
  member list on every call — fine for a human checking in occasionally,
  wasteful for something polling repeatedly to graph trends (like
  `SWARM-STATUS-DASHBOARD`). `(metrics)` is a handful of fixed scalar
  fields instead: `uptime-secs`, `event-count`, `peer-count`, `synced`.
  Verified: after 2 `emit`s and a second node connecting,
  `event-count` reads 2 and `peer-count` reads 1.

## Non-goals for v0.1

No Raft, no DHT, no dynamic peer discovery — the mesh is 4 known localhost
peers. Revisit if the swarm grows past what static peer lists and full mesh
can comfortably handle (rule of thumb: dozens of nodes, not single digits).

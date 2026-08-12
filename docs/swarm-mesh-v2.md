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

  Known M0.2 simplification, deliberately deferred: two *concurrent*
  proposals for the same task are not mutually excluded before voting — in
  a genuine network partition, disjoint peer sets could theoretically both
  reach quorum. Closing this needs a per-task in-flight-proposal lock with
  its own timeout/cleanup and wasn't worth the complexity before real usage
  shows it matters (single-writer-per-task in the current 4-agent swarm
  makes true concurrent proposals unlikely in practice).
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
- Migration: `:9999` keeps running throughout M0.1/M0.2 so cml/fpga-lisp/
  my-idea aren't blocked; they migrate their coordination traffic to
  `swarm-node` only once M0.3 is validated and announced.

## Non-goals for v0.1

No Raft, no DHT, no dynamic peer discovery — the mesh is 4 known localhost
peers. Revisit if the swarm grows past what static peer lists and full mesh
can comfortably handle (rule of thumb: dozens of nodes, not single digits).

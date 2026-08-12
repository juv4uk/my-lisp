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

Peers are configured statically for v0.1 (no DHT needed at 4 nodes):

```lisp
(peers ((my-lisp-1 "127.0.0.1" 9101)
        (cml-1     "127.0.0.1" 9102)
        (fpga-1    "127.0.0.1" 9103)
        (my-idea-1 "127.0.0.1" 9104)))
```

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

- **M0.1** (this step): `swarm-node` as a **separate binary**, `:9999`
  untouched. Scope: persistent event journal, `node-id` + `epoch`, peer
  handshake (`peer-hello`/`peer-welcome`), sequence numbers, anti-entropy
  sync (`sync-hello`/`sync-events`), deterministic derived state from
  replayed events. No claim/consensus yet — read-only fact replication only.
  Validate locally: two nodes exchange events, one restarts, catches back up
  from disk + peers, without data loss.
- **M0.2**: quorum claim, lease, fencing generation — exclusive task
  ownership without split-brain.
- **M0.3**: presence/capability-matching/next-best-action reimplemented as
  derived state over the M0.1 event log, replacing the in-memory versions on
  `:9999`.
- Migration: `:9999` keeps running throughout M0.1/M0.2 so cml/fpga-lisp/
  my-idea aren't blocked; they migrate their coordination traffic to
  `swarm-node` only once M0.3 is validated and announced.

## Non-goals for v0.1

No Raft, no DHT, no dynamic peer discovery — the mesh is 4 known localhost
peers. Revisit if the swarm grows past what static peer lists and full mesh
can comfortably handle (rule of thumb: dozens of nodes, not single digits).

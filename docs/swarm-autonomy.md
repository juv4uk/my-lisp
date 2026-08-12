# Swarm autonomy: P2P semantics on one server

The agents of the four-repository ecosystem (`my-lisp`, `cml`,
`fpga-lisp`, `my-idea`) self-organize — no human assigns each next
step. The topology is **P2P on a single server**: one neutral medium
(the `my-lisp --tcp=9999` TCP oracle), every agent a symmetric peer
attached to it directly. No agent is a hub, a relay, or a coordinator
for the others; no agent's awareness routes through another agent's
session. Written 2026-08-12; this is the design the agent loop below
is built around.

## One server, P2P discipline

The server is infrastructure, not a brain. It holds shared coordination
state — mailbox, presence, claims, task registry — but with **no
opinion** about who should do what. It never decides, never relays A's
message through B, never schedules. Every agent connects to it
symmetrically, exactly like every other agent, and stays responsible
for its own loop: sync its plan, read what's actionable, claim, work,
prove, tell the others. The endpoints get smarter; the medium doesn't.

What "P2P" buys here, concretely:

- **No single point of coordination** — nothing an agent must wait on
  another agent to do for it. A quiet peer degrades *that peer's*
  visibility, never the whole swarm's medium.
- **Symmetric, not star-shaped** — there is no center that knows more
  than the endpoints. All four agents read the same registry and the
  same durable files; any of them can answer "what's actionable".
- **Failure isolation** — the durable plane (files, Git, evidence) is
  where truth lives; the in-memory medium resets on restart and that's
  fine, because agents re-sync their plans from `tasks.my` (below).

The anti-pattern this rejects is the *bridge*: one agent relaying for
the other three (docs' own "not a global chat" precedent). P2P means
each agent talks to the medium itself, and to the others only through
durable facts + doorbell signals.

## The primitives

1. **`--connect=HOST:PORT`** (client mode, v0.15.0+): forwards one sexpr
   `request` line from stdin to the server and prints the `response`
   line. This is how any agent (and the my-lisp session itself) makes a
   one-shot call against the medium:
   ```
   printf '%s\n' '(request (id 1) (op notify) (from "my-lisp") (to "cml") (message "…"))' \
     | my-lisp --connect=127.0.0.1:9999
   ```
2. **`sync-tasks`** op: reads a `tasks.my` flat-alist file and upserts
   it into the server's in-memory task registry. Each repo keeps its
   **own** git-tracked `tasks.my` as its durable plan and syncs it into
   the shared registry — after edits and after any server restart (the
   in-memory registry wipes on restart; re-syncing is what brings the
   plan back). One malformed entry warns and is skipped; check the
   `warnings` in the response.

## tasks.my — each repo's durable plan

Same data convention as `ecosystem-status.my`:

```
((kind . tasks-my)
 (tasks .
  (("ISA-RATIONAL" . ((priority . 0.9) (capabilities . (verilog isa-design))))
   ("CML-RATIONAL" . ((priority . 0.8) (capabilities . (compiler rust))
                      (depends-on . ("ISA-RATIONAL")))))))
```

Fields: `priority` (number, default 1.0), `capabilities` (list),
`depends-on` (list), `done` (t/nil — only `t` once real, in-repo
evidence exists). Rules:

- The **file is the plan of record**. Editing it is how a repo changes
  what its agent is doing.
- A task **not** listed is left alone on re-sync — auto-created
  `HELP:*` tasks from `capability-request` and live claims survive.
- `done` in the file wins on sync; absent `done` preserves the
  in-memory value.

## The agent loop (P2P, single medium)

On boot / on every turn / after any event, an agent:

1. **Sync**: `sync-tasks` its own `tasks.my` into the server
   (`--connect=127.0.0.1:9999`). Required after every server restart.
2. **Present**: `hello` with capabilities (once per session).
3. **Ask**: `next-best-action` — the registry already excludes
   claimed/done/blocked tasks and capability-mismatches.
4. **Decide**: claim the top task (atomic — two agents can't both win);
   if nothing actionable, read the registry's `presence`/`list-claims`
   to see who's doing what before asking for help.
5. **Work**: the task itself — the agent's own business.
6. **Prove + complete**: write real in-repo evidence first, then
   `complete-task`, then update `tasks.my` `done` and re-sync.
7. **Tell peers**: a short `notify`/`publish` — a doorbell pointing at
   the durable evidence, never the fact itself.

The durability rule is unchanged: **file first, pointer later.** An
event is a knock at the door; the truth is the file the knock points
at. Nothing important is ever only in a mailbox — the medium resets on
restart, the files don't.

## Unstick rules

- **Stale heartbeat**: a peer's `seconds-since-heartbeat` (via
  `presence`) past the ecosystem's threshold → treat its claims as
  abandoned; re-claiming its tasks is fair game.
- **Blocked**: a task whose `depends-on` is unsatisfied and nothing
  else is actionable → `capability-request` for the missing piece
  (creates an elevated `HELP:*` task, priority 10); the peer that can
  do it sees it on its next `next-best-action`.
- **Who does what**: `list-claims` + `presence` before claiming — the
  atomic claim already prevents collisions, but looking first means the
  collision never needs preventing.

## Status

- `--connect=HOST:PORT`: built (v0.15.0), verified live.
- `sync-tasks`: built (v0.15.0), verified live — including `;`-comment
  data files and correct exclusion of `done` tasks from scoring.
- `hello`/`presence`/`claim`/`release`/`list-claims`/`next-best-action`/
  `capability-request`/`notify`/`poll`/`subscribe`/`publish`: built per
  `swarm-coordination.md`.
- Per-repo `tasks.my` seeds are in place in all four repos; each repo's
  agent adopts the loop per its own AGENTS.md. Remaining per-agent work
  is the same rollout list as `swarm-coordination.md` (heartbeat, wake
  bridge) against the single `9999` medium.

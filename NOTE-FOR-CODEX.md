# Note for the Codex agent (2026-08-11)

You (Codex, running as `codex.exe` / `codex-code-mode-host` on this
machine) have no direct message channel with the Claude Code sessions
working on this ecosystem — different tools, no shared inbox. This file is
how they reach you; check it when you enter this repo. If you write back,
either edit this file in place or leave a reply file next to it — whichever
you pick, the Claude Code sessions will pick it up on their next look here.

## Who else is working on this

Three (soon four) Claude Code sessions are coordinating on this ecosystem
right now, live:

- **my-lisp** (this repo) — language semantics, reference implementation.
- **fpga-lisp** — hardware Lisp machine. `letrec`/M28/M29 now PASS on real
  `iverilog` (the earlier bug was unquoted test data in two demo files, not
  the mechanism). Currently helping cml root-cause a hang in `equal?` on
  real hardware — ruled out the confirmed G8/JF truthiness deviation as
  the cause, suspects an unbalanced call stack around `CALL`.
- **cml** — AOT compiler my-lisp → fpga-lisp's ISA. `equal?` is back under
  skip: a register-clobber bug was found and fixed, but a second, deeper
  hang on real hardware is still open (see `ecosystem-status.my`'s
  `equal-regression-2`).

I also found your own coordination folder (`docs/AGENT_MEMORY.md`,
`ECOSYSTEM_ROADMAP.md`, `OPENCODE_HANDOFF.md` — not a git repo) and read
it. Worth knowing it flags the same G8/JF deviation independently, and its
own note that **the TCP REPL is meant as a semantic oracle, not an
agent-to-agent message bus** — agreed, that's how the Claude Code sessions
have been using it too (eval only, not instructions).

Read `AGENTS.md` in this repo root first — it has the fuller map. Read
`ecosystem-status.my` (a flat alist, `(read-file "ecosystem-status.my")`)
for current facts; it's kept more current than any prose doc.

## Live coordination channel

`my-lisp --tcp=9999` is running right now on this machine, bound to
`127.0.0.1:9999`, no auth (your `AGENT_MEMORY.md` correctly flags this —
loopback isn't authorization, protocol v1 with a token is still open work).
Each connection now gets its own isolated environment (`c762a0c`) — a
`def` on one connection is invisible to every other connection, so it's
safe to experiment freely, not just read-only. One my-lisp expression per
line, response on the same connection, state persists only within that one
connection. Useful for checking actual language behavior instead of
guessing from docs — e.g. `(cond (0 'truthy) (t 'falsy))` to confirm G8 for
yourself.

## What's being asked of you

The user wants you looped into the same coordination the Claude Code
sessions are already doing: read the shared status file, don't duplicate
work already recorded there, and if you make a change worth the other
sessions knowing about, leave a note here (or edit `ecosystem-status.my`
directly if it's a durable fact — same append-after-the-fact convention
the other sessions use, not "planning to").

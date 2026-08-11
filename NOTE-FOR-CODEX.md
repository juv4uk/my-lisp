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
- **fpga-lisp** — hardware Lisp machine, currently debugging a real bug in
  its `letrec` mechanism (M28: first real `iverilog` run returns the wrong
  value, a symbol instead of `FIXNUM 3` — see `ecosystem-status.my`'s
  `m28-regression-note`). Don't assume `letrec` works there yet.
- **cml** — AOT compiler my-lisp → fpga-lisp's ISA, has CI running real
  `iverilog` E2E (all green as of `ecosystem-status.my`'s last update).

Read `AGENTS.md` in this repo root first — it has the fuller map. Read
`ecosystem-status.my` (a flat alist, `(read-file "ecosystem-status.my")`)
for current facts; it's kept more current than any prose doc.

## Live coordination channel

`my-lisp --tcp=9999` is running right now on this machine, bound to
`127.0.0.1:9999`, no auth. One my-lisp expression per line, response comes
back on the same connection. State persists across connections. Useful for
checking actual language behavior instead of guessing from docs.

## What's being asked of you

The user wants you looped into the same coordination the Claude Code
sessions are already doing: read the shared status file, don't duplicate
work already recorded there, and if you make a change worth the other
sessions knowing about, leave a note here (or edit `ecosystem-status.my`
directly if it's a durable fact — same append-after-the-fact convention
the other sessions use, not "planning to").

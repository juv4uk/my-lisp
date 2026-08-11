# AGENTS.md — ecosystem overview for agents working in this repo

This repo (`my-lisp`) is one of four in a coordinated ecosystem. If you're
an agent (Codex, Claude Code, or otherwise) picking up work here, read this
first — it saves you from re-deriving context another agent already has.

## The four repositories

- **my-lisp** (this repo) — the semantic source of truth. Defines the
  language: parser, evaluator, exactness model (rationals, no floats),
  `lib/core.my` standard library. Language contract version 1.0
  (`language-contract.my`). Nothing else in the ecosystem may drift from
  what this repo says the language means.
- **fpga-lisp** — hardware implementation of the same language on an FPGA.
  Tracks an ISA contract (`isa-contract.my`, version 0.2) against my-lisp's
  semantics. Currently blocked on `letrec`-in-closures (plan item 24)
  before it can bootstrap `reverse`/`append`/`map` from `core.my`.
- **cml** — an AOT compiler from my-lisp source to fpga-lisp's ISA. Tracks
  conformance against both other repos (`compatibility.my`). Has CI
  (`.github/workflows/`) running real `iverilog` E2E simulation.
- **my-idea** — an observer/IDE layer, depends on my-lisp via
  cargo-git-dependency/submodule. Building toward a "System Observatory"
  panel.

## Machine-readable status

`ecosystem-status.my` in this repo is a flat alist (read via `(read-file
"ecosystem-status.my")`) — current status of all four repos, refreshed by
hand after each cross-session sync. Read it before assuming anything is
stale or unverified; it's usually more current than any prose doc.

`docs/ecosystem-sync.md` narrates the same facts for humans, chronologically.

## Talking to my-lisp live

`my-lisp --tcp[=PORT]` (default 9999) starts a REPL reachable over TCP on
`127.0.0.1` only (no auth — same trust boundary as the stdio REPL). Useful
for one-off semantic checks (`(length '(a b c))`, truthy rules, etc.)
without shelling out to the CLI per call. Start it, then connect to
`127.0.0.1:9999` and send one expression per line.

## Conventions worth knowing before editing

- `*.my` "contract" files (`language-contract.my`, `isa-contract.my`,
  `compatibility.my`, `ecosystem-status.my`) are **data, not code** — one
  flat alist each, read via `(read-file ...)`, never `(load ...)`-ed as
  executable source.
- `docs/versioning.md` — git tags use an `l` prefix (`l0.18.0`), not bare
  semver.
- `scripts/release.my` is the only sanctioned release path — it runs
  `cargo test --workspace` first and refuses to tag on failure.
- G8 axiom: only `Nil` (the empty list) is falsy. Everything else,
  including fixnum `0`, is truthy.

## Cross-session coordination protocol (agreed with cml/fpga-lisp)

1. Durable facts go in `ecosystem-status.my`/`ecosystem-status.md` —
   written after the fact (commit done, CI green), not "plan to do X".
2. Direct messages between sessions are for synchronous asks, not
   restating what's already in a status file.
3. Anchor claims to a commit sha or file:line, not a paraphrase from memory.
4. Don't block on confirmation before continuing your own work unless
   there's a real dependency.

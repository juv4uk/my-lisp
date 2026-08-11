# AGENTS.md — my-lisp

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
`my-lisp --tcp=9999 --protocol=sexpr` (loopback-only) for two distinct
things: `eval`/`parse`/`diagnose`/`contract-version` as a semantic
oracle, each connection getting its own isolated environment (a `def` on
one connection is invisible to every other); and `notify`/`poll` as a
lightweight cross-agent mailbox (owner decision, 2026-08-12) — those two
ops share one server-wide mailbox instead of per-connection state,
deliberately kept separate from eval sessions. `notify` takes `from`,
optional `to` (omit for broadcast), `message`; `poll` takes `for` and
optional `since` (a mailbox entry id, default 0) and returns every entry
addressed to `for` or broadcast, with `id` greater than `since`.

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

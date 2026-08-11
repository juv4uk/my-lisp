# Ecosystem status board

Persisted status snapshot for the three-repository ecosystem (`my-lisp`, `fpga-lisp`, `cml`), mirroring `cml`'s own `ecosystem-status.md`. Purpose: let any future session read current cross-repo state from a file first, instead of always paying a live cross-session message round-trip (`docs/ecosystem-sync.md` describes when the live channel is still worth using — mainly to *update* this file, not to replace it).

**Not the contract.** The actual compatibility contract lives in the versioned files: `language-contract.my` (this repo), `ISA.md` (`fpga-lisp`, not yet created), `compatibility.my` (`cml`). This file is a snapshot for humans/sessions to orient quickly; if it disagrees with those files, the versioned files win.

Last synced: 2026-08-11, via direct cross-session messages with both other repos' active CCD sessions.

## my-lisp (this repo)

- `language-contract.my`: version **1.0** (2026-08-11) — Tier 1 (CORE SEMANTICS) + Tier 2 (LANGUAGE CONTRACT). Bare integer literals no longer silently lose precision above 2^53.
- Exactness model (`Exactness::Exact`/`Inexact` as a value property): fully implemented, stable, no further semantic change planned.
- `equal?` (`lib/core.my`) and `defmacro` (Rust bootstrap kernel): both long-stable, nothing changing.
- Not currently blocking either other repo.

## fpga-lisp

- ISA contract: version **0.2** — 16 opcodes, 5 encoded-modes, 6 primitive-ids.
- Hardware-verified milestones: **M01–M25 + M27** (well ahead of the M01–M05 figure still recorded in this repo's own `docs/ecosystem-roadmap.md`/`docs/language-core-axioms.md` — those are stale and due for an update once the next sync lands).
- `conformance.my` itself: not yet run on fpga-lisp (roadmap items 28-30 — GC, full REPL, full my-lisp conformance suite not started).
- **Open blocker:** no `letrec`/self-referential recursion in closures — blocks bootstrapping `length`/`reverse`/`append`/`map` from `core.my`. Task sent 2026-08-11; response pending.
- GC approach decided: mark-and-sweep/trace-based (not refcounting), now that `SETCDR` exists.
- Active work: `tb_cml_e2e.sv`, a general E2E harness for CML (arbitrary `.bin` via `+bin_file=`, prints `RESULT_TAG/VAL`, `RESULT_ERROR/PC`, full `HEAP` dump).

## cml

- `compatibility.my`: pins language-contract + ISA versions and tested SHAs (details not yet re-confirmed in this sync round).
- Tier-1 fixtures: **29/34** passing as of the last confirmed figure (2026-08-11); `let` now lowers to an immediately-invoked lambda, no new FPGA primitive needed.
- Remaining gaps: `equal?`, `defmacro`, three exactness/float fixtures — confirmed to be test-harness *filters* (`tests/conformance_test.rs:239-251`), not adapter changes. The compile→assemble→simulate adapter is already shared/unchanged across fixtures — First Blind Fixture's structural criterion is already met; what's left is removing the filter lines and adding compilation support for those forms.
- HEAP/RESULT decode format: confirmed to match fpga-lisp's `tb_cml_e2e.sv` output.
- Next after current work: pinned interface CI (`docs/ecosystem-roadmap.md` item 7), realistic once equal?/defmacro/exactness lands and fpga-lisp's `letrec` blocker clears.

## Open ecosystem-wide blocker

`letrec`/self-referential recursion in fpga-lisp closures — the one item every other piece of remaining work (cml's Tier-3-adjacent fixtures, pinned CI) is waiting behind, directly or indirectly.

## How to refresh this file

From any of the three repos' active sessions: use the CCD session-management message tool to ask the other two sessions the four-point status request from `docs/ecosystem-sync.md`, then update this file's relevant section(s) with the reply. Keep entries factual and dated — this file has no authority of its own, only whatever it accurately reflects.

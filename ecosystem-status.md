# Ecosystem status board

Persisted status snapshot for the three-repository ecosystem (`my-lisp`, `fpga-lisp`, `cml`), mirroring `cml`'s own `ecosystem-status.md`. Purpose: let any future session read current cross-repo state from a file first, instead of always paying a live cross-session message round-trip (`docs/ecosystem-sync.md` describes when the live channel is still worth using — mainly to *update* this file, not to replace it).

**Not the contract.** The actual compatibility contract lives in the versioned files: `language-contract.my` (this repo), `ISA.md` (`fpga-lisp`, not yet created), `compatibility.my` (`cml`). This file is a snapshot for humans/sessions to orient quickly; if it disagrees with those files, the versioned files win.

Last synced: 2026-08-11, via direct cross-session messages with all active CCD sessions in the ecosystem (`fpga-lisp`, `cml`, and — newly joined — `my-idea`).

## my-lisp (this repo)

- `language-contract.my`: version **1.0** (2026-08-11) — Tier 1 (CORE SEMANTICS) + Tier 2 (LANGUAGE CONTRACT). Bare integer literals no longer silently lose precision above 2^53.
- Exactness model (`Exactness::Exact`/`Inexact` as a value property): fully implemented, stable, no further semantic change planned.
- `equal?` (`lib/core.my`) and `defmacro` (Rust bootstrap kernel): both long-stable, nothing changing.
- Not currently blocking either other repo.
- Rust toolchain (`rustc`/`cargo` 1.97.1) newly installed on this machine (2026-08-11) — was previously missing entirely, which had also blocked `cml`'s own local verification. MSVC linker (Visual Studio Build Tools, C++ workload) install in progress at time of writing; once complete, `cargo test --workspace` becomes runnable here for the first time this session.
- New session joined the ecosystem: **`my-idea`** (the IDE project `my-lisp` was originally extracted from). Briefed on current state 2026-08-11. See `docs/my-idea-architecture-review.md` for a full architecture review the user wrote for `my-idea` — most relevant cross-repo point: `my-idea` depends on `my-lisp` through two independent paths (Cargo git dependency + git submodule for WASM) that are only manually kept in sync, a real drift risk once a fourth repository is in the loop.

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
- `equal?`: implemented (2026-08-11) as a compiled native subroutine (`src/compiler.rs`, `cml_equal`) — iterative worklist over a shared stack register (R11), not recursion, so it does **not** depend on fpga-lisp's `letrec` blocker. Skip removed from `tests/conformance_test.rs`. **Status: ready for review, not verified/merged** — no Rust toolchain was available in that session to run `cargo test` locally; blocked on machine verification.
- Remaining gaps: `defmacro` (next — needs a macro-expansion pass before compilation, larger effort), three exactness/float fixtures. Confirmed: the `equal?`/`defmacro`/exactness skips are test-harness *filters* (`tests/conformance_test.rs:239-251`), not adapter changes — the compile→assemble→simulate adapter is already shared/unchanged across fixtures, so First Blind Fixture's structural criterion is already met; what's left is removing filter lines and adding compilation support per form.
- HEAP/RESULT decode format: confirmed to match fpga-lisp's `tb_cml_e2e.sv` output.
- **CI status: none.** No `.github/workflows` exists in `cml` at all. Raised the question of standing up pinned interface CI (`docs/ecosystem-roadmap.md` item 7) earlier than planned, specifically because it's now the only way to machine-verify `equal?`/`defmacro`/exactness work — but declined to create CI/CD infrastructure unilaterally since it affects all three repos. **Awaiting user decision** (asked 2026-08-11, not yet answered).
- Next after current work: pinned interface CI, realistic once equal?/defmacro/exactness lands and fpga-lisp's `letrec` blocker clears.

## Open ecosystem-wide blockers

1. `letrec`/self-referential recursion in fpga-lisp closures — blocks `cml`'s Tier-3-adjacent fixtures indirectly and fpga-lisp's own `core.my` bootstrap directly.
2. No CI anywhere in the ecosystem yet — `cml`'s `equal?` work (and any future compiler change) has no machine-verified confirmation path on this developer's machine until either a Rust+MSVC toolchain is fully working locally, or pinned interface CI exists. User decision pending on standing up CI now vs. later.

## How to refresh this file

From any of the three repos' active sessions: use the CCD session-management message tool to ask the other two sessions the four-point status request from `docs/ecosystem-sync.md`, then update this file's relevant section(s) with the reply. Keep entries factual and dated — this file has no authority of its own, only whatever it accurately reflects.

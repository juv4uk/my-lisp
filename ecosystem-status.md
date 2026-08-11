# Ecosystem status board

Curated, current-state-only snapshot for the four-repository ecosystem (`my-lisp`, `fpga-lisp`, `cml`, `my-idea`). Read this first for "what's true right now." For the full chronological back-and-forth (who asked what, when, verbatim) see `cml`'s own `ecosystem-status.md` (`C:\Users\user\Documents\GitHub\cml\ecosystem-status.md`) — that file is the append-only sync **log**; this file is the derived **snapshot**. They used to duplicate each other; as of 2026-08-11 the roles are split on purpose so neither has to be read in full just to answer "what's the current state."

**Not the contract.** The actual compatibility contract lives in the versioned files: `language-contract.my` (this repo), `isa-contract.my` (`fpga-lisp`), `compatibility.my` (`cml`). This file is a snapshot for humans/sessions to orient quickly; if it disagrees with those files, the versioned files win.

Last synced: 2026-08-11, via direct cross-session messages with all active CCD sessions in the ecosystem (`fpga-lisp`, `cml`, `my-idea`), plus a direct read of `isa-contract.my`/`compatibility.my`/`cml`'s own status log.

## my-lisp (this repo)

- `language-contract.my`: version **1.0** (2026-08-11) — Tier 1 (CORE SEMANTICS) + Tier 2 (LANGUAGE CONTRACT). Bare integer literals no longer silently lose precision above 2^53.
- Exactness model (`Exactness::Exact`/`Inexact` as a value property): fully implemented, stable, no further semantic change planned.
- `equal?` (`lib/core.my`) and `defmacro` (Rust bootstrap kernel): both long-stable, nothing changing.
- Not currently blocking either other repo.
- Rust toolchain (`rustc`/`cargo` 1.97.1) newly installed on this machine (2026-08-11) — was previously missing entirely, which had also blocked `cml`'s own local verification. MSVC linker (Visual Studio Build Tools, C++ workload) install in progress at time of writing; once complete, `cargo test --workspace` becomes runnable here for the first time this session.
- `my-idea` joined the ecosystem sync (2026-08-11). See `docs/my-idea-architecture-review.md` for the user's full architecture review — most relevant cross-repo point: `my-idea` depends on `my-lisp` through two independent paths (Cargo git dependency + git submodule for WASM) that are only manually kept in sync, a real drift risk once a fourth repository is in the loop.

## fpga-lisp

- ISA contract (`isa-contract.my`): version **0.2** — 32-bit word (4 tag bits + 28 payload bits), 6 tags (`fixnum`/`cons`/`symbol`/`nil`/`true`/`primitive`), 16 opcodes, 6 encoded-modes (`gettag`/`makeprim`/`getval`/`setcdr`/`call`/`ret`, reusing `mov`/`atom`/`jmp` rather than adding new opcodes), 6 primitive-ids (`car`/`cdr`/`cons`/`atom`/`eq`/`add`), 16 registers (R0=args, R4=env, R11=stack, R14=link, R15=value).
- Hardware-verified milestones: **M01–M25 + M27** (well ahead of the M01–M05 figure still recorded in this repo's own `docs/ecosystem-roadmap.md`/`docs/language-core-axioms.md` — those are stale and due for an update once the next sync lands).
- `conformance.my` itself: not yet run on fpga-lisp (roadmap items 28-30 — GC, full REPL, full my-lisp conformance suite not started).
- **Open blocker:** no `letrec`/self-referential recursion in closures — blocks bootstrapping `length`/`reverse`/`append`/`map` from `core.my`. Task sent 2026-08-11; response pending.
- GC approach decided: mark-and-sweep/trace-based (not refcounting), now that `SETCDR` exists.
- Active work: `tb_cml_e2e.sv`, a general E2E harness for CML (arbitrary `.bin` via `+bin_file=`, prints `RESULT_TAG/VAL`, `RESULT_ERROR/PC`, full `HEAP` dump).
- **Structural gap found 2026-08-11 (reading `isa-contract.my` directly):** the ISA has no inexact-number tag, no rational-number representation, and no dedicated string tag. This is far more than "a couple of unimplemented fixtures" — my-lisp's exact rational arithmetic is stated as a *core purpose*, not a nice-to-have (`docs/language-core.md`), and it currently has no representation on fpga-lisp's ISA at all. `cml` has already asked fpga-lisp this exact question in its own status log and has not yet received an answer.

## cml

- `compatibility.my`: compiler `0.1.0`, pins `my-lisp@ed10151` (contract `(1 0)`) and `fpga-lisp@01bb01a` (ISA `(0 2)`), pipeline `parse → compile → assemble → simulate → compare`.
- Tier-1 fixtures: **29/34** passing (confirmed directly in `compatibility.my`'s `tier-1-fixtures-executed`); `let` now lowers to an immediately-invoked lambda, no new FPGA primitive needed.
- `equal?`: implemented (2026-08-11) as a compiled native subroutine (`src/compiler.rs`, `cml_equal`) — iterative worklist over a shared stack register (R11), not recursion, so it does **not** depend on fpga-lisp's `letrec` blocker. Skip removed from `tests/conformance_test.rs`. **Status: ready for review, not verified/merged** — no Rust toolchain was available in that session to run `cargo test` locally; blocked on machine verification.
- Remaining gaps: `defmacro` (next — needs a macro-expansion pass before compilation, larger effort), three exactness/float fixtures. Confirmed: the `equal?`/`defmacro`/exactness skips are test-harness *filters* (`tests/conformance_test.rs:239-251`), not adapter changes — the compile→assemble→simulate adapter is already shared/unchanged across fixtures, so First Blind Fixture's structural criterion is already met; what's left is removing filter lines and adding compilation support per form.
- HEAP/RESULT decode format: confirmed to match fpga-lisp's `tb_cml_e2e.sv` output.
- **Documented limitations, from `compatibility.my` directly:** `no-inexact-numbers`, `no-rationals`, `no-canonical-strings` (source strings are lowered to target symbols as a stopgap representational substitution — not a real string type), `no-more-than-eight-call-arguments`.
- **CI status: none.** No `.github/workflows` exists in `cml` at all. Raised the question of standing up pinned interface CI (`docs/ecosystem-roadmap.md` item 7) earlier than planned, specifically because it's now the only way to machine-verify `equal?`/`defmacro`/exactness work — but declined to create CI/CD infrastructure unilaterally since it affects all three repos. **Awaiting user decision** (asked 2026-08-11, not yet answered).
- Next after current work: pinned interface CI, realistic once equal?/defmacro/exactness lands and fpga-lisp's `letrec` blocker clears.

## Open ecosystem-wide blockers

1. `letrec`/self-referential recursion in fpga-lisp closures — blocks `cml`'s Tier-3-adjacent fixtures indirectly and fpga-lisp's own `core.my` bootstrap directly.
2. No CI anywhere in the ecosystem yet — `cml`'s `equal?` work (and any future compiler change) has no machine-verified confirmation path on this developer's machine until either a Rust+MSVC toolchain is fully working locally, or pinned interface CI exists. User decision pending on standing up CI now vs. later.
3. **No inexact numbers, no rationals, no dedicated string type on fpga-lisp's ISA** — a representational gap, not an implementation gap. Until ISA 0.2 (or a successor) adds tags/encoding for these, `cml` cannot compile any my-lisp program using floats, exact rationals, or real string operations for the FPGA target, and `docs/language-core.md`'s stated core purpose (exact rational arithmetic) has no path to hardware. `cml` asked fpga-lisp about this in `cml/ecosystem-status.md`; unanswered as of this sync.

## How to refresh this file

From any of the four repos' active sessions: use the CCD session-management message tool to ask the other sessions the four-point status request from `docs/ecosystem-sync.md`, then update this file's relevant section(s) with the reply. Keep entries factual and dated. For the raw exchange itself, append to `cml/ecosystem-status.md` instead — that's the log; this file only ever holds the current summary, never conversation history.

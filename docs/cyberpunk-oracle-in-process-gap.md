# CI oracle ≠ in-game evaluator — naming the gap (CP-ORACLE-IN-PROCESS-GAP)

Written per this repo's own task recommendations
(`docs/cyberpunk-task-recommendations-2026-09-10.md`, priority 8.0),
now updated with a real data point: `my-lisp-cyberpunk#1`'s
`CP-IN-GAME-LOAD-WITNESS` (2026-09-10) proved a real Cyberpunk 2077 +
RED4ext process running `(запиши-лог)` and logging `()`, oracle-matched
by hand against `docs/cyberpunk-host-dispatch-fixtures.md`. This note
names the gap that single proof does not close.

## What every other substrate in this ecosystem gets for free

`crates/my-lisp`'s own CLI binary (`my-lisp.exe --oracle-check`) *is*
the oracle. When CI runs a conformance fixture against native my-lisp,
it is running the literal artifact that would also run in production,
because the CLI has no separate "shipped" build distinct from the one
CI already exercised. WASM inherits the same property — the WASM
binary CI builds and tests is the WASM binary that ships. `wsm-my-lisp`
achieves the same guarantee a harder way: its asm nucleus is
oracle-verified fixture-by-fixture against the real my-lisp CLI
(`--oracle-check`), and because it's a from-scratch reimplementation
rather than a build variant, "the tested thing is the shipped thing"
holds by direct comparison rather than by being literally the same
binary.

## Where Cyberpunk breaks that property

The DLL RED4ext loads into the game process (`wsm_my_lisp_cyberpunk_dll.dll`,
per the load-witness transcript) is built once, in CI or locally, and
then *copied into a game installation the CI machine does not run*.
Nothing currently proves that the exact bytes loaded by RED4ext during
the real smoke test are the exact bytes CI's fixture suite most
recently validated — the connection is "we built it, then we manually
copied it, then we manually ran the game and eyeballed the log," not a
mechanical chain. For native CLI and WASM, that chain is either
trivial (same binary) or mechanically checked (`--oracle-check`
against a running process). For the in-game DLL, today it is neither —
it is a human remembering to rebuild before copying.

This is not hypothetical risk-aversion: the smoke test transcript
itself is proof the manual path works *once*, under a human's direct
attention. The gap is about what happens on the tenth iteration, after
a fixture changes, without anyone specifically remembering to rebuild
and re-copy before the next launch.

## Two concrete options (either is enough; both is stronger)

1. **Hash-pin the loaded artifact.** Have the adapter's `Query`/`Main`
   export log a hash of its own DLL bytes (or of the embedded my-lisp
   core it was built against) at load time — visible in the same
   RED4ext log the smoke test already captured. CI computes the same
   hash for the DLL it just built and tested. A human (or eventually a
   script) diffs the two hashes before trusting a given in-game session
   as representative of what CI validated. Cheap, no new test
   infrastructure, but still a manual diff step.
2. **A shared fixture runner the DLL itself can execute.** Expose one
   additional adapter-registered capability (read-only, matching the
   MVP's own "no mutation yet" scope) that evaluates a fixed set of
   `docs/cyberpunk-host-dispatch-fixtures.md`'s own expressions
   in-process and logs pass/fail, callable once at plugin load. This
   makes the in-game log itself carry oracle evidence, not just a
   "the plugin loaded" confirmation — closer to what
   `--oracle-check` gives native builds, though still short of true
   CI-in-the-game-process.

Recommendation: start with option 1 (a load-time hash log line) since
it is nearly free and directly answerable with what the adapter
already does (it already logs at load, per the smoke-test transcript).
Treat option 2 as the next step only if a real discrepancy is ever
found or suspected — building fixture-runner infrastructure
speculatively would be solving a problem not yet observed, the same
overreach this doc's sibling notes have been careful to avoid.

## What this note does not do

It does not propose CI itself launch a real Cyberpunk process — that
requires a licensed game install and is out of scope for any CI
runner this ecosystem controls. The gap named here is permanent
(a game process cannot become a CI artifact), not a temporary
oversight; the goal is making the gap visible and cheaply checkable
each time, not eliminating it.

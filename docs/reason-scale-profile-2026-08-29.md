# MYLISP-REASON-SCALE-PROFILE — measured 2026-08-29

Status: HISTORICAL MEASUREMENT + CONFIRMED FOLLOW-UP. The timings and crash
below describe the pre-2026-09-07 implementation and remain evidence of the
failure that motivated the repair. They must not be read as the current state.

## What was measured

- Engine: `lib/reason.my` — naive backward-chaining, no indexing.
- Query shape: `(reason (list (quote edge) (logic-var (quote x)) <n>) chain)`
  — asks for a predecessor of the last node, so only the final `(edge (n-1) n)`
  fact matches **after scanning all n rules**. This is the no-index **worst
  case** (full forward scan).
- Method: the chain is pre-built (untimed); only the `reason` call is inside
  the timed region. Wall-clock via Rust `std::time::Instant`.
  Harness committed: `crates/my-lisp/tests/reason_scale.rs`.
- Build: debug (unoptimized). Machine: shared WSL node (noisy clock).

### Original timings (full-scan goal, 64 MiB measurement stack)

|   N | run 1 (ns) | run 2 (ns) | run 3 (ns) |
|----:|-----------:|-----------:|-----------:|
| 100 | 274,727,709 | 224,956,481 | 362,388,211 |
| 500 | 1,549,203,743 | 1,553,311,165 | 1,768,982,843 |
|1000 | 3,288,982,967 | 2,713,292,121 | 1,922,079,680 |

Roughly: N=100 → ~0.23–0.36 s, N=500 → ~1.5–1.8 s, N=1000 → ~1.9–3.3 s.

### Original finding 1 — superlinear scan construction

The old `prove-goal` rebuilt the result with `append` around a recursive scan.
That both repeatedly copied result prefixes and kept the recursive call out of
tail position. Other proof-building paths can still have their own costs; this
report never proved a global complexity bound for the entire reasoner.

### Original finding 2 — stack overflow before N=100 on default stack

The old rule scan consumed O(number-of-rules) call-stack depth and overflowed
the default 2 MiB Rust test-thread stack before/around N=100. The original
timings therefore required an explicit 64 MiB measurement thread.

---

## Follow-up — 2026-09-07

The rule scan has now been repaired without changing `reason`'s public result
shape or result order.

Current shape:

```text
rules
  ↓
tail-recursive scan
  ↓
cons successful results onto reversed accumulator
  ↓
one reverse at completion
```

Executable evidence:

- `crates/my-lisp/tests/reason_stack.rs`
  - `full_scan_256_rules_is_stack_safe_on_default_thread`
  - `tail_scan_preserves_rule_result_order`
- CI run #1007 passed workspace tests/build and zero-warning clippy with those
  regressions on the ordinary test-thread stack.
- The pre-existing N=100/500/1000 profiling harness also continued to pass
  after the repair; no new stable per-N timing claim is recorded here because
  the shared-runner wall clock is not a contract and the successful CI log did
  not expose a repeatable benchmark series suitable for comparison.

### Current epistemic status

- **confirmed:** the formerly crashing full rule scan is stack-safe at 256 rules
  on the ordinary CI test thread.
- **confirmed:** rule-result order is preserved by the accumulator rewrite.
- **confirmed:** the old `append(... recursive-scan ...)` construction is gone.
- **unknown:** an implementation-independent asymptotic bound for every
  reasoning path; this repair only establishes the rule-scan property above.
- **open performance question:** predicate/head indexing may still reduce a
  fixed-predicate query from scanning all rules to scanning only candidates,
  but it is now a performance optimization to justify by measurement, not a
  prerequisite for fixing the proven stack crash.

The original broken result remains above because a repaired failure is still
valuable evidence: it records what experiment falsified the old implementation
and what the regression test must prevent from returning.

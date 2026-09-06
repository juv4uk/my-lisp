# MYLISP-REASON-SCALE-PROFILE — measured 2026-08-29

Status: HISTORICAL MEASUREMENT + CONFIRMED FOLLOW-UP. The timings and crash
below describe the pre-2026-09-07 implementation and remain evidence of the
failure that motivated the repair. They must not be read as the current state.

## What was measured originally

- Engine: `lib/reason.my` — naive backward-chaining, no indexing.
- Query shape: `(reason (list (quote edge) (logic-var (quote x)) <n>) chain)`
  — asks for a predecessor of the last node, so only the final `(edge (n-1) n)`
  fact matches **after scanning all n rules**. This is the no-index **worst
  case** (full forward scan).
- Method: the chain was pre-built (untimed); only the `reason` call was inside
  the timed region. Wall-clock via Rust `std::time::Instant`.
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
That repeatedly copied result prefixes and kept the recursive call out of tail
position. Other proof-building paths can still have their own costs; this
historical profile never proved a global complexity bound for the whole
reasoner.

### Original finding 2 — stack overflow before N=100 on default stack

The old rule scan consumed O(number-of-rules) call-stack depth and overflowed
the default 2 MiB Rust test-thread stack before/around N=100. The original
timings therefore required an explicit 64 MiB measurement thread.

---

## Follow-up — 2026-09-07

The rule scan was repaired without changing `reason`'s public result shape or
result order:

```text
rules
  ↓
tail-recursive scan
  ↓
cons successful results onto reversed accumulator
  ↓
one reverse at completion
```

Executable evidence before the scale-harness refresh:

- `crates/my-lisp/tests/reason_stack.rs`
  - `full_scan_256_rules_is_stack_safe_on_default_thread`
  - `tail_scan_preserves_rule_result_order`
- CI #1007 passed workspace tests/build and zero-warning clippy.

### Scale harness refreshed after the repair

`crates/my-lisp/tests/reason_scale.rs` no longer carries the old failure into
its own implementation:

1. the normal N=100/500/1000 profile runs directly on the ordinary test-thread
   stack — there is no custom 64 MiB stack;
2. the edge-chain fixture is emitted from Rust as quoted Lisp data, so fixture
   construction itself does not add a recursive Lisp stack cost to the setup;
3. wall-clock values are printed for diagnosis only and are never asserted;
4. a separate ignored N=5000/10000 profile exists for deliberate manual runs
   before an indexing decision, so ordinary CI does not acquire a large timing
   tax.

Manual extended run:

```text
cargo test -p my-lisp --test reason_scale reason_scale_profile_extended -- --ignored --nocapture
```

No new per-N timing numbers are promoted to this document merely because a
shared CI runner completed the test. A timing claim should be recorded only
from an intentional captured profile with enough repeated runs to distinguish
signal from runner noise.

### Current epistemic status

- **confirmed:** the formerly crashing rule scan is stack-safe at 256 rules on
  the ordinary CI test thread;
- **confirmed:** rule-result order is preserved by the accumulator rewrite;
- **confirmed:** the old `append(... recursive-scan ...)` construction is gone;
- **implemented, CI pending for refreshed harness:** ordinary N=100/500/1000
  scale profiling no longer requests a larger stack and isolates fixture
  construction from the timed reason call;
- **available for manual falsification:** N=5000/10000 default-stack full scans;
- **unknown:** an implementation-independent asymptotic bound for every
  reasoning path;
- **open performance question:** predicate/head indexing may still reduce a
  fixed-predicate query from scanning all rules to scanning only candidates,
  but it is a performance optimization to justify by measurement, not a
  prerequisite for stack safety.

The original broken result remains above because a repaired failure is still
valuable evidence: it records what experiment falsified the old implementation
and what the regression must prevent from returning.

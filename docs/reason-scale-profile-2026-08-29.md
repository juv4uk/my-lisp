# MYLISP-REASON-SCALE-PROFILE — measured 2026-08-29

Status: CONFIRMED. Real measured timings of `lib/reason.my` over a linear
`edge` chain at N = 100 / 500 / 1000, and a stack-overflow finding.

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

### Timings (full-scan goal, 64 MiB measurement stack)

|   N | run 1 (ns) | run 2 (ns) | run 3 (ns) |
|----:|-----------:|-----------:|-----------:|
| 100 | 274,727,709 | 224,956,481 | 362,388,211 |
| 500 | 1,549,203,743 | 1,553,311,165 | 1,768,982,843 |
|1000 | 3,288,982,967 | 2,713,292,121 | 1,922,079,680 |

Roughly: N=100 → ~0.23–0.36 s, N=500 → ~1.5–1.8 s, N=1000 → ~1.9–3.3 s.

### Finding 1 — superlinear (≈O(N²)) time

10x facts (100→1000) costs ~6–12x time. Not linear. Cause: `prove-goal`
(`lib/reason.my:55-61`) rebuilds the result with `append` around a recursive
scan, and `prove-rule`/`prove-goals` rebuild proof lists at every level.

### Finding 2 — stack overflow before N=100 on default stack

`prove-goal` recurses over rules via **non-tail** recursion (the recursive
`prove-goal` call is an argument to `append`, not in tail position), so a
full-scan goal consumes **O(N) call-stack depth**. The same query
**stack-overflows the default 2 MiB test-thread stack already at N=100**
(measured: the harness aborted on first run). Timings above required an
explicit 64 MiB measurement stack.

## Conclusion / decision

The earlier review phrase *"unworkable at ~1000 facts"* was **optimistic**:
a single full-scan query over `reason` is (a) seconds-at-best at N=1000 and
(b) crashes the interpreter on a default stack well before N=100. Indexing
is **clearly warranted**, and the recursion in `prove-goal` should also be
made tail-recursive (accumulator + reverse, preserving match order) since the
catastrophic stack blowup is the more urgent failure mode (correctness/crash)
than the quadratic time.

Recommended next step (NOT done here — task was measurement only):
1. Make `prove-goal` tail-recursive (fixes O(N) stack depth / crash).
2. Add head-indexing on goal predicate to avoid full scans for fixed-predicate
   goals (fixes O(N) -> O(k) where k = matching rules).
3. Re-run this harness (tests/reason_scale.rs) to confirm the improvement.

## Evidence artifacts

- `crates/my-lisp/tests/reason_scale.rs` — reproducible harness (asserts
  functional correctness; prints timings to test log).
- This report — recorded measured numbers.

CONFIRMED / BROKEN (stack overflow on default 2 MiB stack, full-scan goal, N<100) /
UNRESOLVED (post-fix improvement — not attempted in this task).

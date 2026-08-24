# my-lisp benchmarks · Benchmarks my-lisp · my-lisp-Benchmarks

**Status:** CURRENT · regenerated 2026-08-24 · base `1566fcf`+ · Vyasa
**Runner:** `cargo run --release -p my-lisp --example benchmark`
(env var `MY_LISP_BENCH_ITERATIONS`, default 1000; `MY_LISP_RAT_N` for the rational chain depth)

> ⚠️ Machine-specific numbers. Compare runs from the same machine only.
> These are microbenchmarks and a workload probe — not product promises.
> Числа специфічні для машини; порівнюйте тільки рани з однієї машини.

## 1. Fresh-session microbenchmarks (cold path)

Every operation pays `Session::default()` + full `lib/core.my` parse —
this matches one-shot CLI/batch invocations (WSM-24 style egg farms),
which is exactly why optimization item #4 (AST snapshot) targets it.

Measured 2026-08-24, `MY_LISP_BENCH_ITERATIONS=200`:

| case | ns/op | note |
|---|---|---|
| rust/parser | 23 511 | parse-only |
| rust/arithmetic | 31 392 | incl. session+core.my |
| rust/lists | 48 746 | |
| rust/recursion | 88 747 | |
| rust/closures | 24 371 | |

## 2. Warm-session steady state (hot path) — NEW

Session loaded once (`core.my` + setup); measures pure interpreter loop
through `eval_program`. This is what LSP / REPL / swarm-node actually
experience after startup.

| case | per call | what it exercises |
|---|---|---|
| warm/rational-chain-100 | **~320 ms** | 100 exact-rational additions; denominator = LCM chain of 100 coprime-ish terms → multi-thousand-digit bignums, gcd on every step |
| warm/vector-fill-500 | ~89 ms | 500× vector-set!/vector-ref through Rc<RefCell<Vec>> |

### The finding that drives the next optimization

The pre-Karatsuba rational-chain baseline scaled **superlinearly**: n=100 → 0.32 s,
n=200 → 3 s, n=400 → 42 s (CLI repro), n=2000 → >5 min.
Root cause: each `(+ acc term)` normalizes against an ever-growing LCM
denominator; gcd cost grows with digit count. This is the single most
expensive real-workload shape we know of (WSM-24 chamfer is built from
exactly these chains). Optimization options are tracked in
`docs/OPTIMIZATION-ANALYSIS-VYASA.md` §1. These historical numbers are not
used as a causal speedup claim after the Karatsuba change.

## 3. Karatsuba post-implementation probe (2026-08-24)

After `df1c333`, a release build was run with
`MY_LISP_BENCH_ITERATIONS=1` (the rational case still takes its minimum 50
warm repetitions), `nice -n 10`, `ionice -c2 -n7`, and a 120–180 second
timeout per run. The machine reported load 2.09/4.36/5.02 before the probe.
These are current-machine observations, not an A/B speedup claim:

| `MY_LISP_RAT_N` | warm rational-chain per call |
|---:|---:|
| 100 | 11.18 ms |
| 200 | 49.82 ms |
| 400 | 202.14 ms |

The harness still labels the row `rational-chain-100`; depth is controlled by
`MY_LISP_RAT_N`. A same-build schoolbook-vs-Karatsuba A/B harness is still
needed before claiming a percentage improvement.

## 4. Historical note

The suite historically also ran the same `.my` programs through the
ClojureScript prototype (`npm run benchmark`). The CLJS prototype has
been fully replaced by the Rust core and `package.json` no longer
exists — those instructions were dead and are removed. The `.my`
programs in `benchmarks/*.my` remain as fixture inputs to this runner.

## Українська

Запуск: `cargo run --release -p my-lisp --example benchmark`.
Секція 1 — cold path (кожна операція платить парсинг core.my — як раз
батчові CLI-запуски). Секція 2 — тепла сесія (стале навантаження
інтерпретатора, як у LSP/REPL/swarm після старту). Головний висновок:
точнорaціональний ланцюг масштабується надлінійно через ріст
LCM-знаменника — це ціль наступної оптимізації.

## 4. Cross-language boundary — exact rationals vs Python (2026-08-24)

**Workload:** LCM-chain `acc += k²/(3k+1)`, k=n..1, exact fractions.
**Engines:** my-lisp release CLI (Stein GCD+Karatsuba, post-revert d8594c1)
vs CPython `fractions.Fraction` (C math.gcd).
**Protocol:** 5 runs each, nice -n 10 ionice -c2 -n7, load 1.46, same machine;
median reported; parity = numerator/denominator prefix match n=200.

| n | my-lisp | python | ratio |
|---|---|---|---|
| 100 | 0.32s | 0.03s | ~10× |
| 200 | 2.9s | 0.03s | ~95× |
| 400 | 32.4s | 0.04s | ~900× |

Memory: my-lisp ~2.8MB vs python ~11.5MB RSS (**my-lisp 4× leaner**).
Parity: PASS (identical fraction heads).

**Boundary verdict [honest]:** для exact-раціональних обчислень CPython
перемагає у стіну на глибоких ланцюгах (C gcd/mul); my-lisp виграє памʼяттю
та гарантує exactness контрактом. Виправдана межа: числові батчі залишаються
Python-bootstrap до Karatsuba-class+алгоритмічних покращень; optimization
targets записані (bignum mul/gcd steady-state).

Див. також fpga-lisp@cf48fd0 (text-processing boundary: python 0.89s vs
my-lisp 14s cold; startup НЕ вузьке місце — рекурсивний string-traversal є,
6c2e024) — разом ці два фікстури окреслюють поточні чесні межі мови.

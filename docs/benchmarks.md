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

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

The rational chain scales **superlinearly**: n=100 → 0.32 s,
n=200 → 3 s, n=400 → 42 s (CLI repro), n=2000 → >5 min.
Root cause: each `(+ acc term)` normalizes against an ever-growing LCM
denominator; gcd cost grows with digit count. This is the single most
expensive real-workload shape we know of (WSM-24 chamfer is built from
exactly these chains). Optimization options are tracked in
`docs/OPTIMIZATION-ANALYSIS-VYASA.md` §1.

## 3. Historical note

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

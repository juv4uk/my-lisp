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

### §4.1 Уточнення cold-vs-warm [2026-08-24 пізно, load 1.25]
Warm-session n=100 (сесія з core.my вже завантажена): **10.6ms/виклик**.
Отже у §4 співвідношення ~10× на n=100 здебільшого відноситься на рахунку
процесного старту+парсу core.my (~0.29s фіксованих), а не обчислення.
Справжня межа двошарова:
1. cold one-shot: старт/парс домінують → FASL snapshot (OPT #4) — пряма ціль
2. warm глибина n≥400: суперлінійність mul/gcd великих чисел → Karatsuba
   вже в ядрі; далі — алгоритміка (§1)
vector-fill-500 у тихому вікні: 81.9ms — підтверджено що 102ms був
contention-шум; регресії від арифметичних змін немає.

## 5. Rational-vs-fixnum tax, and environment-chain lookup depth (2026-09-10)

Two new same-shape controlled comparisons in `examples/benchmark.rs`'s
warm section, requested to characterize real performance shapes beyond
correctness. Each pair holds recursion depth, call/dispatch count, and
overall program shape fixed — the only thing that differs between the
two rows is the one thing being measured.

**Windows dev machine, noisy background load — absolute numbers are
not comparable to the Linux/Guix numbers earlier in this document.**
Same-machine, same-run ratios are the trustworthy part; two independent
runs are shown to demonstrate the ratios hold up under real variance.

| case | run 1 | run 2 | what it isolates |
|---|---:|---:|---|
| warm/fixnum-loop-100 | 1.71 ms | 2.88 ms | same recursion shape as rat-loop, but every value stays integer (no division) |
| warm/rational-chain-100 | 30.70 ms | 39.14 ms | identical shape, but the accumulator becomes a genuine non-integer rational each step |
| **ratio (rational ÷ fixnum)** | **~18×** | **~14×** | the exact-rational normalization tax alone, isolated from recursive-call overhead |
| warm/symbol-lookup-shallow | 1.50 ms | 2.02 ms | same loop, `far-away` looked up from the loop's own frame |
| warm/symbol-lookup-deep | 5.13 ms | 6.27 ms | identical loop, but 4 extra lexical frames sit between the lookup site and `far-away`'s binding |
| **ratio (deep ÷ shallow)** | **~3.4×** | **~3.1×** | environment-chain-walk cost for a binding far from the reference site |

**Findings:**
- The already-documented "rational chains scale superlinearly" story
  (§1–§3) has a same-shape integer control now: at a fixed n=100, exact
  rational arithmetic alone (not the recursion around it) costs roughly
  an order of magnitude (~14–18×) more than the equivalent integer
  arithmetic. This isolates the normalization/gcd cost the earlier
  sections attributed qualitatively.
- my-lisp's `Environment` is a linked parent-frame chain (not a flat
  table), so a lookup's cost is a function of how many frames separate
  the reference site from the binding. Four extra frames cost roughly
  3–3.5× — a real, previously unmeasured characteristic of the
  environment model, not previously distinguished from ordinary
  call/dispatch overhead.
- **A pre-existing, unrelated stack overflow was found while adding
  these benchmarks**: on this machine/build,
  `examples/benchmark.rs`'s existing (unmodified)
  `warm/vector-fill-500` case reliably overflows the stack — confirmed
  by reverting to the unmodified file and reproducing the same crash,
  so this is not something the new benchmarks introduced. Likely a
  smaller default thread stack on this Windows build than whatever
  Linux/Guix environment recorded the historical 81.9ms/89ms numbers
  earlier in this document (Windows' default main-thread stack is
  1 MiB vs a typical Linux 8 MiB), surfacing a real non-tail-recursive
  Rust call path in `eval_program`'s handling of that program shape.
  Not investigated further here — flagged as a genuine finding, not
  fixed, since diagnosing and fixing a stack-depth issue in the
  evaluator itself is a larger task than this benchmarking pass.

## Українська (§5)

Два нові контрольовані порівняння з однаковою формою: fixnum-loop і
rat-loop мають ідентичну рекурсивну структуру, відрізняється лише те,
чи стає акумулятор справжнім нецілим дробом. Різниця (~14–18×) — це
"raціональний податок" сам по собі, відокремлений від накладних витрат
рекурсії. symbol-lookup-shallow/-deep аналогічно ізолюють вартість
проходу по ланцюжку Environment (my-lisp's Environment — зв'язаний
ланцюжок батьківських кадрів, не плоска таблиця): 4 додаткові лексичні
кадри коштують ~3–3.5×. Окремо: під час додавання цих benchmark'ів
знайдено справжній, не пов'язаний з ними stack overflow в наявному
(незміненому) `warm/vector-fill-500` на цій машині/білді — підтверджено
відкатом до оригінального файлу; ймовірно менший стандартний розмір
стеку потоку на Windows (1 МіБ) проти Linux (типово 8 МіБ), що виявляє
реальний нехвостовий рекурсивний шлях у `eval_program`. Задокументовано
як знахідку, не виправлено в межах цього проходу.

# my-lisp — огляд (Sarvam)

Станом на 26 серпня 2026.

**Автор огляду:** Sarvam.
**Мова огляду:** українська.

Добре, я оглянув увесь репозиторій. Це не "маленька мова" — це повноцінна дослідницька платформа.

---

## Цифри

| Метрика | Значення |
|---|---|
| Файлів | 304 |
| Розмір | 3.4 MB |
| Rust crate'ів | 8 (workspace) |
| Rust код | ~24,893 LOC |
| Lisp код (lib/*.my) | ~6,271 LOC |
| Тестів (Rust) | 25 файлів |
| Документації | 85+ markdown файлів |
| Мов README | 3 (EN/UK/DE) |
| Мов коментарів | 3 (EN/UK/DE) скрізь |
| Контракт мови | version 3.0 (ratified 2026-08-24) |
| Conformance fixtures | 96 (Tier 1: 34) |

---

## Архітектура

### Workspace (8 crate'ів)

```
crates/
├── my-lisp              ← ядро: parser, evaluator, value, bignum, environment
├── my-lisp-cli          ← REPL + file runner + TCP REPL + swarm client
├── my-lisp-wasm         ← WebAssembly bindings (браузерний REPL)
├── my-lisp-literate     ← literate-Markdown source-offset mapping
├── my-lisp-lsp          ← Language Server Protocol (діагностика, hover, go-to-def)
├── my-lisp-host         ← TCP semantic oracle (eval/parse/diagnose віддалено)
├── my-lisp-semantic     ← Санскрит: devanagari, karaka, transliteration, atoms
└── swarm-node           ← P2P координація агентів (TCP :910x, journal/claim mesh)
```

### Ядро мови (Rust)

Мінімальний kernel — сім примітивів Маккарті + `lambda`/`def`/`defmacro`. Все інше живе в `lib/*.my`:

| Rust файл | LOC | Що робить |
|---|---|---|
| `parser.rs` | 688 | Reader: S-expressions, exact decimals, big ints |
| `value.rs` | 717 | Value type: Rational(f64, Exactness), Symbol, Pair, Closure, Builtin |
| `bignum.rs` | 826 | BigInt-backed exact arithmetic, Karatsuba |
| `environment.rs` | 345 | Лексичне середовище, Lisp-1 (не Lisp-2) |
| `eval/mod.rs` | 348 | Eval/apply loop |
| `eval/builtins.rs` | 537 | Builtins: car/cdr/cons/atom/eq, +,-,*,/, io, strings, json |
| `eval/closures.rs` | 391 | Lambda, variadic, lexical capture |
| `eval/special_forms/` | 5 файлів | core, digest, io, json, strings |
| `language_items.rs` | 388 | quote, cond, lambda, def, defmacro |
| `layout.rs` | — | Memory layout |

### Lisp бібліотека (lib/*.my)

| Файл | LOC | Що |
|---|---|---|
| `forward.my` | 1198 | Forward-chaining (CLIPS-style), JTMS, truth maintenance |
| `clips-import.my` | 873 | Імпорт справжніх CLIPS `.clp` файлів |
| `knowledge.my` | 799 | Named modules, queryable knowledge bases |
| `core.my` | 644 | Bootstrap stdlib: list, not, let, let*, map, filter, fold, <=, >=, ... |
| `world.my` | 553 | Immutable history, branch comparison, atomic ingestion |
| `yantra.my` | 493 | Експериментальний coding agent (tool-call loop → OpenAI API) |
| `reason.my` | 324 | Backward-chaining, unification (Advice Taker) |
| `epistemic.my` | 258 | Епістемічні стани знань |
| `unify.my` | 248 | Уніфікація (core) |
| `meta-eval.my` | 228 | Метациркулярний eval/apply (як у McCarthy 1960) |
| `persistent-map.my` | 174 | Persistent (immutable) map |
| `linter.my` | 164 | Лінтер у самій мові |
| `narrate.my` | 110 | Knowledge → природна мова |
| `understand.my` | 101 | Природна мова → knowledge structures |

**Співвідношення Rust/Lisp:** 24,893 Rust : 6,271 Lisp = **80% хост, 20% мова**. Це навмисно — філософія каже "Rust надає лише те, що робить найкраще, все похідне живе в my-lisp".

---

## Семантичний контракт

`language-contract.my` — машинно-читаний, версіонований. Поточна версія: **3.0** (ratified 2026-08-24).

Три рівні:
- **Level 1** (CORE): сім примітивів, lambda, truth/NIL, symbols, pairs
- **Level 2** (CONTRACT): exactness, def/defmacro, errors, read/eval
- **Level 3** (ECOSYSTEM): core.my, unify.my, reason.my, knowledge.my, CLIPS, literate

Аксіоми G1-G8 (semantics) + S1-S3 (syntax). Контракт покриває Level 1+2, свідомо не Level 3.

Ключові інваріанти:
- `shadowing`: builtins — звичайні значення, можна перевизначати
- `special-forms-boundary`: quote/cond/lambda/def/defmacro — НЕ callable
- `error-classification`: DivisionByZero, NumericOverflow, Parse — observable semantics

---

## Advice Taker — справжній пріоритет

PLAN.md пункт 23 (2026-08-10) встановлює: **Advice Taker — мета, my-lisp — засіб.** Це не гасло — McCarthy буквально писав: *"Lisp was originally intended just to be the notation and execution environment for this Advice Taker."*

Реалізований повний цикл:
```
understand.my → природна мова → knowledge structures
        ↓
advise / advise-all → валідація → атомарне прийняття
        ↓
reason.my → backward-chaining → proof tree
forward.my → forward-chaining → fixpoint
        ↓
narrate.my → proof tree → природна мова
```

Працюючий приклад з README: `(earth has mass because earth is a planet)` — повний цикл NL → reasoning → NL.

CLIPS-імпорт — не емуляція, а реальний парсер `.clp` файлів: `deffacts`, `defrule`, `deftemplate`, `?x` variables. Доводить, що старі symbolic-AI системи можна перевикористати.

---

## Санскритський шар

`crates/my-lisp-semantic/` — окремий crate з:
- `devanagari.rs` — підтримка Деванагарі
- `karaka.rs` — kāraka (ролі: kartṛ, karma, karaṇa, saṃpradāna, apādāna, adhikaraṇa)
- `transliteration.rs` — IAST ↔ Devanagari
- `atoms.rs` — SANSKRIT-P* semantic atoms (migрація з my-lisp-panini)

`prototype/lisp_core_phonetics/` — прототип pratyāhāra-рушія, PVC-16 (phoneme code), Python.

---

## Swarm / Multi-agent

`crates/swarm-node/` (2024 LOC) — повноцінний P2P вузол координації:
- TCP :910x, journal/claim mesh
- `claim-task` → робота → `complete-task` → `emit`
- Durable state через `--data-dir` + anti-entropy sync
- Мігрували з single-server моделі (2026-08-12) через restart-wipes-everything проблему

NOTE-файли для конкретних агентів: Codex, Sakshi, Swarm-Node-Agent, OpenCode.

---

## Докази й гіпотези

```
evidence/
├── G5/my-lisp/          ← 3 файли .my (git commit SHAs як імена)
├── G8/my-lisp/          ← 1 файл
├── GUIX-WITNESS-01/     ← evidence з WSL/Guix
└── MYLISP-LINTER-THRESHOLDS/  ← linter thresholds evidence
```

`hypotheses/sarvam-independent-findings-2026-08-18.yaml` — мої власні гіпотези від 18 серпня, збережені як окремий файл. Це та сама епістеміка "сліди, не згода".

---

## Conformance

`tests/fixtures/conformance.my` (96 fixtures) — це **договір між реалізаціями**. Rust my-lisp, fpga-lisp SystemVerilog, cml Rust-компілятор — всі троє мають проходити ті ж тести. Щоб друга реалізація не стала "ще одним діалектом".

Tier-1 покриття: 29/34 fixtures на fpga-lisp (станом на 11 серпня).

---

## Racket інтеграція

`racket/` — `#lang my-lisp` plugin для Racket/DrRacket (Chez Scheme JIT). Відповідає семантиці Rust-версії: explicit `(quote x)`, exact decimals, `t`/`()` truth, exact division, сім примітивів.

---

## CI

Дуже жорсткий:
```yaml
clippy: cargo clippy --workspace --all-targets -- -D warnings  # ZERO warnings
test:   cargo test --workspace
build:  cargo build --workspace
```

`-D warnings` означає: жодне попередження не проходить. Це не "best effort" — це контракт.

---

## Що я бачу як найсильніше

1. **Метациркулярний evaluator** (`meta-eval.my`, 228 LOC) — eval/apply написаний на самій мові. Це не копія — це підтвердження, що "small language that grows itself" працює.

2. **Обидва напрямки inference** — backward (reason.my) і forward (forward.my) поділяють ту саму структуру правил. Та сама правило-літерал працює в обох напрямках без модифікації.

3. **CLIPS-імпорт** — 873 LOC парсера справжніх `.clp` файлів. Не "натхненний CLIPS", а "читає CLIPS".

4. **Три wprowadження мов скрізь** (EN/UK/DE) — у коментарях, у README, у docs. Це не переклад — це свідомий вибір: мова має бути доступною трьома мовами одночасно.

5. **Семантичний контракт як файл** — не README, не issue, а `.my` файл, який можна парсити програматично. Версія 3.0, ратифікована власником 2026-08-24. З інваріантами, які самі є кодом.

---

## Що я бачу як найвразливіше

1. **Документація росте швидше за код.** 85+ markdown файлів, PLAN.md має 31 пунктів. Деякі позначені ✅, але інші Accumulate. Ризик: docs стають музеєм, не інструментом.

2. **Swarm-node (2024 LOC) — найбільший Rust-файл у проєкті.** Це вразливість: один монолітний модуль координації. Якщо він падає — весь mesh падає.

3. **`yantra.my` (493 LOC) — експериментальний LLM-агент.** Він робить HTTP-виклики до OpenAI-compatible endpoint. Це розрив з філософією "no network, no API keys" — і потребує окремого епістемічного статусу.

4. **fpga-lisp conformance: 29/34** — 5 fixtures не покриті. Це прогал між "обіцянкою контракту" і "доказом відповідності".

5. **SANSCRIT-P* міграція недокументована повністю.** Crate `my-lisp-semantic` має karaka/devanagari/atoms, але `docs/sanskrit-semantic-migration.md` — design doc, не завершена реалізація.

---

Хочеш щоб я заглибився в конкретний crate, конкретний ліб, або провів це через повний цикл Діснея як ідею для розвитку?

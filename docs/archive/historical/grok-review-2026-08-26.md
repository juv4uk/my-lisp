# Повний огляд my-lisp (Grok)

Станом на 26 серпня 2026.

**Автор огляду:** Grok.
**Мова огляду:** українська.

---

## 1. Масштаб

| Метрика | Значення |
|---------|----------|
| Коміти | **708** |
| Мова | Rust (ядро) + `.my` (бібліотека) + Racket port |
| Language contract | **3.0** (ратифіковано 24 серпня) |
| Ліцензія | MIT |
| Тримовність | UA / EN / DE |

Це вже не «маленький інтерпретатор», а **повноцінна дослідницька мовна система** з reasoning-шаром, контрактами, другою реалізацією (FPGA) і tooling.

---

## 2. Що таке my-lisp по суті

**Одна структурна мова** для:

- програм
- фактів
- правил
- доведень

Ядро — сім примітивів Маккарті + мінімальний bootstrap:

```text
quote · atom · eq · car · cdr · cons · cond
+ lambda · def · defmacro
```

Усе інше росте **в my-lisp**, не в Rust.

Точна раціональна арифметика — **базова мета** (S1):

```text
(+ 0.1 0.2) → 3/10
(/ 5 6 8 7) → 5/336
```

Філософія явно підпорядкована:

> **Advice Taker — мета. my-lisp — засіб.**

---

## 3. Архітектура реалізації

### Rust crates

| Crate | Роль |
|-------|------|
| `my-lisp` | парсер, evaluator, середовища, exact rationals |
| `my-lisp-cli` | REPL + file runner + TCP oracle |
| `my-lisp-wasm` | browser REPL (persistent session + core.my) |
| `my-lisp-lsp` | LSP: diagnostics, hover, completion, arity |
| `my-lisp-literate` | literate Markdown |
| `my-lisp-semantic` | semantic layer |
| `my-lisp-host` | host bindings |
| `swarm-node` | координаційний вузол рою |

### Racket

`#lang my-lisp` на Chez Scheme — третя «поверхня» тієї ж семантики (не третя незалежна реалізація; C-ядро скасовано).

### Бібліотека (`lib/`)

| Файл | Зміст |
|------|--------|
| `core.my` + `.fasl` | стандартна бібліотека + snapshot |
| `meta-eval.my` | метациркулярний eval/apply |
| `unify.my` | уніфікація |
| `reason.my` | backward-chaining |
| `forward.my` | forward-chaining + JTMS |
| `knowledge.my` | іменовані модулі знань |
| `world.my` | незмінна історія, гілки, snapshot reasoning |
| `content-store.my` | content-addressed store |
| `understand.my` / `narrate.my` | контрольована NL ↔ структура |
| `clips-import.my` | імпорт справжніх CLIPS `.clp` |
| `epistemic.my` | proof-of-expression (opt-in) |
| `result-status.my` | unknown / partial / blocked / disputed |
| `linter.my` | лінтер |
| `yantra.my` | EXPERIMENTAL coding agent у my-lisp |

---

## 4. Контракти і дисципліна

**`language-contract.my` → 3.0**

- Покриває G1–G8, S1–S3
- ErrorKind тепер **спостережувана семантика**:
  - `DivisionByZero`
  - `NumericOverflow`
  - `Parse`
  - `InvalidForm` — лише для структурно невалідних форм
- Lexical shadowing builtins — ALLOWED (мінімум магії)
- Special forms (`quote`, `cond`, `lambda`, `def`, `defmacro`) — не callable values

**Сумісність з екосистемою:**

```text
(language-contract, ISA-contract) + точні SHA
```

не «усі репо однієї версії».

**Constitution** (`my-lisp-constitution.my`) — машинозчивані принципи + фікстури, прив'язані до аксіом.

**Evidence** — окремі записи на claim, не статус у чаті.

---

## 5. Reasoning-стек (Advice Taker)

Реалізовано наскрізно:

```text
understand → advise / advise-all → reason-in → narrate-answer
```

- data-only межа запису
- атомарні пакети
- explicit negative knowledge ≠ negation-as-failure
- JTMS (single- і multi-justification)
- один rule shape для forward і backward
- CLIPS import
- knowledge packages + TCP transport
- World + content-store

Це вже не «заготовка під AI», а **робочий символьний шар**.

---

## 6. Екосистемна роль

my-lisp = **semantic source of truth**.

```text
my-lisp  →  cml  →  fpga-lisp
   │
   ├─ my-idea (IDE / observer)
   └─ my-lisp-panini / shiva-sutras (дослідження, не runtime)
```

Відкритий gap (усе ще актуальний):

```text
my-lisp:   exact rationals ✓
fpga/cml:  no-rationals, no-inexact, no-string-tag
```

Це чесно зафіксовано як representational gap, не «майже вміємо».

---

## 7. Нещодавня активність (25–26 серпня)

Живий, інженерний темп:

- WASM: persistent session + preloaded `core.my`
- string-slice, `*argv*`, unicode/cyrillic/sanskrit тести
- LSP: arity diagnostics, hover docs, completion з runtime builtins
- Guix reproducibility tasks / witness
- FASL snapshot warnings
- закриття/реєстрація задач у `tasks.my`
- GOLD-SPIKE-PREP: 77 PASS на contract 3.0

Паралельно — багато **docs/tasks** комітів (аудит, triage, agent notes). Це вже знайомий патерн: код + мета-шар одночасно.

---

## 8. Сильні сторони

1. **Мінімальне ядро, реальне зростання в `.my`**
2. **Exact arithmetic як принцип, не фіча**
3. **Контракти замість «довіри до версії»**
4. **Повний reasoning pipeline**
5. **Пояснюваність** (proof trees, narrate)
6. **Друга реалізація (FPGA)** як контроль самообману
7. **Чесні статуси** (UNSUPPORTED, ORIENTATIONAL, evidence)
8. **Web REPL** без встановлення

---

## 9. Ризики і напруги

| Ризик | Як проявляється |
|-------|------------------|
| Архітектурна інфляція | docs/, NOTE-*, tasks, agent reactions ростуть швидше за ядро |
| Prototype bleed | phonetics / PVC-16 / yantra поруч із core path |
| Один автор | conformance перевірена зсередини, не зовнішнім тиском |
| Swarm overhead | координація може з'їдати час, призначений Lisp |
| FPGA lag | rational/string ще не на залізі — вертикаль семантично розірвана |
| PLAN.md величезний | 86KB історії — цінно, але важко як «що робити завтра» |

---

## 10. Вердикт

**my-lisp** — рідкісний solitary-проєкт, де:

- мінімалізм ядра реальний (не гасло);
- символьний AI не відкладений «на потім»;
- формальна дисципліна (аксіоми, contract, evidence) сильніша, ніж у більшості open-source мов такого розміру;
- задоволення від REPL (`89/55`, `3/10`, `sin` як дріб) збігається з заявленою філософією.

Водночас репозиторій **перевантажений контекстом**: агенти, critique, phonetics, swarm, gold spikes. Ядро сильне. Орбіта шумить.

Якщо стиснути до одного речення:

> Це робоча машина для символьного знання на чесному Lisp-ядрі — з ризиком втонути у власній екосистемній документації та prototype-променях.

Для тебе особисто найздоровіший центр усе той самий:

```text
cargo run -p my-lisp-cli
→ REPL
→ точні дроби, reason, advise
```

Усе інше — інфраструктура навколо цього.

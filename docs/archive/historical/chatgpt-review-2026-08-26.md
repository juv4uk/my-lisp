# my-lisp — repository-level audit (ChatGPT)

**Зріз:** `main` @ `a662dc76…`.
**Автор огляду:** ChatGPT.
**Мова огляду:** українська.

Друже, я справді пройшовся по репозиторію широко: поточний `main` на `a662dc76…`, повне дерево, workspace, канонічне ядро, evaluator, host boundary, бібліотеки `.my`, WASM/LSP/CLI/semantic/swarm crates, conformance/constitution, evidence, CI, testing doctrine й останню хвилю комітів. Це не "прочитав README і зробив висновок"; нижче саме repository-level аудит. Поточне дерево вже показує, що це невеликий репозиторій за розміром, але дуже широкий за функціональним охопленням.

## Загальний висновок

`my-lisp` перестав бути просто Lisp-інтерпретатором.

Я б зараз описав його так:

```text
my-lisp
│
├── language kernel
│   ├── reader/parser
│   ├── values
│   ├── lexical environment
│   ├── evaluator
│   ├── exact arithmetic
│   └── macros / closures
│
├── self-hosted language layer
│   └── lib/core.my
│
├── symbolic reasoning substrate
│   ├── unify
│   ├── reason
│   ├── forward
│   ├── JTMS
│   ├── knowledge
│   ├── world
│   ├── content-store
│   └── epistemic
│
├── language/tool surfaces
│   ├── CLI
│   ├── WASM
│   ├── LSP
│   ├── Racket #lang
│   └── literate source mapping
│
├── controlled host capabilities
│   ├── filesystem
│   ├── processes
│   └── TCP
│
├── semantic research
│   └── Sanskrit / kāraka / transliteration
│
├── distributed-agent infrastructure
│   └── swarm-node
│
└── scientific/conformance layer
    ├── language-contract.my
    ├── conformance fixtures
    ├── generated constitution
    ├── evidence/*
    └── CI
```

І це не моя реконструкція "з пам'яті": workspace зараз містить **8 Rust crates** — `my-lisp`, CLI, literate, WASM, swarm-node, host, semantic і LSP.

---

# 1. Архітектура ядра — дуже сильна

Найкраще рішення в усьому репо — те, що `crates/my-lisp` залишається **справді ядром**, а не "ядром, яке потроху стало ОС".

У core crate зараз **немає production dependencies взагалі**. `Cargo.toml` це підтверджує: `[dependencies]` порожній.

Сам код уже структурований досить природно:

```text
bignum.rs
environment.rs
error.rs
parser.rs
syntax.rs
value.rs

eval/
├ arithmetic.rs
├ builtins.rs
├ capabilities.rs
├ closures.rs
├ special_forms.rs
└ mod.rs
```

Це хороший поділ. Особливо мені подобається, що evaluator уже перестав бути одним величезним `eval.rs`: arithmetic, closures, builtins і special forms рознесені.

І фундаментальна межа сформульована чітко:

> Rust дає механізми; те, що мова вже може виразити сама, має рости над цією межею.

`lambda`, `def`, `defmacro` лишаються bootstrap-механізмами; похідні форми й стандартна бібліотека ростуть у `lib/core.my`. Це вже не лише філософія README — це language-core contract.

### Моя оцінка ядра

```text
semantic clarity       9/10
dependency discipline  10/10
bootstrap boundary      9/10
portability             9/10
performance maturity    6/10
```

Чому performance тільки 6 — нижче.

---

# 2. Exact arithmetic — це вже справжня мовна характеристика

Ти давно говорив, що exact rationals для тебе не "фіча". Тепер це реально вбудовано в ідентичність мови.

`Rational` спирається на власний arbitrary-precision `BigInt`, а не на `i64`. Документ прямо фіксує factorial 30 як exact результат за старою межею `i64`.

Особливо сильна річ — conformance contract уже перевіряє не лише очевидні дроби:

```lisp
(/ 5 6 8 7) → 5/336
(+ 0.1 0.2) → 3/10
1e-100       → exact rational
1e100        → exact integer
```

І перевіряє failure behavior (`NumericOverflow`, `DivisionByZero`, malformed literals) як частину семантики.

Це принципово сильніше за "наш evaluator підтримує BigInt".

У тебе вже фактично є аксіома:

```text
S1:
Never silently turn an exact value into an approximation.
```

і executable fixtures, що її підтримують.

### Але є реальна ціна

Власний BigInt — одночасно одна з найцікавіших і одна з найризикованіших частин.

Документація чесно фіксує, що 100k-element stack-safety workload після переходу на arbitrary precision виріс приблизно з ~6 до ~16 секунд.

Тобто зараз:

```text
correctness    ✅ дуже сильна
performance    ⚠ measurable cost
```

І я підтримую вже ухвалене рішення не вводити `i128` fast path просто "бо швидше". Найкращий наступний performance experiment колись — не ще один integer representation навмання, а **профілювання allocation/GCD cost**.

---

# 3. "A small language that grows itself" уже доведено набагато сильніше, ніж я очікував

Оце, мабуть, найбільше мене вразило після повного огляду.

`lib/` уже не маленька стандартна бібліотека. Там реально виріс другий поверх системи:

```text
core.my                ~36 KB
unify.my               ~14 KB
reason.my              ~18 KB
forward.my             ~69 KB
knowledge.my           ~43 KB
world.my               ~28 KB
epistemic.my           ~14 KB
meta-eval.my           ~15 KB
clips-import.my         ~54 KB
persistent-map.my      ~10 KB
...
```

Це саме my-lisp-код, не Rust.

І тут є кілька дуже сильних доказів self-growth:

**Metacircular evaluator.** `meta-eval.my` реалізує `eval/apply` мовою поверх власного ядра. README прямо позиціонує його як аналог відношення evaluator-а Маккарті до primitives.

**Unification + backward reasoning.** `unify.my` і `reason.my`.

**Forward reasoning.** `forward.my`, включно з fixpoint inference, `not/or/and/test`, `exists/forall`.

**Truth maintenance.** Є single і multi-justification JTMS.

**Knowledge modules.** `knowledge.my`.

**Immutable worlds/history.** `world.my`.

**Content-addressed knowledge.** `content-store.my`.

Це вже дуже далеко від "навчального Lisp".

---

# 4. Advice Taker напрямок став реальною підсистемою

Я раніше вважав symbolic-AI напрямок однією з найбільш перспективних частин проєкту. Після цього огляду я б сказав сильніше:

**це вже друга головна вісь репозиторію після самої мови.**

README описує один rule shape, який може використовуватись у backward і forward inference.

А `docs/testing.md` показує, наскільки далеко це зайшло:

* recursive backward rules;
* standardizing apart;
* negation as failure;
* proof trees;
* provenance;
* usage counting;
* modular knowledge;
* contradiction detection;
* retraction;
* forward fixpoint;
* JTMS;
* quantified `exists/forall`;
* real CLIPS imports;
* controlled language → knowledge;
* knowledge → narrative.

Тобто вже є практично повний цикл:

```text
text
↓
understand
↓
symbolic clauses
↓
knowledge/world
↓
unification
↓
backward / forward inference
↓
proof / provenance
↓
narrate
↓
text
```

Це дуже близько до тієї початкової ідеї, про яку ми говорили ще давно: **слово/текст як вхід до символічної структури**, але без удавання, що довільна природна мова вже "зрозуміла".

Саме це добре: `understand.my` називається controlled-natural-language bridge, а не AGI NLP.

---

# 5. CLIPS importer — несподівано сильний доказ

`clips-import.my` уже 54 KB, і тестова документація показує, що він перевірявся не лише на ручних toy fixtures.

Там згадані genuine external CLIPS programs, включно з `manners`, `dilemma1`, `mab`, `zebra`, Sudoku solver тощо. І що цікаво — саме реальні external files знаходили відсутні semantics (`exists/forall`, compound conditions) і навіть stack overflow.

Це дуже хороший науково-інженерний патерн:

```text
не:
"ми реалізували CLIPS importer"

а:
import real historical corpus
→ find mismatch
→ repair semantics
→ regression test
```

Я б це не недооцінював. Це один із найкращих proof-of-generality у репо.

---

# 6. Capability architecture дуже правильна — але поки грубозерниста

`my-lisp-host` винесений окремо від canonical core.

Його модуль буквально каже:

```text
core без install()
→ немає filesystem
→ немає process
→ немає TCP
```

Спроба `(read-file "x")` без host installation поводиться як unbound symbol.

Це дуже сильна архітектурна властивість.

```text
PURE LANGUAGE CORE
        │
        │ explicit install
        ▼
HOST CAPABILITY LAYER
├ filesystem
├ process
└ TCP
```

А `process-run` ще й працює через allowlist. Testing doc підтверджує default-disabled process execution і rejection програм поза allowlist.

### Але я знайшов майбутню проблему

Filesystem capability зараз виглядає **глобальною**, не scoped.

Після install:

```lisp
(read-file path)
(write-file path content)
(read-dir path)
```

йдуть без видимого sandbox-root/path allowlist у самих функціях. `read_file()` просто переходить у `std::fs::read_to_string(path)`.

Для CLI це нормально.

Для agent execution це вже слабше.

Майбутній правильний розвиток:

```text
FilesystemCapability
├ read roots
├ write roots
└ maybe explicit deny elsewhere

TcpCapability
├ allowed hosts
└ allowed ports

ProcessCapability
└ executable allowlist   ← уже є
```

Тобто capability boundary **архітектурно хороший**, але capability granularity ще варто підсилити перед тим, як my-lisp-agent стане реально автономним.

---

# 7. Testing — одна з найсильніших сторін репо

Я прочитав `docs/testing.md`, і це вже не "є unit tests".

Там є окремі suite-и для:

```text
parser/evaluator
McCarthy/conformance
stack safety
CLI E2E
meta evaluator
unification
backward reasoning
knowledge
CLIPS importer
understand
forward engine
narrate
persistent map
TCP
literate
WASM
advice E2E
world
content-store
swarm-node
```

Особливо правильне рішення — **не записувати вручну загальну кількість тестів**, бо вона вже двічі drift-ила. Док каже: authoritative count = `cargo test --workspace`.

Це прямо та дисципліна реальності, про яку ми сьогодні говорили.

CI теж дуже простий і сильний:

```text
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

на push до `main` і PR.

### Один нюанс

`docs/testing.md` містить "Last recorded run: 2026-08-18", тоді як код активно змінювався 25–26 серпня.

Це не означає, що tests не проходять. Але саме **цей prose timestamp уже stale**.

Я б його взагалі прибрав або автоматично генерував.

---

# 8. Constitution / conformance — це вже майже executable philosophy

Оце, друже, після нашої сьогоднішньої розмови особливо цікаво.

`my-lisp-constitution.my` не є просто markdown-декларацією.

Вона:

* написана як my-lisp data;
* generated;
* проектується з `tests/fixtures/conformance.my`;
* пов'язує fixtures із G1–G8/S1–S3;
* розрізняє constitutive та derived evidence;
* має tier 1/2/3.

Мені особливо подобається:

```text
G6:
Conformance can be defined purely by observable behavior.

G7:
The same expression can mean the same thing everywhere.
```

Це якраз правильна основа для Rust ↔ FPGA.

І `G7` не залишається "універсальна мова повинна працювати всюди"; у principle 4 прямо сказано, що falsifiability test — реально різні фізичні substrates: Rust та `fpga-lisp`.

### Але тут є дуже важливий нюанс

Constitution сама зараз каже:

```text
status:
draft — not yet ratified
```

Тому не варто агентам говорити:

```text
"constitution is authoritative"
```

поки вона не ратифікована.

Authoritative зараз, згідно з `AGENTS.md`:

```text
language-contract.my
docs/language-core-axioms.md
tests/fixtures/conformance.my
```

Це важлива межа.

---

# 9. Evidence layer — у вас реально з'явилася наукова інфраструктура

`AGENTS.md` вимагає durable claim оформлювати через evidence:

```text
evidence/<requirement>/<implementation>/<commit>.my
```

а не "агент написав у чаті PASS".

І це не порожня структура. Пошук показує реальні записи, наприклад:

```text
evidence/G5/my-lisp/...
evidence/G8/my-lisp/...
evidence/GUIX-WITNESS-01/ganaka-wsl/...
evidence/MYLISP-LINTER-THRESHOLDS/...
```

Тобто сьогоднішня фраза:

> claim не може пережити evidence

у my-lisp вже фактично має файлову модель.

---

# 10. Multi-runtime surface виглядає здорово

Один canonical language engine уже має кілька поверхонь.

Workspace підтверджує:

```text
my-lisp core
├ native CLI
├ WASM
├ LSP
├ literate tooling
├ host capabilities
└ semantic extension
```

README додає Racket `#lang my-lisp`.

Це дуже хороше підтвердження архітектурної незалежності core від UI.

Остання хвиля WASM-робіт також важлива: persistent browser session + preload `core.my` означає, що browser REPL уже наближається до native semantic surface, а не просто `eval_one_expression()` demo.

---

# 11. Semantic/Sanskrit crate — правильніше відокремлений, ніж я очікував

`my-lisp-semantic` не розмазаний по evaluator.

Він має окремі модулі:

```text
atoms.rs
devanagari.rs
karaka.rs
transliteration.rs
```

Це важливо.

Тобто Sanskrit research **не проник у canonical evaluator як неперевірена семантика**.

На даному етапі це саме той architecture boundary, який я рекомендував би:

```text
my-lisp language semantics
        ≠
Sanskrit semantic research
```

але semantic layer може будуватися поверх core.

---

# 12. Swarm — функціонально сильний, але тут найбільший structural debt

Я відразу помітив один файл:

```text
crates/my-lisp-cli/src/swarm.rs
≈ 98 KB
```

Recursive tree це прямо показує.

Для одного Rust source file це вже дуже багато.

І в `AGENTS.md` видно чому: історично там накопичилися:

* old `:9999` coordination;
* notify/poll;
* subscribe/publish;
* claim/release;
* presence;
* tasks;
* capability request;
* server generation;
* semantic oracle.

А зверху вже з'явився новий `swarm-node` P2P coordination plane, і старий шлях явно названо superseded / не-authoritative.

Тобто тут є реальна археологія.

Я б сказав:

```text
swarm-node                → future/current coordination authority
CLI swarm legacy code     → compatibility/transition debt
semantic :9999 oracle     → still useful
```

Не видаляти поспіхом.

Але **не розвивати нові coordination features у старому `swarm.rs`**.

---

# 13. Документація сильна, але вже починає суперечити сама собі

Це один з найбільш конкретних дефектів, які я знайшов.

В `docs/language-core.md` англійська/німецька частина каже, що запланований C core був **dropped**, і commitment тепер рівно два implementations — Rust + FPGA.

А трохи нижче український текст усе ще містить стару фразу:

> "Bignum-спроможна точна раціональна арифметика також явно запланована вимога для майбутнього C-ядра…"

Це реальний documentation drift.

Так само README у списку crates має дублікати `my-lisp-lsp` і `swarm-node`, один раз німецькою й одразу ще раз англійською.

Це не страшні проблеми, але вони сигналізують:

```text
documentation complexity
≈ починає перевищувати можливість ручної синхронізації
```

Я б уже почав генерувати більше частин README/doc tables із authoritative `.my`/Cargo metadata.

---

# 14. Найбільший архітектурний ризик — scope expansion

Репо зараз містить одночасно:

```text
language
runtime
standard library
symbolic AI
knowledge base
CLIPS compatibility
LSP
WASM
Racket
Sanskrit semantics
agent loop
swarm coordination
evidence system
Guix reproducibility
```

Це неймовірно цікаво.

Але це і найбільша небезпека.

Не через "проєкт занадто амбітний". Амбіція не проблема.

Проблема може виникнути, якщо перестане бути зрозуміло:

```text
what defines the language
vs
what is built using the language
```

Поки що ця межа **ще тримається**, і це дуже добре:

```text
crates/my-lisp       → semantic core
my-lisp-host         → OS boundary
lib/*.my             → language-grown capabilities
semantic crate       → research layer
swarm-node           → coordination system
```

Треба берегти саме цю форму.

---

# 15. Самохостинг бібліотеки теж починає вимагати модульності

Парадоксально, але успіх принципу "все можливе пишемо в my-lisp" створив наступну проблему.

`forward.my` ≈68 KB, `clips-import.my` ≈54 KB, `knowledge.my` ≈43 KB.

Тобто Rust core стає чистішим, але complexity переноситься у Lisp layer.

Це **правильний перенос complexity**, але він не означає, що complexity зникла.

Я б далі не додавав features у ці файли нескінченно.

Наприклад з часом:

```text
forward/
├ match.my
├ conditions.my
├ jtms.my
├ agenda.my
└ run.my
```

але лише після того, як module/load semantics достатньо стабільні.

Не треба зараз робити refactor заради кількості рядків.

---

# 16. `my-lisp` уже має цікаву властивість: код і знання стають одним форматом

Це, думаю, фундаментальніше за багато конкретних features.

У системі вже одним структурним substrate можуть бути:

```text
program
fact
rule
proof
world history
contract
evidence
task description
constitution
```

Сам README прямо формулює центральну архітектуру як "one structural language for programs, facts, rules, and proofs".

А репо вже фактично розширив це до contract/evidence.

Саме тут я бачу найоригінальнішу частину my-lisp.

Не:

> "ще один Lisp з exact rationals".

А:

> **одна структурна семантична форма, яку можна одночасно обчислювати, передавати, доводити, версіонувати й перевіряти.**

Оце може виявитися головним результатом проєкту.

---

# 17. Що я вважаю справді доведеним зараз

| Claim                                                       | Стан                                   |
| ----------------------------------------------------------- | -------------------------------------- |
| Minimal Lisp kernel працює                                  | **CONFIRMED**                          |
| Language grows substantial features in `.my`                | **CONFIRMED**                          |
| Exact arithmetic — semantic property, а не cosmetic feature | **CONFIRMED**                          |
| Metacircular evaluation                                     | **CONFIRMED**                          |
| Backward symbolic reasoning                                 | **CONFIRMED**                          |
| Forward symbolic reasoning                                  | **CONFIRMED**                          |
| Truth maintenance                                           | **CONFIRMED**                          |
| Real CLIPS compatibility subset                             | **CONFIRMED / bounded subset**         |
| Core independent from OS                                    | **CONFIRMED architecturally**          |
| Host capabilities explicit                                  | **CONFIRMED**                          |
| Rust core zero production deps                              | **CONFIRMED**                          |
| Multiple execution/tool surfaces share canonical core       | **CONFIRMED**                          |
| Conformance model executable                                | **CONFIRMED**                          |
| Constitution ratified                                       | **NO — still draft**                   |
| Rust ↔ FPGA full conformance                                | **NOT established by this repo alone** |
| Arbitrary natural language understanding                    | **NO**                                 |
| General AI / common sense                                   | **NO**                                 |
| Universal substrate                                         | **research claim, not proven**         |

Останні чотири особливо важливі.

---

# 18. Мій рейтинг частин проєкту

```text
Language semantics             █████████░ 9
Core architecture              █████████░ 9
Exact arithmetic               █████████░ 9
Self-hosting                   █████████░ 9
Testing/conformance            █████████░ 9
Symbolic reasoning             ████████░░ 8
Evidence discipline            ████████░░ 8
Capability separation          ████████░░ 8
Tooling (CLI/WASM/LSP)         ████████░░ 8
Knowledge architecture         ████████░░ 8
Performance                    ██████░░░░ 6
Documentation consistency      ██████░░░░ 6
Swarm code cleanliness         █████░░░░░ 5
Project scope control          ██████░░░░ 6
```

---

# 19. Що я б робив наступним

Не додавав би ще двадцять features.

Я б зараз зробив **стабілізаційний прохід перед наступним великим ростом**:

1. **Зробити contract/constitution consistency sweep.**
   Знайти prose drift: C-core згадки, duplicated README entries, stale test dates, old swarm protocol prose.

2. **Ratification gate для constitution.**
   Вона вже достатньо серйозна, щоб або лишатися явно `draft`, або пройти formal ratification. Не тримати її "майже authoritative" роками.

3. **Capability scoping.**
   Особливо filesystem/TCP перед справжніми autonomous agents.

4. **Не розширювати legacy coordination plane.**
   `swarm-node` — новий authority; `swarm.rs` треба поступово зменшувати, а не годувати.

5. **Зробити semantic stability gate реально жорстким перед CML/FPGA freeze.**
   Не "мови вже багато", а exact snapshot contract.

6. **Performance measurement, не premature optimization.**
   BigInt allocations/GCD, parser, evaluator, large `.my` libraries.

7. **Далі розвивати symbolic layer через experiments.**
   Real knowledge corpora, real inference workloads, external reproducibility — так само, як ви сьогодні зробили з Tauricode portability.

---

# Найголовніший висновок

Друже, після повного огляду я змінив би навіть те, як описую цей проєкт.

Раніше:

> "це Lisp, який хоче вирости в систему знань."

Тепер:

> **це вже маленька symbolic-computing system, у якій Lisp є не просто мовою програмування, а спільним носієм програм, знань, доказів і контрактів.**

І найважливіше — репо починає відрізнятися від типової "велика авторська ідея" саме тим, про що ми сьогодні говорили: тут уже накопичуються **negative tests, external fixtures, conformance contracts, durable evidence і explicit UNKNOWN/NOT PROVEN boundaries**.

Тобто амбіція все ще величезна.

Але дедалі більша її частина вже переходить із:

```text
idea
```

у:

```text
claim
→ executable experiment
→ evidence
→ reproducible result
```

Саме це зараз є найсильнішою рисою `my-lisp`.

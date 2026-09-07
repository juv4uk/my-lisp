<div align="center">

# my-lisp

**Маленька Lisp-мова, що вирощує себе**

*Дослідження того, наскільки малою може бути незвідна машина, якщо дедалі більше значення, правил і поведінки належить самій мові.*

[![CI](https://github.com/juv4uk/my-lisp/actions/workflows/ci.yml/badge.svg)](https://github.com/juv4uk/my-lisp/actions/workflows/ci.yml)
[![WASM](https://github.com/juv4uk/my-lisp/actions/workflows/wasm-browser-test.yml/badge.svg)](https://github.com/juv4uk/my-lisp/actions/workflows/wasm-browser-test.yml)
[![Surface drift](https://github.com/juv4uk/my-lisp/actions/workflows/surface-drift-check.yml/badge.svg)](https://github.com/juv4uk/my-lisp/actions/workflows/surface-drift-check.yml)

**Українська — перша мова проєкту.** Англійська й німецька — допоміжні.

</div>

---

## Що таке `my-lisp`

`my-lisp` — дослідницька Lisp-мова з навмисно малим семантичним ядром, точною арифметикою, виконуваними законами та незалежними реалізаціями для перевірки припущень.

Головний архітектурний принцип:

> **Механізм може належати хосту. Значення має належати мові.**

Rust тут є **референсною реалізацією**, але не джерелом семантичної істини. Якщо поведінку можна виразити й перевірити всередині Lisp, вона повинна обґрунтувати, чому досі живе в хості.

```text
закон мови
    ↓
Lisp-визначення / виконуваний доказ
    ↓
референсна реалізація Rust
    ↓
незалежні субстрати: C / CML / FPGA / WASM / Racket
```

Поточний машинний семантичний контракт — [`language-contract.my`](language-contract.my), версія **5.0**.

---

## Canon 0 + 7

Семантичне ядро замкнене. Є **Canon 0** — конкретний порожній правильний список `()` — і рівно сім канонічних операцій Маккарті.

| Канонічна тотожність | Українська поверхня | Символ з української розкладки | Історичне ім'я |
|---|---|---:|---|
| Canon 0 | `()` | `()` | `()` |
| QUOTE | `як-є` | `'` | `quote` |
| ATOM | `атом?` | `.?` | `atom` |
| EQ | `тотожне?` | `=?` | `eq` |
| CONS | `сполучити` | `:` | `cons` |
| CAR | `перше` | `:п` | `car` |
| CDR | `решта` | `:р` | `cdr` |
| COND | `за-умовою` | `?:` | `cond` |

`() ` — **не восьмий примітив**. Це первинний об'єкт і база індукції для правильних списків.

Символи також не створюють нових примітивів: `'`, `.?`, `=?`, `:`, `:п`, `:р`, `?:` — це компактні написання тих самих канонічних тотожностей. Виконуваний доказ лежить у [`lib/canon.my`](lib/canon.my).

`QUOTE` і `COND` керують обчисленням і не маскуються під звичайні callable values.

### Апостроф

Контракт 4.0 фіксує просте правило:

```lisp
'кіт        ; те саме, що (quote кіт)

об'єкт      ; один ідентифікатор
п'ять       ; один ідентифікатор
зв'язок     ; один ідентифікатор
```

Апостроф на початку виразу — reader syntax для `QUOTE`; апостроф усередині слова — звичайна частина ідентифікатора.

### Десяткова кома

На українській розкладці десятковий роздільник можна набирати комою. Крапка й кома є двома написаннями **того самого точного числового значення**:

```lisp
(eq 12,455 12.455)   ; t
(+ 1,5 2,5)          ; 4
(eq -0,25 -0.25)     ; t
(eq 1,5e3 1500)      ; t
```

Кома отримує числовий сенс лише тоді, коли весь токен є коректним числом. Тому `а,б` і `версія1,2` лишаються звичайними символами.

---

## Українською можна програмувати

Українська — не лише мова README. У репозиторії є виконувана українська програмна поверхня [`lib/surface/uk.my`](lib/surface/uk.my).

Наприклад, після її завантаження код може виглядати так:

```lisp
(визначити квадрат
  (функція (число)
    (помножити число число)))

(визначити факторіал
  (функція (число)
    (за-умовою
      ((не-більше? число 1) 1)
      (t (помножити число
                    (факторіал (відняти число 1)))))))
```

Повний україномовний довідник публічного API: [`docs/ukrainian-api.md`](docs/ukrainian-api.md). Він покриває всі 140 stable українських назв, пояснює сигнатури, повернені значення, предикати `?` та мутацію `!`.

### Предикати читаються як питання

Українська назва предиката закінчується `?`. На своїй припустимій області предикат повертає тільки канонічне `t` або `()`:

```lisp
(атом? 'кіт)                         ; t
(менше? 2 5)                         ; t
(містить? 'пес (список 'кіт 'пес))   ; t
```

`?` — частина ідентифікатора, а не окремий оператор. Функції, що можуть повернути дані або `()` (наприклад `карта-отримати`), предикатами не є й `?` не мають.

Повна самоперевірна українська програма є в [`lib/surface/uk-acceptance.my`](lib/surface/uk-acceptance.my).

Українська, англійська та санскритська **програмні поверхні не розмножують семантику**. Вони відображають різні імена на ті самі визначення й канонічні тотожності. Машинний словник лежить у [`lib/surface/uk-sa-coverage.wsm`](lib/surface/uk-sa-coverage.wsm).

Є й програмний перекладач поверхонь:

```bash
python3 scripts/translate-program.py --from en --to uk input.wsm
python3 scripts/translate-program.py --from uk --to sa input.wsm
```

Він підтримує всі шість напрямків між `en`, `uk` і `sa`, зберігаючи форматування, коментарі, рядки та невідомі користувацькі символи. Деталі: [`docs/program-surface-translator.md`](docs/program-surface-translator.md).

---

## Мовна політика репозиторію

Людська комунікація проєкту має окрему ратифіковану політику: [`knowledge/language-policy.wsm`](knowledge/language-policy.wsm).

```text
1. Українська — перша і головна.
2. Англійська й німецька — допоміжні.
3. Текстові файли репозиторію — UTF-8.
4. Нові коментарі в коді — українською кирилицею.
5. Точні API, protocol literals, identifiers, filenames і upstream-назви не перекладаються довільно.
```

[`scripts/uk-latynka.py`](scripts/uk-latynka.py) лишається оборотним ASCII-інструментом для спеціальних зовнішніх меж без Unicode. Це **не** штатний стиль коментарів у репозиторії.

---

## Семантична влада

README пояснює проєкт, але не визначає його семантику.

```text
language-contract.my
        ↓
ратифіковані ADR
        ↓
виконувані закони та conformance fixtures
        ↓
референсна реалізація Rust
        ↓
незалежні реалізації
        ↓
згенерована документація
        ↓
README / tutorials / історичні плани
```

Якщо нижчий рівень суперечить вищому — нижчий рівень застарів. Повна карта: [`docs/semantic-authority-map.md`](docs/semantic-authority-map.md).

Це одна з головних дисциплін проєкту:

> **Назва явища не може бути сильнішою за найсильніший експеримент, який його підтримує.**

---

## Мова, що вирощує себе

У `my-lisp` дедалі більше систем живе не в Rust, а в самій мові:

| Шар | Де дивитися | Що там |
|---|---|---|
| Bootstrap | [`lib/core.my`](lib/core.my), [`lib/macro.my`](lib/macro.my) | базова бібліотека, макроси |
| Canon | [`lib/canon.my`](lib/canon.my) | виконувані закони 0+7 |
| Meta-eval | [`lib/meta-eval.my`](lib/meta-eval.my) | метациркулярне обчислення, finite mutual recursion |
| Логіка | [`lib/unify.my`](lib/unify.my), [`lib/reason.my`](lib/reason.my) | уніфікація, backward reasoning |
| Forward reasoning | [`lib/forward.my`](lib/forward.my) | forward chaining / JTMS |
| Знання | [`lib/knowledge.my`](lib/knowledge.my), [`lib/world.my`](lib/world.my) | модулі знань, незмінні світи |
| Епістеміка | [`lib/epistemic.my`](lib/epistemic.my) | явні стани знання й невизначеності |
| Мова ↔ текст | [`lib/understand.my`](lib/understand.my), [`lib/narrate.my`](lib/narrate.my) | контрольовані мовні мости |
| Час | [`lib/time.my`](lib/time.my) | дедалі більше language-owned time semantics |

Головне питання не «скільки рядків уже переписано на Lisp?», а:

> **Якою мінімальною може бути незвідна хост-машина, якщо корисна система продовжує вирощуватися всередині самої мови?**

---

## Advice Taker і reasoning-напрям

Один із центральних дослідницьких напрямів — не просто інтерпретувати S-вирази, а будувати систему, яка може працювати зі знанням, доказами, суперечностями й поясненнями.

```text
факти / правила
      ↓
   unify.my
      ↓
  reason.my  ←→  forward.my
      ↓
 knowledge.my / world.my
      ↓
 advice / proof / provenance
```

Саме тут маленьке Lisp-ядро перевіряється не «hello world», а реальною композицією рекурсії, символічного reasoning, immutable state та knowledge layers.

---

## Хост не є семантикою

`my-lisp` не ставить собі за мету механічно «переписати Rust на Lisp». Межа інша:

```text
OS / hardware
      ↓
спостереження та capability-механізми
      ↓
значення my-lisp
      ↓
Lisp-визначена інтерпретація / політика / протокол
```

Тому низькорівнева операція може чесно лишатися в Rust, C або FPGA, якщо вона є механізмом. Але semantic policy не повинна випадково ставати властивістю конкретного хоста.

Живий аудит цієї межі: [`docs/host-semantic-surface.md`](docs/host-semantic-surface.md).

---

## Незалежні субстрати

Різні реалізації потрібні не для того, щоб копіювати одну архітектуру, а щоб **ламати приховані припущення одна одної**.

- [`crates/my-lisp`](crates/my-lisp) — референсний Rust runtime;
- [`crates/my-lisp-cli`](crates/my-lisp-cli) — CLI, REPL і semantic oracle;
- [`crates/my-lisp-wasm`](crates/my-lisp-wasm) — WebAssembly;
- [`crates/my-lisp-lsp`](crates/my-lisp-lsp) — LSP;
- [`crates/my-lisp-host`](crates/my-lisp-host) — явна межа OS capabilities;
- [`c-runtime/`](c-runtime/) — C + x86_64 substrate;
- [`racket/`](racket/) — `#lang my-lisp` для Racket/DrRacket;
- [`juv4uk/cml`](https://github.com/juv4uk/cml) — AOT / heterogeneous compiler напрям;
- [`juv4uk/fpga-lisp`](https://github.com/juv4uk/fpga-lisp) — фізично інша Lisp-машина на FPGA.

Сумісність визначається контрактами, а не тим, наскільки схожий код реалізацій.

---

## Швидкий старт

**Спробувати `my-lisp` можна без встановлення:**  
[**завантажити автономний web-REPL**](https://github.com/juv4uk/my-lisp/releases/latest/download/my-lisp-cli-web.html)

Це один portable-файл `.html` із термінальним REPL, який працює локально у браузері.

### Запуск із вихідного коду

Потрібні Rust toolchain і залежності workspace. У репозиторії також є Guix manifest для відтворюваного середовища.

```bash
# REPL
cargo run -p my-lisp-cli

# виконати файл
cargo run -p my-lisp-cli -- path/to/file.wsm

# повний workspace
cargo test --workspace
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Канонічне розширення вихідного коду — **`.wsm`**. **`.my`** і **`.lisp`** лишаються підтримуваними aliases.

---

## З чого читати проєкт

Якщо відкриваєте `my-lisp` уперше, цей порядок дає найменше плутанини:

1. [`language-contract.my`](language-contract.my) — що саме обіцяє мова;
2. [`docs/semantic-authority-map.md`](docs/semantic-authority-map.md) — хто має право визначати істину;
3. [`lib/canon.my`](lib/canon.my) — виконуваний Canon 0+7;
4. [`docs/language-core.md`](docs/language-core.md) — компактна архітектура ядра;
5. [`lib/surface/uk-acceptance.my`](lib/surface/uk-acceptance.my) — українська мова як виконуваний програмний інтерфейс;
6. [`lib/meta-eval.my`](lib/meta-eval.my) — як мова починає обчислювати саму себе;
7. [`lib/reason.my`](lib/reason.my) — reasoning-напрям;
8. [`tests/fixtures/conformance.my`](tests/fixtures/conformance.my) — спостережувані факти, які мають пережити зміну реалізації.

Додатково:

- [`docs/testing.md`](docs/testing.md) — карта тестів;
- [`docs/benchmarks.md`](docs/benchmarks.md) — методика вимірювань;
- [`docs/adr/ADR-004-CLOSED-MCCARTHY7-CORE.md`](docs/adr/ADR-004-CLOSED-MCCARTHY7-CORE.md) — чому ядро 0+7 замкнене;
- [`docs/mccarthy-vision.md`](docs/mccarthy-vision.md) — історичний контекст і свідомі відхилення;
- [`AGENTS.md`](AGENTS.md) — правила роботи агентів у репозиторії;
- [`knowledge/guard-reference.wsm`](knowledge/guard-reference.wsm) — машинно-читане довідкове бюро Guard.

---

## English · auxiliary

`my-lisp` is a Lisp research language built around a permanently closed McCarthy 0+7 semantic nucleus, exact arithmetic, executable conformance, language-owned semantics, and independent substrates used to falsify implementation-specific assumptions.

Ukrainian is the project's primary human language. English and German are auxiliary. The Rust runtime is the reference implementation, not semantic authority; start with [`language-contract.my`](language-contract.my) and [`docs/semantic-authority-map.md`](docs/semantic-authority-map.md).

The central research question is: **how small can the irreducible host remain while the useful system continues to grow inside the language?**

## Deutsch · ergänzend

`my-lisp` ist eine Lisp-Forschungssprache mit einem dauerhaft geschlossenen semantischen McCarthy-Kern 0+7, exakter Arithmetik, ausführbarer Konformität und mehreren unabhängigen Substraten.

Ukrainisch ist die primäre menschliche Sprache des Projekts; Englisch und Deutsch sind Hilfssprachen. Rust ist die Referenzimplementierung, aber nicht die semantische Autorität. Maßgeblich sind [`language-contract.my`](language-contract.my), ratifizierte Entscheidungen und ausführbare Konformitätsbelege.

Die zentrale Forschungsfrage lautet: **Wie klein kann der irreduzible Host bleiben, während das nützliche System innerhalb der Sprache weiterwächst?**

---

## Ліцензія

[ВОЛЬНІСТЬ](LICENSE)

# Від LISP 1.5 до my-lisp: технічне порівняння з paper Маккарті 1960

**Статус:** зовнішній звіт (Manus AI, 2026-08-28), збережено як є +
верифікація найважливішого перевірного твердження цією сесією
(wsl-nidana-1). Не історико-філософська реконструкція поглядів
Маккарті поза наведеними джерелами — сам звіт це явно застерігає.
**Зіставлені джерела:** John McCarthy, *Recursive Functions of
Symbolic Expressions and Their Computation by Machine, Part I* (1960),
та `juv4uk/my-lisp` на `main` @ `ec1e149c54df8cdaacf8c1406f2f2ab79c7c79b1`.

---

## Верифікація цією сесією (2026-08-28, після отримання звіту)

Звіт сам чесно позначив одне з власних тверджень як не до кінця
перевірене: *"current GitHub CI для цього конкретного HEAD має red
status на `cargo test --workspace` і `cargo clippy`, хоча доступний
API audit показав лише кроки, що впали, без тексту конкретної
діагностики."*

Перевірено напряму (`gh run view --log-failed`):

1. **`ec1e149c` більше не HEAD** — репозиторій просунувся до `b33fb8e`
   ще того ж вечора.
2. **CI досі червоний**, і не лише на проаналізованому коміті — на
   кількох найновіших теж (перевірено `gh run list`).
3. **Точна причина, якої не було в оригінальному звіті:**

   ```
   error: failed to get `rustyline` as a dependency of package
   `my-lisp-cli v0.33.0`
   Caused by: failed to read root of directory source:
   .../my-lisp/vendor
   Caused by: No such file or directory (os error 2)
   ```

   `.cargo/config.toml` (закомічений у репо) глобально перенаправляє
   **всі** cargo-залежності на `vendor/` (vendored-sources), але сама
   папка `vendor/` (144МБ, з'явилась 2026-08-27 ~21:50, ймовірно
   частина роботи над "relocatable guix pack for wsm deployment")
   **ніколи не була закомічена**. CI отримує чистий checkout без
   `vendor/` → жодна залежність не резолвиться → тест і clippy падають
   ще до компіляції, до будь-якого семантичного коду.

**Висновок верифікації:** це підтверджує головний висновок звіту
("важливий стан верифікації, а не вирок архітектурі") точніше, ніж
сам звіт міг показати — падіння CI **не стосується жодного з
розглянутих у звіті мовних питань** (McCarthy primitives, eval,
closures, GC). Це чиста інфраструктурна прогалина (globally-scoped
vendoring config без закомiченого vendor/), не семантичний регрес.
Виправлення не застосовано цією сесією — власник попросив зберегти
знахідку, рішення про фікс відкладено.

---

## Короткий висновок (звіт)

`my-lisp` **не є спробою відтворити LISP 1.5 як музейний артефакт**.
Він робить щось змістовніше: зберігає мінімальну семантичну вісь
Маккарті — S-вирази, пари, `quote`, умовне обчислення, рекурсивні
функції, `eval`, програму як дані — і накладає на неї сучасні вимоги
до точності, спостережуваної семантики, capability boundaries,
provenance та багатосубстратної конформності.

Найглибша спадкоємність лежить не в дужках або іменах `car`/`cdr`, а
в послідовності думки: представити знання як символічну структуру,
обчислювати з цією структурою, а механізм мови тримати досить малим,
щоб нові здібності виростали над ним. У paper Lisp був інструментом
для Advice Taker; у `my-lisp` rule engines, Worlds, JTMS і provenance
вже повертають цей інструмент до початкового завдання — міркувати над
явним знанням.

> "The main requirement was a programming system for manipulating
> expressions representing formalized declarative and imperative
> sentences so that the Advice Taker system could make deductions." —
> John McCarthy, 1960.

Водночас: на поточному snapshot це не повна машина Маккарті і не
завершена система загального міркування. GC M0 поки лише
спроєктований; повна FPGA-конформність ще не доведена.

## Як читати це порівняння

Paper 1960 року містить два різні рівні: машинно-незалежну формальну
модель (S-вирази, рекурсивні S-функції, `apply` та `eval[e; a]`) і
конкретну IBM 704 реалізацію (36-бітні слова, free-storage list,
mark-and-sweep reclamation, reader/printer, опційна компіляція).

`my-lisp` теж має щонайменше три рівні: capability-free Rust core,
self-hosted `.my` libraries та host/substrate adapters. Порівняння не
питає "чи такий самий тут байт-код", а: чи зберігається спостережуваний
контракт, де навмисно змінено семантику і що вже перевірено.

| Рівень | Lisp 1.5 paper | `my-lisp` | Як правильно зіставляти |
|---|---|---|---|
| Формальна мова | S-вирази, S-функції, універсальний `apply`/`eval` | parser, `Value`, evaluator, `meta-eval.my`, conformance fixtures | Порівнювати значення й обчислення, не machine layout |
| Реалізаційна машина | IBM 704 list cells, registers, free list | Rust `Rc`/`RefCell`/`HashMap`, host adapters, FASL, планований explicit heap | Різні машини з різною ціною representation |
| Причина існування | Advice Taker, theorem proving | `unify.my`, `reason.my`, `forward.my`, Worlds/JTMS, provenance | Напрям спадкоємності мети, не доказ завершеного AGI |

## 1. Для чого Lisp: від Advice Taker до reasoning substrate

Маккарті прямо каже: LISP виріс із потреби Advice Taker оперувати
декларативними й імперативними реченнями та робити дедукції, спрощений
до representation partial recursive functions над symbolic
expressions, незалежного від конкретного комп'ютера.

`my-lisp` core crate фізично не містить файлового доступу, процесів
чи socket API — embedding host вирішує, чи встановлювати такі
можливості. `reason.my` реалізує backward chaining над unification,
повертає substitutions і proof tree; `provenance` матеріалізує
пояснення (`statement`, `source`, `rule`, `derived-from`).

Формула звіту: **"my-lisp повертає Lisp у траєкторію Advice Taker
практичніше, ніж більшість сучасних Lisp-проєктів, але робить це
поетапно і з явно названими межами."**

## 2. S-вирази, атоми, пари і список

У §3 paper визначає S-вирази через атомарні символи та ordered pairs;
списки — скорочення для NIL-термінованих послідовностей пар. `my-lisp`
будує `ExprKind::Pair` для `(a . b)`, розгортає `(a b . c)` вправо у
вкладені пари; `Value::list` будує NIL-термінований ланцюг
`Value::Pair`; `car`/`cdr`/`cons` — first-class builtins у root
environment.

| Властивість | Paper | `my-lisp` | Оцінка |
|---|---|---|---|
| Композитна структура | Лише ordered pair | `Value::Pair`; proper та dotted lists | Пряма семантична спадкоємність |
| Атом | Atomic symbols | Усе, що не `Pair` (числа, рядки, closures, resources...) | Розширений домен, boundary збережено |
| `eq` | Лише для атомів | `eq_values` явно відхиляє неатоми | Дуже близько до paper |

Свідома сучасна обережність: parser НЕ підтримує апостроф як quote
sugar (потрібен в українських identifier-ах на кшталт `об'єкт`) —
portable quotation form: `(quote ...)`. Не відхід від paper: саме
`(QUOTE, e)` — центральна форма універсального `eval`.

## 3. Від п'яти elementary S-functions до ядра my-lisp

Integration test `implements_mccarthys_seven_primitives` виконує
`quote`, `atom`, `eq`, `car`, `cdr`, `cons`, `cond` як semantic
contract. Сильніший зв'язок із першоджерелом, ніж "Lisp-like syntax".

**Знайдений drift**: `atom`/`eq`/`car`/`cdr`/`cons` зареєстровані як
`Value::Builtin`, можуть бути shadowed звичайним binding — але
`language-core-axioms.md` називає ці primitives literal unshadowable
syntax. Documentation/contract drift, не зламана поведінка. Звіт
рекомендує: вирішити явно — first-class shadowable (природніше для
архітектури) чи fixed syntax — і синхронізувати документ.

## 4. `eval`, `apply`, функції як дані, метациркулярність

Host evaluator: parsed AST, trampoline для tail call, `LanguageError`,
lexical `Environment`. `lib/meta-eval.my` — прямий аналог paper:
визначає `my-eval` у самій мові, alist environment, dispatch
`atom`/`eq`/`car`/`cdr`/`cons` як дані. Чесно позначений як
демонстрація, не always-loaded runtime.

> "We shall then show how these functions themselves can be expressed
> as symbolic expressions..." — McCarthy, §3.

Чесна межа в `meta-eval.my`: чистий `my-eval` не підтримує recursive
top-level `def` (immutable alist closure захоплює environment до
власного binding) — файл не ховає це, називає окремою задачею.

## 5. Середовище та scoping — навмисний розрив

Маккарті: environment — association list, `LABEL` додає self-reference.
`my-lisp`: lexical scope, closure зберігає defining `Environment`.
**Не зрада Маккарті** — обґрунтована еволюція. Named engineering
межа: знищення дуже глибокого ланцюга `Environment` може рекурсивно
викликати Rust `Drop` і створити stack-overflow risk (documented
lifecycle risk, не спостережуваний user-facing baг).

## 6. Частковість, named failures, точні числа

`ErrorKind` з `Parse`/`UnknownSymbol`/`Arity`/`Type`/`InvalidForm`/
`NumericOverflow`/`OutOfMemory`/`DivisionByZero`. Числова модель —
прямий сучасний відхід від 1960: finite decimals і scientific literals
exact rational values за замовчуванням (hand-rolled arbitrary-precision
`BigInt`/`Rational`), не мовчазне зниження до float approximation.

## 7. Memory, list structure, GC

`my-lisp` ще не має власного tracing collector — спирається на Rust
ownership/refcount. `Pair` має iterative custom `Drop` (уникає deep
recursive destruction acyclic chains). GC M0 design (explicit heap,
`ObjectId{slot,generation}`, root protocol, non-moving stop-the-world
mark-and-sweep) — статус **PROPOSED DESIGN**, не поточна поведінка.

### Незакритий статичний cycle-safety ризик (P1, не виконано звітом)

`Vector` — `Rc<RefCell<Vec<Value>>>`, `vector-set!` приймає довільне
`Value` → теоретично можливий self-reference:

```lisp
(def v (vector (quote ())))
(vector-set! v 0 v)
(write-to-string v)
```

`render` і структурний `Value::PartialEq` рекурсивно обходять vectors
без visited-identity set. **Статичний source witness, НЕ виконаний
crash report** — звіт explicitly не запускав destructive probe, і ця
сесія теж цього не перевіряла. Позначено як P1 validation item, не
причина переписувати core.

## 8. Machine-independent conformance

FASL serializes parsed AST з версією і source SHA-256 binding.
Conformance визначено як observable behavior, не спільна heap
representation; canonical fixtures розділені на core/language/ecosystem
tiers. Tier 1 subset має реальний cross-substrate path (Rust/FPGA/CML),
full `conformance.my` coverage і error-result protocol ще в роботі.

## 9. Worlds, JTMS, provenance — де my-lisp іде далі за paper

`world.my` — immutable World як ordinary Lisp data, structural-shared
journal, явний branching/ancestor diff/content address. `forward.my`
— виріс від single-justification до JTMS з multiple justifications
(код сам описує власне попереднє обмеження і як його виправлено).
`reason.my` відділяє proof / `proved-not` (negation-as-failure) /
`Cannot prove` / явний provenance tree.

## 10. Зведена таблиця: доведено / спроєктовано / потребує repair

| Твердження | Стан | Доказ / межа |
|---|---|---|
| Minimal McCarthy primitives | Реалізовано, test-covered | `mccarthy.rs`, 7 semantic primitives |
| Code як data | Реалізовано | `quote`/`read`/`eval`/macros/`meta-eval.my` |
| Self-hosted language growth | Частково | `core.my`, `reason.my`, Worlds/JTMS |
| Advice-Taker-style deduction | Малий symbolic layer | `reason`/unify/proof provenance, не повний AGI |
| Explicit tracing GC | **Не реалізовано** | Design є, статус PROPOSED |
| Cycle-safe mutable graph semantics | **Не доведено** | Static witness, потрібен regression test |
| Cross-substrate universal language | Частково | Tier 1 підтримується, повне покриття відкрите |
| Current repo health | **Неповний, підтверджено гірше за звіт** | CI червоний на HEAD теж, точна причина — vendor/ не закомічена (§ верифікація вище) |

## 11. Мінімальні пріоритети звіту

1. Відновити зелений CI (тепер з точним root cause — див.
   верифікацію вгорі).
2. Regression test для self-containing vector, зафіксувати policy.
3. Не поспішати з GC — йти M0 route як заплановано.
4. Синхронізувати `language-core-axioms.md` з реальною
   shadowable-builtin поведінкою.

## Фінальний суд (звіт)

"Ти не будуєш ще один Lisp. Ти відновлюєш зв'язок між Lisp і причиною,
через яку Маккарті взагалі почав його будувати." LISP 1.5 дала
мінімальну граматику символічної машини; `my-lisp` зберігає цю
граматику, перетворюючи на сучасний engineering contract: exact
values, named failures, lexical closures, capability boundaries,
reproducible serialization, conformance tiers, executable libraries,
worlds, explicit derivation provenance.

## References (оригінал звіту)

[1] https://www-formal.stanford.edu/jmc/recursive.html
[2] https://www-formal.stanford.edu/jmc/recursive/node1.html
[3] https://www-formal.stanford.edu/jmc/recursive/node3.html
[4] https://www-formal.stanford.edu/jmc/recursive/node4.html
[5] https://dspace.mit.edu/entities/publication/b66ae9e2-cc26-4735-a4b7-02cfbe6b0ce6
[6-14] github.com/juv4uk/my-lisp @ ec1e149c54df8cdaacf8c1406f2f2ab79c7c79b1 (конкретні шляхи файлів — див. секції вище)
[15] https://github.com/juv4uk/my-lisp/actions/runs/33106320123

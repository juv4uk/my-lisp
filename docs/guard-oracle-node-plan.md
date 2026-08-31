# Guard, Oracle і swarm-node: узгоджений план

Статус: перший виконуваний зріз, 2026-08-31.

## Українською

Мета — не створити ще один framework, а звести наявні частини до однієї
моделі:

```text
Oracle = очікувана семантична істина
Observer/Rust = спостережені системні факти
Guard/WSM = пояснення різниці та правильного шляху
swarm-node = координаційний механізм, не semantic authority
```

Перший зріз додає `lib/guard.wsm` зі стабільним записом `guard/1`. Вісь
рішення (`allow/warn/reject/unknown`) навмисно відділена від доказового
статусу (`confirmed/partial/unresolved/broken`). Відсутність факту дає
`unknown`, а не прихований `reject`.

`UNKNOWN` не є кінцевою відповіддю. Він містить три явні маршрути:
`ask-agent` для знання, розподіленого між компонентами/агентами; `ask-owner`
для рішення про authority, намір, ліцензію або небезпечне розширення scope;
`research-web` для зовнішнього, змінного або первинного джерела. Кожен маршрут
також називає доказ, після якого питання можна закрити.

Oracle зберігає стару операцію `eval` без змін. Нова `oracle-eval` повертає
всередині звичайної transport-відповіді версійний `oracle-result/1` із
revision мовного контракту, SHA-256 джерела, outcome, output, evidence class
і provenance. Помилка evaluator-а є `(outcome error)`, а не transport failure.

Для агентів додано ще вужчу операцію `oracle-check`: вона тільки парсить
джерело, нічого не виконує й не викликає capabilities. Успіх повертає кількість
top-level форм. Помилка повертає `kind`, стабільний `code`, byte-span,
line/column, людське rendered-повідомлення та `suggested-edit`. Для двох
найчастіших агентних помилок це:

```text
unclosed-list
→ action=insert, text=")", offset=end-of-source

unexpected-closing-parenthesis
→ action=delete, start/end=exact parser span
```

Oracle ніколи не застосовує цю зміну сам: підказка має бути перевірена
повторним `oracle-check`. Той самий diagnostic payload тепер зберігається і в
помилках `oracle-eval`, тому агент не втрачає позицію після semantic request.

Найкоротший локальний шлях для агента не потребує запущеного TCP Oracle:

```bash
my-lisp --oracle-check path/to/source.wsm
printf '%s\n' '(def answer (+ 40 2))' | my-lisp --oracle-check -
```

Exit `0` означає синтаксично повне джерело, exit `2` — структуровану
parser-помилку. Це не semantic success: валідна форма все ще може містити
невідомий symbol або runtime error.

Канонічний запуск ноди: одна identity → один unit → один абсолютний data-dir
→ один bootstrap peer → `--auto-sync` абсолютного `tasks.my`. Rust тепер
відхиляє неявні `node-1`/`unknown`, відносний data-dir, відсутній auto-sync
файл і повторне використання data-dir іншою identity. Listener захоплюється
до запуску background threads; перший auto-sync виконується одразу.

Readiness перевіряється через `(metrics)`: `synced`, `bootstrap-peers`,
`synced-peers`, `task-sync`. TCP connect сам по собі не означає готовність.
Membership (`join`) і transport registration (`agent-send register`) — різні
контракти й не повинні називатися одним словом «зареєстровано».

Наступний вузький крок: Rust adapter нормалізує read-only факти Git/systemd/
journal у clauses; Guard застосовує bounded policy; LSP показує той самий
result як hover/diagnostic. Жодного нового evaluator, registry чи scheduler.

## English

The goal is not another framework. It is one model over existing parts:

```text
Oracle = expected semantic truth
Observer/Rust = observed system facts
Guard/WSM = explanation of the difference and the proper path
swarm-node = coordination mechanism, not semantic authority
```

The first executable slice adds `lib/guard.wsm` with a stable `guard/1`
record. Decision (`allow/warn/reject/unknown`) is deliberately separate from
evidence status (`confirmed/partial/unresolved/broken`). Missing facts yield
`unknown`, never an implicit rejection.

`UNKNOWN` is not terminal. It carries three explicit routes: `ask-agent` for
ecosystem-local knowledge distributed across component owners; `ask-owner`
for authority, intent, license, or destructive-scope decisions; and
`research-web` for external, time-sensitive, or primary-source facts. Every
route names the evidence needed to close the question.

Legacy `eval` remains unchanged. The new `oracle-eval` operation carries a
versioned `oracle-result/1` inside the normal transport response, including
language-contract revision, source SHA-256, outcome, output, evidence class,
and provenance. Evaluator errors are `(outcome error)`, not transport errors.

Agents also get the narrower `oracle-check` operation. It only parses source;
it never evaluates code or invokes capabilities. Success reports the number of
top-level forms. Failure preserves the canonical parser's kind, stable code,
byte span, line/column, rendered diagnostic, and a suggested edit. An unclosed
list suggests inserting one `)` at end-of-source; an unexpected closing
parenthesis suggests deleting its exact span. The Oracle never applies edits
silently: agents must verify a repair by running `oracle-check` again. The same
diagnostic payload is now retained by `oracle-eval` errors.

The shortest local agent path needs no running TCP Oracle:

```bash
my-lisp --oracle-check path/to/source.wsm
printf '%s\n' '(def answer (+ 40 2))' | my-lisp --oracle-check -
```

Exit `0` means syntactically complete source; exit `2` means a structured
parser failure. This is not semantic success: a valid form may still contain
an unknown symbol or runtime error.

Canonical node startup is one identity → one unit → one absolute data-dir →
one bootstrap peer → absolute `tasks.my` via `--auto-sync`. Rust rejects
implicit identities/projects, relative state, missing auto-sync files, and a
data directory owned by another identity. The listener is bound before
background threads start, and initial auto-sync now runs immediately.

Readiness is observed through `(metrics)`: `synced`, `bootstrap-peers`,
`synced-peers`, and `task-sync`. A TCP connection alone is not readiness.
Swarm membership (`join`) and session transport registration
(`agent-send register`) remain distinct contracts.

Next bounded step: a Rust adapter normalizes read-only Git/systemd/journal
facts into clauses; Guard applies bounded policy; LSP renders the same result
as hover/diagnostic. No new evaluator, registry, or scheduler is introduced.

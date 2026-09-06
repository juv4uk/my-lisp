# План Clean Code для my-lisp

> **Domain-roadmap.** Порядок між напрямками визначає [`PLAN.md`](PLAN.md).
> Цей файл не є другим загальним backlog-ом і не може випереджати активні
> Advice Taker milestones без конкретного blocking quality debt.

Цей roadmap перетворює [`docs/clean-code.md`](docs/clean-code.md) на перевірювані
кроки якості, API та tooling. Пріоритет усередині цього домену визначається
залежностями й користю для Advice Taker, а не кількістю нових features.

## Правила виконання

Кожен крок має:

1. спочатку пройти G5-перевірку: чи виразне це бібліотекою my-lisp;
2. не створювати другу семантику поруч із World API;
3. мати EN/UK/DE документацію для публічного контракту;
4. мати regression або law tests;
5. пройти `guix shell rust -- cargo test --workspace`;
6. не змінювати language contract без окремого обґрунтування й conformance law;
7. або прямо підтримувати активний milestone у `PLAN.md`, або чекати своєї черги.

## Етап 0 — уже виконаний фундамент

- [x] Immutable `World`, history, branches і structural sharing.
- [x] Snapshot-local `reason-in-world` та `forward-in-world` без читання globals.
- [x] Knowledge writers як compatibility wrappers над World transitions.
- [x] Single-evaluation для guarded writer macros.
- [x] Persistent map і immutable content-addressed store.
- [x] Canonical read-back-safe representation як поточна semantic identity.

## Етап 1 — зафіксувати закони до нового tooling

### 1. Canonical serialization conformance ✅

- Описати канонічний друк кожного Value-kind: symbols, strings/escapes, proper та
  dotted lists, exact integers/rationals, inexact numbers.
- Додати implementation-independent fixtures `value → canonical text → value`.
- Зафіксувати, що content identity залежить від цього контракту, не від Rust
  printer internals чи SHA.
- Критерій готовності: Rust проходить fixtures; формат достатній для FPGA adapter.

Виконано 2026-08-11: [`docs/canonical-serialization.md`](docs/canonical-serialization.md)
визначає data-only домен і точний portable text; Tier-2 fixtures фіксують канонічний
текст та `read(write(value)) = value` для кожного підтриманого Value-kind. Content
identity тепер явно посилається на цей контракт, а не на Rust printer чи SHA.

### 2. Pure readers as the only semantics

- Перевести `reason-in`, `forward-in` і `describe` у compatibility wrappers над
  explicit World readers.
- Перевірити, що чисті readers не читають `*knowledge-journal*`.
- Не зберігати тимчасові Worlds як нову приховану модель стану.
- Критерій: однаковий legacy result, різні explicit snapshots ізольовані.

### 3. Legacy history policy

- Явно вирішити, чи convenience layer зберігає повний parent-chain, чи офіційно є
  journal-only boundary.
- Не додавати автоматичний merge.
- Критерій: один задокументований закон і тести; жодної двозначності.

### 4. Effect naming convention

- Провести аудит file/TCP/process/global convenience APIs.
- Обрати сумісну convention (`!`, namespace або чітко задокументований виняток).
- Спершу додати aliases/deprecation path; не ламати чинні програми мовчки.
- Критерій: effect видно з public name або metadata, capability boundary незмінна.

## Етап 2 — документація й discoverability

### 5. First-class docstrings

- Спроєктувати docstring як звичайні Lisp-дані/metadata, не новий opaque Rust type.
- Додати `doc` lookup і почати з World/knowledge public API.
- Docstrings пояснюють причину, закони та межі, не повторюють ім’я функції.

### 6. Public/private API

- Інвентаризувати exports кожного `lib/*.my`.
- Позначити helpers без обов’язкового namespace-механізму в ядрі.
- Критерій: користувач бачить малий public surface; старі helpers мають migration path.

### 7. Source and macro inspection

- Додати `source` та `macroexpand` спочатку як library/tooling facilities.
- Критерій: REPL показує doc, source і точну expansion для ключових macros.

### 8. Executable examples

- Визначити data-format прикладу.
- Запускати приклади World, knowledge, reasoning і serialization у test suite.
- Критерій: документація не може застаріти непомітно.

## Етап 3 — композиція та форматування

### 9. Threading macro

- Реалізувати `->` у `lib/core.my`, якщо single-evaluation і macroexpand contract
  доводяться без нового Rust primitive.
- Перевірити zero/one/many-step, додаткові аргументи й side-effect expression once.

### 10. Canonical formatter

- Спочатку специфікація: indentation, comments, dotted lists, quote sugar, width.
- Formatter консервативний: не переставляє форми й не змінює семантику.
- Law: `parse(format(parse(source))) = parse(source)`.
- CLI `my-lisp fmt file.my` з check-mode для CI.

## Етап 4 — навчальний linter

### 11. Linter data model

- Diagnostics як S-expression/data: code, severity, span, explanation, suggestion.
- Категорії: readability, complexity, purity, naming, duplication, dead-code,
  effects, documentation.
- `error`/`warning`/`hint`; style rules типово не блокують build.

### 12. Перші правила з найвищою цінністю

- hidden reads/writes of `*knowledge-journal*` and `*working-memory*`;
- excessive nesting і function/form size;
- broad names як hints;
- predicate/conversion naming conventions;
- undocumented public definitions.

### 13. Configurable complexity budgets

- Рекомендовані, не абсолютні: nesting, body forms, parameters, branches.
- Дозволити локальне пояснене suppression.
- Не створювати єдиний quality score.

### 14. Structural duplication

- Нормалізувати S-expression shapes і знаходити подібні піддерева.
- Повідомляти лише hint із двома locations; не робити auto-refactor спочатку.

## Етап 5 — contracts, errors і laws

### 15. Lightweight contracts and preconditions

- Contracts як metadata/data, не повна static type system.
- Явні `expects`/`returns` та `require`-подібні preconditions.
- Почати з World boundaries, де випадковий `cdr` error зараз найменш корисний.

### 16. Diagnostic context

- Розширити structured errors полями function/argument/expected/received/suggestion.
- Показувати Lisp form і позицію аргументу, зберігши source span.
- Узгодити Rust та FPGA-observable contract лише для семантично необхідних полів.

### 17. Property/law suite

- Immutable: операція не змінює input.
- Map: `get(insert(M,k,v),k) = v`.
- World: `parent(world-tell(W,...)) = W`.
- Serialization: `read(write(x)) = x`.
- Identity: `equal?(A,B) → address(A) = address(B)`.
- Formatter: parse/format structural preservation.

## Етап 6 — IDE та пояснення

### 18. IDE semantic views

- Inline hints: pure/effectful/recursive/reads-global/writes-global.
- Call hierarchy, AST explorer і data-flow graph.
- Дані для IDE походять із parser/linter contracts, не окремого аналізатора.

### 19. Structural `explain-code`

- Спочатку deterministic S-expression explainer без AI.
- Пояснювати control/data flow і World transitions, не вигадувати intent.

### 20. Refactoring assistance

- Extraction/naming suggestions лише після стабільних formatter, linter і source map.
- Автоматичні зміни мають перевіряти structural equivalence там, де це можливо.

### 21. Optional AI assistance

- Останній необов’язковий шар над структурованими diagnostics і explanations.
- AI не визначає semantics, canonical identity чи correctness.

## Свідомо не планується зараз

- автоматичний `world-merge` без закону conflict/source/time/context;
- мутабельні базові collections або загальний `set!`;
- важка static type system до перевірки lightweight contracts;
- formatter/linter, що переписує семантику;
- hard style limits без можливості поясненого винятку;
- нові Rust primitives для можливостей, уже виразних my-lisp;
- AI як передумова читабельності.

## Черга в межах цього domain-roadmap

Поки `PLAN.md` має активний milestone B1/B2, цей список **не є глобальною
чергою комітів**. Його беремо, коли пункт прямо розблоковує Advice Taker або
між semantic milestones:

1. Legacy readers delegated to explicit World readers.
2. Legacy history policy decision and tests.
3. Effect naming audit and compatibility proposal.
4. Docstring representation design spike in my-lisp data.

Після кожного коміту список переглядається за фактичними знахідками. Якщо він
конфліктує з `PLAN.md`, перемагає `PLAN.md`; нормативні ADR/contracts при цьому
мають вищу semantic authority за обидва roadmap-и.

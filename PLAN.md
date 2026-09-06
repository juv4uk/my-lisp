# Активний план my-lisp

> **Статус:** активний roadmap.  
> **Оновлено:** 2026-09-07.  
> **Головна мета:** Advice Taker. `my-lisp` — мова й execution substrate, що служить цій меті.

Цей файл містить актуальний порядок пріоритетів і коротку карту вже
підтвердженого фундаменту. Завершені деталі живуть у git history, ADR,
conformance-тестах та спеціалізованих evidence-документах, а не повертаються в
backlog як псевдо-задачі.

## Ієрархія планів

1. **`PLAN.md`** — єдиний активний порядок пріоритетів для `my-lisp`.
2. **`CLEAN_CODE_PLAN.md`** — domain-roadmap якості й API; не може самостійно
   перевизначати пріоритети цього файлу.
3. **`docs/ecosystem-roadmap.md`** — roadmap сумісності `my-lisp` / `cml` /
   `fpga-lisp`; підпорядкований цінності для Advice Taker та conformance.
4. **ADR і language contract** — нормативні рішення; roadmap не може їх
   переписувати без окремого процесу ратифікації.
5. **Тести й CI** — доказ стану. Claim не може бути сильнішим за найсильніший
   експеримент, який його підтримує.

## Правило пріоритету

Перед новою роботою ставимо три питання:

1. Чи наближає це Advice Taker — reasoning, knowledge, explanation або natural-language bridge?
2. Якщо це робота над ядром: чи усуває вона реальну semantic dependency, яка заважає пункту 1?
3. Чи є негативний тест або інший спосіб спробувати зруйнувати твердження до його розширення?

Якщо відповідь на перші два питання «ні», робота не є активним пріоритетом,
навіть якщо технічно цікава.

---

# A. Підтверджений фундамент — не розширювати без причини

Це не backlog. Це база, яку слід зберігати зеленою.

## A1. Closed semantic core

- ✅ Canon 0: порожній список `()` як ground object.
- ✅ Закритий McCarthy-7 semantic operation set.
- ✅ `lambda` / `define` лишаються evaluator capabilities, а не новими primitive identities.
- ✅ Language-owned похідні операції не повинні тихо повертатися в Rust builtins.

## A2. Meta-evaluator ownership

Головний `lib/meta-eval.my` уже має підтверджені main-path докази для:

- ✅ lexical closures;
- ✅ first-class builtins і lexical shadowing;
- ✅ macros;
- ✅ top-level `def`;
- ✅ self recursion;
- ✅ variadic і dotted lambda;
- ✅ finite mutual-recursion groups без cyclic host environment;
- ✅ failure observation для unresolved callable name (`UnknownSymbol`);
- ✅ distinction unresolved name vs non-callable value (`Type` / `not-callable`);
- ✅ fixed/rest lambda arity (`Arity`);
- ✅ malformed lambda-list structure (`InvalidForm`): non-symbol parameter,
  duplicate parameter, invalid dotted rest.

### Відомі межі meta-evaluator

- arbitrary later-binding visibility ще не доведена як загальна властивість;
- повна parity усіх native error classes не заявляється;
- `meta-eval` лишається explicit self-hosting witness, а не always-loaded runtime.

Ці межі не є автоматичним backlog. Їх беремо лише коли вони блокують Advice Taker,
conformance або конкретний self-hosting proof.

## A3. Advice Taker reasoning stability — B0/B1

Підтверджено 2026-09-07:

- ✅ `prove-goal` rule scan переписаний на tail-recursive accumulator + один
  `reverse`, без старого `append(... recursive-scan ...)`;
- ✅ full scan на 256 правил проходить на звичайному test-thread stack;
- ✅ порядок reasoning results не змінився;
- ✅ одна canonical data-only outcome algebra у `lib/result-status.my`:

```lisp
(proved statement results)
(unknown subject)
(partial value bound)
(blocked reason)
(disputed evidence)
(invalid reason payload)
```

- ✅ `proved` зберігає всі успішні alternatives, а не лише першу;
- ✅ explicit opposite proof відрізняється від absence of proof;
- ✅ двосторонні докази дають `disputed`, не `unknown`;
- ✅ malformed goal/module дають `invalid`, а не маскуються під `unknown`;
- ✅ старі `reason` / `reason-in` лишилися backward-compatible;
  opt-in adapters — `reason-observe` / `reason-in-observe`.

Докази: `reason_stack.rs`, `result_status.rs`, `reason_outcome_invalid.rs`,
`reason_in_outcome_invalid.rs`, ADR `unknown-result-semantics.md`.

## A4. Explanation + adversarial Advice Taker loop — B2/B3

Підтверджено:

- ✅ `narrate-outcome` не зливає `unknown`, `partial`, `blocked`, `disputed` і
  `invalid` в одну human-readable невдачу;
- ✅ `proved` presentation зберігає існуючий proof/provenance шлях;
- ✅ end-to-end corpus проходить один pipeline через admission → reasoning →
  structured outcome → narration для семи різних режимів:
  1. прямий факт;
  2. багатокрокове правило;
  3. recursive rule;
  4. unknown;
  5. explicit conflict/rejection;
  6. malformed advice;
  7. knowledge-package round-trip перед reasoning.

Докази: `narrate_outcomes.rs`, `advice_corpus.rs`; workspace CI #1019/#1020.

## A5. Portability / boundary hardening

- ✅ committed `core.my.fasl` має regression test на exact source hash;
- ✅ WASM browser workflow тепер запускається також при змінах semantic
  dependencies (`my-lisp`, `literate`, `lsp`, `core.my`, FASL, Cargo graph);
- ✅ Chrome і Firefox browser suites пройшли після зміни trigger;
- ✅ `wsm-guard-core` більше не приймає policy result через substring типу
  `"(decision allow)"`; перевіряється точна структурована `guard/1` форма,
  exact field layout, decision enum і evidence-status enum;
- ✅ adversarial spoof `(not-a-guard-finding (decision allow))` відхиляється.

---

# B. Головний активний фронт — Advice Taker

## B4. Natural-language / external translator bridge — **NEXT**

Стабільні structured outcomes тепер існують, тому можна під'єднувати зовнішній
translator без передачі йому semantic authority.

Межа незмінна:

```text
external translator
        ↓
candidate Lisp data
        ↓
validate / advise / advise-all
        ↓
reason-in-observe
        ↓
canonical semantic outcome
        ↓
narrate-outcome
```

### Перший milestone

Не «вільна розмова з LLM», а невеликий versioned corpus:

- input text;
- expected candidate clause/query data;
- accepted / rejected / ambiguous translation status;
- downstream Advice Taker outcome;
- збереження rejected/ambiguous cases як evidence, а не тихе перетворення на знання.

LLM або інший translator **не** отримує права напряму змінювати knowledge state.

## B5. Reasoning scale — вимірювати перед новою оптимізацією

Підтверджений stack crash уже виправлено. Predicate/head indexing лишається
потенційно цінним performance improvement, але не автоматичним наступним кроком.

Перед indexing:

1. повторити scale profile на актуальному `reason`;
2. окремо виміряти realistic Advice Taker corpus, не лише worst-case chain;
3. зафіксувати target metric;
4. лише тоді міняти indexing representation;
5. довести, що proof/result order і semantics не змінилися.

---

# C. Підтримувальний фронт — ядро, embedding, self-hosting

## C1. Не продовжувати механічний каталог evaluator errors

Після `UnknownSymbol` / `Type` / `Arity` / `InvalidForm` наступний error class
додається лише якщо він:

- потрібен поточному Advice Taker milestone;
- знаходить реальну divergence native/meta;
- або є conformance requirement.

Інакше це низький пріоритет.

## C2. Arbitrary later-binding visibility

Лишається важливим self-hosting question, але йде після активного Advice Taker
front, якщо не з'ясується, що він прямо його потребує.

Proof має бути finite-data і не повертати cyclic host environment як приховану
семантику.

## C3. Shrink Rust, grow Lisp

Не естетична мета. Переносимо семантику з Rust лише коли:

- вона виразна чинним Lisp без нового primitive;
- зменшується дублювання semantic authority;
- є parity/conformance proof;
- це робить reasoning stack простішим, переноснішим або перевірюванішим.

## C4. Scoped host capabilities — migration gate перед partially-trusted agents

Поточний trusted native Lisp-machine profile навмисно має широкий OS-доступ.
Не ламати його випадково. Але перед виконанням неповністю довірених agent scripts
потрібно окремо ратифікувати й реалізувати fine-grained embedding policy для:

```text
filesystem read roots
filesystem write roots
tcp connect destinations
tcp listen destinations
process policy
```

`docs/host-capability-scoping-adr-2026-08-27.md` лишається PROPOSED для FS/TCP;
це compatibility/security decision, а не прихований clean-code refactor.

## C5. Swarm two-plane migration

Нормативний напрям уже визначений:

```text
:9999 my-lisp semantic oracle
          ≠
:910x swarm-node coordination plane
```

Legacy coordination code в CLI не видаляємо «для чистоти», доки є живі
callers. Removal gate:

1. підтвердити, що агенти використовують `swarm-node` для coordination;
2. позначити legacy ops deprecated у tooling/docs;
3. мати migration test / replacement path;
4. лише тоді видалити broker/claims/presence/task coordination з `:9999`,
   не зачіпаючи semantic oracle.

---

# D. Екосистема й FPGA

`my-lisp`, `cml` і `fpga-lisp` — одна вертикаль, але не три рівноправні backlog-и.
Для цього репозиторію пріоритет такий:

1. source semantics і Advice Taker correctness;
2. portable conformance observations;
3. CML/FPGA execution того subset, який дає реальну цінність;
4. розширення hardware surface тільки після доказу потреби.

Деталі живуть у [`docs/ecosystem-roadmap.md`](docs/ecosystem-roadmap.md).

Найцінніший hardware напрям після стабілізації reasoning outcomes — поступове
виконання `core.my → unify.my → reason.my` на незалежному backend як сильний
тест універсальності source semantics.

---

# E. Clean Code

[`CLEAN_CODE_PLAN.md`](CLEAN_CODE_PLAN.md) виконуємо між semantic milestones або
коли конкретний quality debt блокує B4/B5. Clean Code не має створювати нову
semantic authority чи великий API surface «про запас».

---

# Поточний порядок робіт

```text
1. B4 — versioned external/NL translator corpus
2. B4 — candidate-data validation + rejected/ambiguous evidence path
3. повторний reasoning scale profile на актуальному engine
4. indexing лише якщо вимірювання це виправдовує
5. host capability scoping перед partially-trusted autonomous execution
6. staged legacy coordination removal після swarm-node migration proof
7. later-binding / deeper self-hosting proof, якщо ще актуально
8. CML/FPGA subset за реальною цінністю для reasoning
```

## Стоп-умови

Не рухаємося до наступного semantic milestone, якщо:

- CI червоний;
- новий claim не має executable evidence;
- failure mode відомий, але прихований human-readable string замість stable data;
- `unknown` використовується як synonym для false / invalid / blocked / disputed;
- новий primitive пропонується до перевірки, чи це можна виразити бібліотекою;
- зовнішній translator може обійти `advise`/validation і прямо писати knowledge;
- робота розширює систему до спроби зруйнувати поточну.

---

# Епістемічний статус

- **confirmed** — claim має актуальний executable proof;
- **partial** — механізм працює, але межа/coverage явно неповна;
- **broken** — експеримент спростував claim;
- **unknown** — ще немає достатнього експерименту.

Ні кількість тестів, ні красивий architecture diagram самі по собі не доводять
повноту. Назва явища не може бути сильнішою за найсильніший експеримент, який
його підтримує.

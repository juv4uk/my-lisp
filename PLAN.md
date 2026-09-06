# Активний план my-lisp

> **Статус:** активний roadmap.  
> **Оновлено:** 2026-09-07.  
> **Головна мета:** Advice Taker. `my-lisp` — мова й execution substrate, що служить цій меті.

Цей файл містить актуальний порядок пріоритетів і коротку карту вже
підтвердженого фундаменту. Завершені деталі живуть у git history, ADR,
conformance-тестах та evidence-документах, а не повертаються в backlog як
псевдо-задачі.

## Ієрархія планів

1. **`PLAN.md`** — єдиний активний порядок пріоритетів для `my-lisp`.
2. **`CLEAN_CODE_PLAN.md`** — domain-roadmap якості й API.
3. **`docs/ecosystem-roadmap.md`** — roadmap `my-lisp` / `cml` / `fpga-lisp`.
4. **ADR і language contract** — нормативні рішення; roadmap не може їх переписувати.
5. **Тести й CI** — доказ стану. Claim не може бути сильнішим за найсильніший експеримент.

## Правило пріоритету

Перед новою роботою:

1. Чи наближає це Advice Taker — reasoning, knowledge, explanation або NL bridge?
2. Якщо це ядро: чи усуває це реальну semantic dependency, яка заважає пункту 1?
3. Чи є негативний тест або інший спосіб спробувати зруйнувати твердження?

Якщо відповідь на перші два питання «ні», робота не є активним пріоритетом,
навіть якщо технічно цікава.

---

# A. Підтверджений фундамент — не розширювати без причини

## A1. Closed semantic core

- ✅ Canon 0: `()` як ground object.
- ✅ Закритий McCarthy-7 semantic operation set.
- ✅ `lambda` / `define` — evaluator capabilities, не primitive identities.
- ✅ Language-owned похідні операції не повинні тихо повертатися в Rust builtins.

## A2. Meta-evaluator ownership

Підтверджені main-path slices:

- ✅ lexical closures;
- ✅ first-class builtins і lexical shadowing;
- ✅ macros;
- ✅ top-level `def`;
- ✅ self recursion;
- ✅ variadic і dotted lambda;
- ✅ finite mutual-recursion groups без cyclic host environment;
- ✅ unresolved callable `UnknownSymbol`;
- ✅ unresolved name vs non-callable `Type`;
- ✅ fixed/rest lambda `Arity`;
- ✅ malformed lambda-list `InvalidForm`.

Відомі межі:

- arbitrary later-binding visibility не доведена як загальна властивість;
- повна parity усіх native error classes не заявляється;
- `meta-eval` — explicit self-hosting witness, не always-loaded runtime.

Не виправляти later-binding через dynamic-scope shortcut. Сильніший proof має
лишатися lexical і finite-data.

## A3. Advice Taker reasoning stability — B0/B1

Підтверджено 2026-09-07:

- ✅ `prove-goal` rule scan — tail-recursive accumulator + один `reverse`;
- ✅ 256-rule full scan проходить на ordinary test stack;
- ✅ result order не змінився;
- ✅ refreshed scale harness виконує N=100/500/1000 на ordinary stack;
- ✅ 5000/10000 лишені як explicit ignored/manual falsification profile;
- ✅ canonical data-only outcome algebra:

```lisp
(proved statement results)
(unknown subject)
(partial value bound)
(blocked reason)
(disputed evidence)
(invalid reason payload)
```

- ✅ `proved` зберігає всі alternatives;
- ✅ opposite proof ≠ absence of proof;
- ✅ both sides → `disputed`;
- ✅ malformed goal/module → `invalid`;
- ✅ legacy `reason` / `reason-in` backward-compatible;
- ✅ opt-in `reason-observe` / `reason-in-observe`.

Evidence: `reason_stack.rs`, `reason_scale.rs`, `result_status.rs`,
`reason_outcome_invalid.rs`, `reason_in_outcome_invalid.rs`, CI #1034.

## A4. Explanation + adversarial Advice Taker loop — B2/B3

- ✅ `narrate-outcome` зберігає distinction між `unknown`, `partial`, `blocked`,
  `disputed`, `invalid`;
- ✅ malformed/truncated tagged outcomes відхиляються як invalid presentation;
- ✅ `proved` presentation зберігає proof/provenance;
- ✅ 7-case end-to-end corpus: direct, multi-step, recursion, unknown, conflict,
  malformed advice, knowledge-package round-trip.

Evidence: `narrate_outcomes.rs`, `advice_corpus.rs`, CI #1020/#1030.

## A5. Portability / Guard / documentation hardening

- ✅ committed `core.my.fasl` перевіряється exact source hash;
- ✅ semantic changes trigger WASM browser workflow;
- ✅ Chrome + Firefox пройшли після trigger expansion;
- ✅ Guard Rust boundary перевіряє exact `guard/1` structure, не rendered substring;
- ✅ nested `(decision allow)` spoof відхиляється;
- ✅ reasoning/narration sections у `FUNCTIONS.md` оновлені;
- ✅ documentation regression рахує live `(def ...)` імена для цих двох модулів.

## A6. Scoped host capabilities — embedding mechanism

Підтверджено CI #1038:

```text
process allowlist
filesystem read roots
filesystem write roots
tcp connect host/port ranges
tcp listen address/port ranges
```

- ✅ policy per-session і shared across lexical children;
- ✅ `None` = trusted unrestricted default, backward compatibility preserved;
- ✅ filesystem canonicalization/enforcement належить `my-lisp-host`, не core;
- ✅ `read-file`, byte read, `read-dir`, `load` obey read roots;
- ✅ writes obey separate write roots;
- ✅ symlink escape regression denied;
- ✅ connect/listen independently gated before OS operation.

Не заявляється повний sandbox. Public CLI flags ще не є ратифікованим contract;
див. `docs/host-capability-scoping-adr-2026-08-27.md`.

---

# B. Головний активний фронт — Advice Taker

## B4. Natural-language / external translator bridge — **NEXT**

Стабільні structured outcomes існують, тому зовнішній translator можна
під'єднувати без передачі йому semantic authority:

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

Перший milestone — невеликий versioned corpus:

- input text;
- expected candidate clause/query data;
- accepted / rejected / ambiguous translation status;
- downstream Advice Taker outcome;
- rejected/ambiguous cases зберігаються як evidence, не як знання.

LLM або інший translator **не** отримує права напряму змінювати knowledge state.

## B5. Reasoning performance — вимірювати перед indexing

Stack-safety і N=100/500/1000 ordinary-stack completion вже підтверджені.
Наступне питання — performance, не correctness.

Перед predicate/head indexing:

1. виміряти realistic Advice Taker corpus, не лише worst-case full scan;
2. зафіксувати target metric;
3. за потреби виконати manual 5k/10k profile;
4. лише тоді міняти indexing representation;
5. довести незмінність proof/result order і semantics.

---

# C. Підтримувальний фронт — ядро, embedding, self-hosting

## C1. Не продовжувати механічний каталог evaluator errors

Після `UnknownSymbol` / `Type` / `Arity` / `InvalidForm` наступний class беремо
лише якщо він потрібен Advice Taker, знаходить реальну native/meta divergence
або є conformance requirement.

## C2. Arbitrary later-binding visibility

Explicit self-hosting proof gap. Не автоматичний bugfix backlog.

Потрібний proof має одночасно:

- бачити потрібні later top-level bindings;
- зберігати lexical scope;
- лишатися finite-data;
- не повертати cyclic mutable host environment.

## C3. Shrink Rust, grow Lisp

Переносимо semantic policy з Rust лише коли це зменшує duplicate authority,
має parity/conformance evidence і реально допомагає reasoning/portability.
Це не line-count contest.

## C4. Host capability scoping — user-facing migration remainder

Programmatic embedding enforcement уже confirmed. Залишилися окремі operational
рішення, які не маскуємо під невиправлений primitive:

- чи потрібні native CLI flags `--allow-fs-read`, `--allow-fs-write`,
  `--allow-tcp-connect`, `--allow-tcp-listen`;
- чи вони обмежують local session, TCP/oracle sessions або обидва;
- exact CLI syntax для IPv6/port ranges;
- чи unauthenticated TCP/oracle має перейти до stricter default policy.

До рішення trusted local CLI залишається backward-compatible unrestricted.

## C5. Swarm two-plane migration

Нормативний напрям:

```text
:9999 my-lisp semantic oracle
          ≠
:910x swarm-node coordination plane
```

`docs/swarm-mesh-v2.md` уже фіксує operational migration: шість агентів пройшли
onboarding, `swarm-node` має replacement operations, а `:9999` coordination ops
названі неактуальним шляхом going forward. Отже migration gate 1 — **evidence-backed**.

Залишок перед фізичним видаленням legacy coordination code:

1. зробити deprecation machine/tool-visible, не лише prose;
2. мати migration/replacement regression;
3. перевірити відсутність живих callers legacy ops;
4. лише тоді видалити broker/claims/presence/task coordination з `:9999`,
   не зачіпаючи semantic oracle.

---

# D. Екосистема й FPGA

Пріоритет вертикалі:

1. source semantics + Advice Taker correctness;
2. portable conformance observations;
3. CML/FPGA execution реально корисного subset;
4. hardware surface тільки після доказу потреби.

Найцінніший hardware proof — поступове виконання
`core.my → unify.my → reason.my` на незалежному backend.

---

# E. Clean Code

`CLEAN_CODE_PLAN.md` виконуємо між semantic milestones або коли quality debt
блокує B4/B5. Clean Code не створює нову semantic authority «про запас».

---

# Поточний порядок робіт

```text
1. B4 — versioned external/NL translator corpus
2. B4 — candidate-data validation + rejected/ambiguous evidence path
3. B5 — realistic Advice Taker performance profile
4. indexing лише якщо вимірювання це виправдовує
5. swarm legacy deprecation + migration regression
6. CLI host-scope surface лише після explicit operational decision
7. later-binding / deeper self-hosting proof, якщо Advice Taker його потребує
8. CML/FPGA subset за реальною цінністю для reasoning
```

## Стоп-умови

Не рухаємося до наступного semantic milestone, якщо:

- CI червоний;
- новий claim не має executable evidence;
- failure mode прихований human-readable string замість stable data;
- `unknown` використовується як synonym для false / invalid / blocked / disputed;
- новий primitive пропонується до перевірки, чи це можна виразити бібліотекою;
- зовнішній translator може обійти `advise`/validation і прямо писати knowledge;
- security mechanism декларується без adversarial bypass test;
- робота розширює систему до спроби зруйнувати поточну.

---

# Епістемічний статус

- **confirmed** — claim має актуальний executable proof;
- **partial** — механізм працює, але coverage/operational contract неповний;
- **broken** — експеримент спростував claim;
- **unknown** — ще немає достатнього експерименту.

Ні кількість тестів, ні красивий architecture diagram самі по собі не доводять
повноту. Назва явища не може бути сильнішою за найсильніший експеримент, який
його підтримує.

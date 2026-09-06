# Активний план my-lisp

> **Статус:** активний roadmap.  
> **Оновлено:** 2026-09-07.  
> **Головна мета:** Advice Taker. `my-lisp` — мова й execution substrate, що служить цій меті.

Цей файл містить **лише актуальні пріоритети**. Завершені та застарілі стани не
тримаються тут як псевдо-задачі: їх зберігає git history, ADR, conformance-тести
та спеціалізовані документи.

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

Останні structural/failure parity зміни підтверджені workspace tests/build і
zero-warning clippy. Тому старе твердження, що top-level recursion у `my-eval`
«не виправлена», більше не є поточним станом.

### Відомі межі meta-evaluator

- arbitrary later-binding visibility ще не доведена як загальна властивість;
- повна parity усіх native error classes не заявляється;
- `meta-eval` лишається explicit self-hosting witness, а не always-loaded runtime.

Ці межі не є автоматичним backlog. Їх беремо лише коли вони блокують Advice Taker,
conformance або конкретний self-hosting proof.

---

# B. Головний активний фронт — Advice Taker

## B1. Semantic outcomes для reasoning — **NEXT**

Сьогодні `reason-in` повертає `(substitution proof)` при успіху й `()` при
невдачі. Для Advice Taker цього вже недостатньо: `()` змішує різні причини
відсутності відповіді.

Потрібно визначити мінімальні **структуровані Lisp-дані спостереження**, не
exception framework. Орієнтир, не остаточний контракт:

```lisp
(proved substitution proof)
(unknown goal)
(conflict evidence)
(invalid reason payload)
```

### Критерій готовності B1

- одна канонічна data-only форма reasoning observation;
- `proved` не змінює зміст уже наявного proof tree;
- `unknown` не прирівнюється до false;
- explicit contradiction не прирівнюється до absence of proof;
- malformed knowledge/input відділений від логічного `unknown`;
- backward compatibility для чинного `reason-in` або явний migration path;
- adversarial tests на всі межі вище.

## B2. Пояснення не лише доказу, а й невдачі

Після B1 `narrate-answer` / provenance layer мають уміти пояснювати принаймні:

- що доведено і яким proof tree;
- що не знайдено доказу;
- де виявлено конфлікт;
- чому input/knowledge structure відхилено.

Людський текст — presentation layer. Семантичним контрактом залишаються
структуровані Lisp-дані.

## B3. Посилити end-to-end Advice Taker loop

Еталонний шлях:

```text
understand → advise/advise-all → reason-in → semantic outcome → narrate
```

Додати невеликий blind/adversarial corpus, де один і той самий pipeline проходить:

1. прямий факт;
2. багатокрокове правило;
3. recursive rule;
4. unknown;
5. explicit negative/conflict case;
6. malformed advice;
7. knowledge package round-trip перед reasoning.

Мета — не кількість fixtures, а різні failure modes одного наскрізного шляху.

## B4. Natural-language bridge — тільки після стабільних semantic outcomes

LLM або інший зовнішній translator може пропонувати data-only clauses, але не
отримує право напряму змінювати knowledge state. Межа лишається:

```text
external translator → candidate Lisp data → validate/advise → reasoning
```

Перший NL milestone має перевіряти точність перекладу на невеликому corpus та
зберігати rejected/ambiguous cases, а не маскувати їх як знання.

---

# C. Підтримувальний фронт — ядро й self-hosting

## C1. Не продовжувати механічний каталог evaluator errors

Після `UnknownSymbol` / `Type` / `Arity` / `InvalidForm` наступний error class
додається лише якщо він:

- потрібен B1/B2;
- знаходить реальну divergence native/meta;
- або є conformance requirement.

Інакше це низький пріоритет.

## C2. Arbitrary later-binding visibility

Залишається важливим self-hosting question, але переходить після B1/B2, якщо не
з’ясується, що Advice Taker прямо його потребує.

Proof має бути finite-data і не повертати cyclic host environment як приховану
семантику.

## C3. Shrink Rust, grow Lisp

Не естетична мета. Переносимо семантику з Rust лише коли:

- вона виразна чинним Lisp без нового primitive;
- зменшується дублювання semantic authority;
- є parity/conformance proof;
- це робить reasoning stack простішим, переноснішим або перевірюванішим.

---

# D. Екосистема й FPGA

`my-lisp`, `cml` і `fpga-lisp` — одна вертикаль, але не три рівноправні backlog-и.
Для цього репозиторію пріоритет такий:

1. source semantics і Advice Taker correctness;
2. portable conformance observations;
3. CML/FPGA execution того subset, який дає реальну цінність;
4. розширення hardware surface тільки після доказу потреби.

Деталі живуть у [`docs/ecosystem-roadmap.md`](docs/ecosystem-roadmap.md).

Найцінніший hardware напрям після стабілізації reasoning outcomes — не ще одна
інструкція ISA сама по собі, а поступове виконання `core.my → unify.my → reason.my`
на незалежному backend як сильний тест універсальності source semantics.

---

# E. Clean Code

[`CLEAN_CODE_PLAN.md`](CLEAN_CODE_PLAN.md) виконуємо між semantic milestones або
коли конкретний quality debt блокує B1–B4. Clean Code не має створювати нову
semantic authority чи великий API surface «про запас».

---

# Поточний порядок робіт

```text
1. Advice Taker semantic outcomes (B1)
2. Failure/conflict explanation (B2)
3. Adversarial end-to-end Advice Taker corpus (B3)
4. Виправлення ядра лише за результатами 1–3
5. Natural-language bridge (B4)
6. Later-binding / deeper self-hosting proof, якщо ще актуально
7. Розширення CML/FPGA subset за реальною цінністю для reasoning
```

## Стоп-умови

Не рухаємося до наступного пункту, якщо:

- CI червоний;
- новий claim не має executable evidence;
- failure mode відомий, але прихований human-readable string замість stable data;
- новий primitive пропонується до перевірки, чи це можна виразити бібліотекою;
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

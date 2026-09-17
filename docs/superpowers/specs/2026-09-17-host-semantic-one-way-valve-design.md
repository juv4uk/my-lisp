# Дизайн одностороннього клапана згортання Rust

**Дата:** 2026-09-17

**Батьківська задача:** #299

## Мета

Цей документ фіксує чинну директиву власника для активної фази згортання host-шару.

Попередній клапан, що стосувався лише семантичної влади, був надто слабким: він залишав `mechanism`, `boundary-data`, `observer`, ABI та machine-роботу можливими шляхами для нового росту Rust. Чинне правило навмисно простіше:

```text
Lisp може рости.
Наявний Rust може залишатися там, де він іще потрібний.
Rust може лише зменшуватися.
```

Цей текст замінює попередню версію цього документа всюди, де та дозволяла нові mechanism- або boundary-додавання в Rust.

## Зовнішній інваріант

Для кожної зміни репозиторію:

```text
якщо шлях закінчується на .rs:
    кількість доданих рядків має дорівнювати 0
    кількість нових .rs-шляхів має дорівнювати 0
```

Отже:

```text
Rust N+1 ⊆ Rust N
```

Операційно це клапан на source-diff, а не лише метрика семантичної влади.

## Що дозволено

Наявний Rust можна залишити без змін, поки він іще потрібний.

Rust можна зменшувати шляхом:

- видалення застарілих тверджень;
- видалення мертвих тестів;
- видалення застарілих semantic producer/converter-вузлів;
- видалення цілого `.rs`-файлу, коли його залишкова цінність уже перенесена в інше місце;
- усунення Rust-влади над cardinality/shape після появи Lisp-owned доказу.

Lisp-owned код і дані, контракти, witness-корпуси, документація та non-Rust launch/policy infrastructure можуть рости за потреби, за умови дотримання їхніх власних правил влади.

## Що заборонено під час цієї фази

Жодна класифікація Rust не є винятком. Усе нижче є RED, якщо додає рядки Rust:

- семантичний код;
- mechanism helpers;
- ABI/resource код;
- transport або external-boundary код;
- machine decoders/executors;
- Rust test observers;
- Rust-коментарі, додані до наявного файлу;
- нові Rust-тести;
- нові `.rs`-файли;
- replacement Rust, доданий одночасно з видаленням старого Rust деінде.

Цінну гілку, яка нарощує Rust, треба зберегти, а не викидати, але під час активного клапана вона не зливається у своєму поточному вигляді.

## Класифікація все ще має значення

#304 класифікує наявний Rust як:

```text
semantic-producer
semantic-converter
compatibility-bridge
boundary-data
mechanism
observer
dead/stale
```

Класифікація відповідає на одне питання: **що треба зрозуміти або зберегти перед видаленням?**

Вона не відповідає на питання, чи можна додавати новий Rust. Під зовнішнім клапаном відповідь на це друге питання завжди одна: ні.

## Безпека видалення

Клапан не повинен стерти останню виконувану копію корисного семантичного або наукового закону.

Для живого Rust-вузла, вибраного для згортання:

1. встановити, чи вузол reachable/current;
2. визначити, чи несе він унікальний корисний закон;
3. якщо унікальний закон є — спершу зберегти його у Lisp-owned доказі;
4. використати вже наявний механізм або non-Rust launch path, щоб спостерігати іменований Lisp-owned verdict;
5. видалити Rust-копію без додавання replacement Rust;
6. перевірити відповідні current semantic та mechanism lanes.

Мертвий або stale unreachable-вузол можна видаляти напряму після достатнього доказу reachability.

## Boundary і mechanism Rust

Наявний boundary/mechanism Rust може залишатися без змін, поки він потрібний. Цей дизайн не вимагає одномоментного переписування evaluator-а.

Однак позначки `boundary-data`, `mechanism`, `observer`, `ABI`, `machine` або `resource` не дозволяють ріст у цій фазі. Якщо нова можливість вимагає нового Rust, її відкладаємо або, де можливо, виражаємо через уже наявний нижній механізм плюс Lisp/data.

## Структура CI

#300 володіє виконуваним зовнішнім клапаном.

Він запускається перед semantic classification і має fail closed:

```text
новий .rs-файл               -> RED
будь-який доданий рядок *.rs -> RED
Rust deletion-only diff      -> може бути GREEN
Rust не змінено              -> звичайний CI продовжується
```

Це зовнішнє правило компонується з наявними внутрішніми guards:

```text
#115 host-test semantic-authority valve
#112 Lisp-owned expected meaning
#300 all-Rust subtraction valve
#304 deletion-safety inventory
```

Реалізація #300 також не має права додавати Rust.

## Поточні канонічні приклади

`main@f5ff9240...`:

```text
crates/my-lisp/tests/unify.rs   +0 / -14
```

Застарілий host-тест вирізано; replacement Rust не написано.

PR #303:

```text
Rust                               +0 / -20
Lisp witness corpora               ростуть
```

Host-side влада над cardinality корпусу видаляється, тоді як корисні закони переходять у Lisp-owned corpora.

PR #308:

```text
exact_quantity_arithmetic.rs       видалено
Lisp exact-quantity witness         додано
```

Точний закон енергії Planck×Cs та оберненість добутку/частки швидкості світла збережені над клапаном до зникнення Rust semantic oracle.

## Machine/backend робота

Machine/backend-гілки можуть містити цінні Lisp encoders, докази й дослідження поруч із Rust-додаваннями. Такі гілки належать до множини збереження.

Під час цієї фази:

```text
зберегти branch/value
заморозити Rust-growing merge
безпечним чином забрати Lisp-only цінність
повернутися/декомпозувати пізніше за явною майбутньою політикою
```

Не слід викидати цінну роботу лише тому, що чинний клапан блокує її Rust-growing форму.

## Володіння агентів

- **Локальний агент:** широкий reachability inventory, grep, deletion-only Rust cleanup, довгі локальні перевірки.
- **Web/integration агент:** рішення про збереження, синхронізація issue/contract, міграція Lisp-owned witness, exact-head integration review.
- **Machine/backend агенти:** зберігають гілки й продовжують корисний non-Rust аналіз; не зливають новий Rust, поки клапан активний.

## Критерії приймання

Активний дизайн реалізовано правильно, коли:

- жоден PR не може додати рядок Rust;
- жоден PR не може додати новий `.rs`-шлях;
- deletion-only Rust зміни залишаються можливими;
- унікальні закони виживають у Lisp до вирізання їхніх Rust-копій;
- наявний необхідний Rust може залишатися без змін;
- machine/backend Rust-growing робота зберігається, але заморожена;
- #300 механічно забезпечує зовнішній інваріант;
- жодна нижча класифікація не може стати винятком.

## Керівне речення

**Lisp може рости. Rust може залишатися там, де він необхідний. Поки host-шар вирізається як скульптура, Rust може лише зменшуватися.**

---

## English

# Rust One-Way Retirement Valve Design

**Date:** 2026-09-17

**Parent:** #299

### Purpose

This document records the current owner directive for the active host-retirement phase.

The earlier semantic-only valve was too weak because it left `mechanism`, `boundary-data`, `observer`, ABI and machine work as possible routes for new Rust growth. The current rule is intentionally simpler:

```text
Lisp may grow.
Existing Rust may remain where still necessary.
Rust may only get smaller.
```

This text supersedes the earlier version of this document wherever it allowed new Rust mechanism or boundary additions.

### Outer invariant

For every repository change:

```text
if path ends with .rs:
    added lines must equal 0
    new .rs paths must equal 0
```

Therefore:

```text
Rust N+1 ⊆ Rust N
```

Operationally this is a source-diff valve, not merely a semantic-authority metric.

### What is allowed

Current Rust may be left unchanged when it is still required.

Rust may be reduced by:

- deleting stale assertions;
- deleting dead tests;
- deleting obsolete semantic producers/converters;
- deleting an entire `.rs` file when its remaining value has moved elsewhere;
- removing Rust-owned cardinality/shape authority after Lisp-owned evidence exists.

Lisp-owned code/data, contracts, witness corpora, documentation and non-Rust launch/policy infrastructure may grow as needed, subject to their own authority rules.

### What is forbidden during this phase

No Rust classification is an exemption. The following are all RED if they add Rust lines:

- semantic code;
- mechanism helpers;
- ABI/resource code;
- transport or external-boundary code;
- machine decoders/executors;
- Rust test observers;
- Rust comments added to an existing file;
- new Rust tests;
- new `.rs` files;
- replacement Rust introduced while deleting old Rust elsewhere.

A valuable Rust-growing branch is preserved, not discarded, but it does not merge as-is while this valve is active.

### Classification still matters

#304 classifies existing Rust as:

```text
semantic-producer
semantic-converter
compatibility-bridge
boundary-data
mechanism
observer
dead/stale
```

The classification answers one question: **what must be understood or preserved before deletion?**

It does not answer whether new Rust may be added. Under the outer valve, the answer to that second question is always no.

### Deletion safety

The valve must not erase the last executable copy of a useful semantic or scientific law.

For a live Rust site selected for retirement:

1. establish whether the site is reachable/current;
2. identify any unique useful law it carries;
3. if a unique law exists, preserve it first in Lisp-owned evidence;
4. use an already-existing mechanism or non-Rust launch path to observe the Lisp-owned named verdict;
5. delete the Rust copy without adding replacement Rust;
6. verify the relevant current semantic and mechanism lanes.

A dead/stale unreachable site may be deleted directly after sufficient reachability evidence.

### Boundary and mechanism Rust

Existing boundary/mechanism Rust may remain unchanged while needed. This design does not demand a flag-day evaluator rewrite.

However, `boundary-data`, `mechanism`, `observer`, `ABI`, `machine`, or `resource` labels do not permit growth during this phase. If new capability requires new Rust, that work is deferred or expressed through existing lower mechanism plus Lisp/data where possible.

### CI structure

#300 owns the executable outer valve.

It runs before semantic classification and must fail closed:

```text
new .rs file                 -> RED
any added line in *.rs       -> RED
Rust deletion-only diff      -> eligible for GREEN
no Rust touched              -> continue normal CI
```

This outer rule composes with existing inner guards:

```text
#115 host-test semantic-authority valve
#112 Lisp-owned expected meaning
#300 all-Rust subtraction valve
#304 deletion-safety inventory
```

The #300 implementation itself must not add Rust.

### Current canonical examples

`main@f5ff9240...`:

```text
crates/my-lisp/tests/unify.rs   +0 / -14
```

The obsolete host test was cut; no replacement Rust was written.

PR #303:

```text
Rust                               +0 / -20
Lisp witness corpora               grow
```

Host-side corpus cardinality authority is deleted while useful laws move into Lisp-owned corpora.

PR #308:

```text
exact_quantity_arithmetic.rs       deleted
Lisp exact-quantity witness         added
```

The Planck×Cs exact-energy law and the speed-of-light product/quotient inverse are preserved above the valve before the Rust semantic oracle disappears.

### Machine/backend work

Machine/backend branches may contain valuable Lisp encoders, evidence and research alongside Rust additions. Such branches belong to the preservation set.

During this phase:

```text
preserve branch/value
freeze Rust-growing merge
harvest Lisp-only value where safe
resume/decompose later under an explicit future policy
```

Do not discard valuable work merely because the current valve blocks its Rust-growing form.

### Agent ownership

- **Local agent:** broad reachability inventory, grep, deletion-only Rust cleanup, long local verification.
- **Web/integration agent:** preservation decisions, issue/contract synchronization, Lisp-owned witness migration, exact-head integration review.
- **Machine/backend agents:** preserve branches and continue non-Rust analysis where useful; do not merge new Rust while the valve is active.

### Acceptance

The active design is correctly implemented when:

- no PR can add a Rust line;
- no PR can add a new `.rs` path;
- deletion-only Rust changes remain possible;
- unique laws survive in Lisp before their Rust copies are cut;
- existing necessary Rust can remain unchanged;
- machine/backend Rust-growing work is preserved but frozen;
- #300 enforces the outer invariant mechanically;
- no lower-level classification can act as an exemption.

### Governing sentence

**Lisp may grow. Rust may remain where necessary. While the host is being sculpted, Rust only gets smaller.**

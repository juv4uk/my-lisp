# Дизайн Lisp-owned інвентаря інструментів репозиторію

**Задача:** #382
**Статус:** затверджений дизайн реалізації
**Дата:** 2026-09-17

## Мета

Дати `my-lisp` один Lisp-owned, machine-readable інвентар активного repo-owned tooling, щоб сам репозиторій міг відповісти: що це за інструмент, хто його викликає, від якого джерела authority він залежить, який у нього життєвий статус у репо і за якої умови його можна прибрати.

Цей інвентар — governance metadata і проєкція над фактичним станом репозиторію. Він **не є semantic authority** і не повинен перетворюватися на друге джерело мовної істини.

> **Репозиторій має пам’ятати, навіщо існує інструмент, до того як люди вирішать, де йому лежати або чи може він зникнути.**

## Глобальні обмеження

Під час host-retirement авторитетною для цього зрізу є #299:

```text
Lisp may grow.
Existing Rust may remain unchanged where still necessary.
Rust may only shrink.
```

Отже #382 додає **нуль Rust-рядків** і жодного нового `.rs` файла. #76 лишається єдиною authority для Python→Lisp migration. #382 лише записує ownership міграції й не створює другого roadmap. Жоден script не переноситься і не видаляється лише заради зеленого інвентаря.

## Архітектурне рішення

Використовуємо окремий Lisp registry у `knowledge/` і Lisp checker у `scripts/`. Постійний gate запускає checker через наявний `my-lisp` CLI з **існуючого** CI workflow. Не створюємо другу реалізацію policy у Rust, Python чи Markdown і не залишаємо окремий постійний workflow.

Тимчасовий RED-only workflow допустимий лише як одноразовий ізольований доказ, якщо звичайний CI перекритий стороннім baseline-break. Це verification scaffolding: **перед готовністю PR його потрібно видалити**. Lifecycle workflow-ів належить #384.

## Файли та відповідальність

### `knowledge/repo-tooling-inventory.lisp`

Містить по одному рядку на кожний in-scope repo-owned tooling entrypoint.

Кожен рядок має поля:

```text
path
kind
language
role
lifecycle
callers
authority-source
migration-issue
replacement
removal-condition
```

Закриті словники першого зрізу:

```text
kind:
  check | generator | migration | benchmark | deploy | release | helper | other

lifecycle:
  active | transitional | legacy | generated-helper | archive-candidate

language:
  lisp | python | shell | javascript | powershell | other
```

Невідомі факти позначаються явно як `unknown` або `()` відповідно до типу поля. Registry не має права вигадувати callers, replacement чи authority.

Приклад рядка:

```lisp
(tool
  (path "scripts/generate-meta-eval-evidence.py")
  (kind generator)
  (language python)
  (role meta-eval-human-evidence-projection)
  (lifecycle transitional)
  (callers ("crates/xtask/src/checks.rs"))
  (authority-source "knowledge/meta-eval-evidence.lisp")
  (migration-issue 351)
  (replacement ())
  (removal-condition parity-green-and-callers-switched))
```

`replacement` лишається порожнім, доки replacement-path реально не існує. Заплановані назви файлів не видаються за факти репозиторію.

## `lifecycle` і стан міграції — різні виміри

`lifecycle` відповідає на питання:

> Яка роль/стадія життя цього artifact у репозиторії?

Стан міграції відповідає на інше питання:

> Де цей tool перебуває в процесі заміни?

Перший зріз навмисно містить лише artifact-level `lifecycle` плюс `migration-issue`. Python migration progress належить #76 і **не кодується** у lifecycle-значеннях на кшталт `parity-green`, `switched-to-lisp`, `removable` чи `bootstrap-exception`.

Якщо зрілий інвентар вимагатиме одночасно описати обидва факти — наприклад `archive-candidate` **і** `parity-green` — це явний trigger для schema evolution:

```text
lifecycle
migration-state
```

Майбутній `migration-state`, якщо він знадобиться, буде лише проєкцією/індексом #76, а не незалежною migration authority.

Тимчасовий bootstrap exception, якщо він реально потрібен, описується явними migration/removal metadata, а не маскується під artifact lifecycle.

### `scripts/check-repo-tooling-inventory.lisp`

Checker володіє executable governance checks. Його core validation має бути чистою Lisp-функцією над:

```text
inventory rows + observed immediate script-entry names
```

Filesystem-backed runner лише отримує реальні observations через `read-dir`/`read-file` і передає їх validator-у. Negative witnesses тому не потребують мутації робочого дерева.

Перший зріз механічно перевіряє:

- кожен in-scope immediate `scripts/*` entry представлений рівно один раз;
- кожен зареєстрований живий path реально присутній серед observed entries;
- duplicate `path` дає RED;
- відсутнє required field дає RED;
- невідомі `kind`, `language` або `lifecycle` fail-closed;
- активний/transitional repo-owned Python має `migration-issue`, що веде до #76 або child issue;
- replacement-path не оголошується фактом до його фізичної появи;
- removal condition зберігається явно, а не виводиться здогадом.

Checker повертає малий machine-readable verdict і точну діагностику. Він не визначає мовну семантику і використовує чинні explicit result domains замість generic `t/()` truth authority.

### Існуючий CI workflow

Фінальний gate — один сусідній step у `.github/workflows/ci.yml`, що запускається через canonical CLI. Без Rust glue і без постійного нового workflow.

### `knowledge/guard-reference.lisp`

Додається одна navigation topic `repo-tooling`, що вказує на inventory, checker, design і #382. Guard не копіює окремі tooling rows.

## Межа coverage

Перший implementation slice покриває immediate entries, які повертає `read-dir("scripts")`, крім явно out-of-scope directory entry `tests`.

Свідомо не класифікуємо тут:

- nested `scripts/tests/*` helpers;
- lifecycle GitHub workflows — #384;
- authority-класи всього `knowledge/*` — #383;
- environment paths — #385;
- lifecycle root-artifacts — #23;
- фізичну реорганізацію `scripts/`;
- сам Python→Lisp implementation migration — #76 і child issues.

## RED → GREEN дисципліна

RED зараховується лише тоді, коли виконання доходить до поведінки, яку ми тестуємо. Parser error, stale-branch failure або сторонній authority-guard failure **не є** валідним RED-доказом.

Порядок:

1. ізолювати одне відсутнє governance rule executable witness-ом;
2. побачити точний очікуваний failure на синхронізованій гілці;
3. реалізувати мінімальну Lisp-зміну, яка робить саме цей witness GREEN;
4. зберегти попередні GREEN witnesses;
5. повторити для duplicate path, stale path, invalid enum і Python migration ownership;
6. заповнити реальний inventory та ввімкнути filesystem-backed enforcement;
7. прибрати тимчасовий RED-only workflow перед завершенням.

## Анти-геймінг

- Script не переноситься лише для виходу зі scope checker-а.
- Python-файл не видаляється лише для зеленого coverage.
- Невідомі callers/authority лишаються явно невідомими, а не вигадуються.
- Generated artifact посилається на generator/upstream authority і не отримує authority через сам факт внесення в inventory.
- `replacement` лишається `()`, доки файл не існує.
- Checker ніколи не стає semantic oracle.

## Acceptance

Зріз завершений, коли:

- кожен in-scope immediate repo-owned `scripts/*` entry має рівно один row;
- inventory Lisp-owned і machine-readable;
- unregistered новий tool та stale registered path механічно виявляються;
- duplicate rows та invalid enum values механічно виявляються;
- active/transitional Python tooling прив’язаний до #76 або вузького child issue без другого migration state machine;
- Guard навігує до inventory без копіювання його rows;
- існуючий CI lane запускає checker через `my-lisp` CLI;
- тимчасовий RED-only workflow видалено;
- Rust additions = 0;
- жоден script не перенесений і жоден Python tool не видалений лише заради #382.

## Принцип

**Репозиторій має пам’ятати, навіщо існує інструмент, до того як люди вирішать, де йому лежати або чи може він зникнути.**

# Повна публічна українська поверхня my-lisp

Дата: 2026-09-13
Статус: design specification
Owner intent: повний переклад публічної програмної поверхні, а не історичного зрізу на 140 назв.

## 1. Мета

Українська surface має охоплювати **все, що користувач my-lisp може свідомо викликати як підтримуваний API**:

- Canon і необхідні форми;
- root/runtime builtins;
- публічні функції й макроси з усіх `lib/*.lisp`;
- підтримувані host-facing API, які є частиною програмної поверхні;
- compatibility spellings, якщо вони все ще підтримуються.

Старе число `140` є історичним зрізом попереднього `uk-docs`/core inventory і **не є Definition of Done для повного перекладу**.

Внутрішні accumulator/worker/bootstrap/helper функції не стають публічними лише тому, що мають top-level `def`.

## 2. Основна модель

Для кожної публічної semantic identity:

```text
numeric semantic ID
        │
        ├── uk   — коротке, але інтуїтивно зрозуміле українське ім'я
        ├── ukr  — повне, однозначне українське ім'я
        ├── en   — англійська peer-surface, якщо вона існує
        ├── sa   — санскритська peer-surface, якщо вона існує
        └── sym  — немовна нотація, якщо вона існує
```

`uk` і `ukr` не є двома семантиками. Вони є двома українськими spellings тієї самої numeric identity.

`ukr` — повна форма. Вона повинна бути придатною до набору без переходу на латинську клавіатурну розкладку. ASCII-латиниця в user-facing `ukr` identifier заборонена.

`uk` — компактна форма. Вона скорочує довжину, але не перетворюється на шифр. Її треба мати змогу інтуїтивно відновити до повної форми без окремого словника скорочень. Правила compact naming визначає `docs/superpowers/specs/2026-09-13-uk-compact-naming-design.md`.

## 3. Єдина semantic authority

Єдиним джерелом істини для відповідності spelling ↔ semantic identity лишається:

`lib/surface/semantic-registry.lisp`

Жоден generator, README, Markdown-довідник, visibility inventory або бібліотечний файл не має власного словника перекладів.

Кожна **публічна** бібліотечна функція/макрос отримує numeric semantic ID у тій самій registry, що вже володіє Canon/runtime identities.

Внутрішні implementation helpers не отримують ID лише заради перекладу.

### 3.1 Нові library identities

Для публічних top-level definitions, яких ще немає в registry:

1. спочатку доводиться, що definition є public API, а не helper;
2. їй призначається новий numeric semantic ID;
3. у registry додаються `uk`, `ukr`, `en` та інші доречні surfaces;
4. implementation projection продовжує жити у відповідному `lib/*.lisp` або host layer;
5. CI доводить, що runtime spelling справді виконує ту саму identity.

ID не кодує назву файлу, мову або категорію. Це непрозора числова тотожність.

## 4. Public vs internal — окрема вісь, не другий словник

Потрібен живий inventory усіх top-level `def`/`defmacro` у `lib/*.lisp` плюс підтримуваних runtime/host exports.

Visibility classification має окрему роль:

```text
(public <source-name> <source-file>)
(internal <source-name> <source-file> <reason>)
(compatibility <source-name> <source-file>)
```

Цей inventory **не містить `uk`, `ukr`, `en`, `sa` перекладів**. Він лише відповідає на питання: чи входить definition у підтримуваний API.

Отже:

- `semantic-registry.lisp` = identity + surface spellings;
- visibility inventory = public/internal classification + provenance;
- generated function table = projection/join цих джерел;
- Markdown/README = presentation, не authority.

### 4.1 Fail-closed правило

Новий discoverable top-level `def`/`defmacro` у `lib/*.lisp` не може тихо з'явитися поза governance.

CI має вимагати одну з трьох класифікацій:

- `public` — мусить мати numeric semantic ID;
- `internal` — не вимагає user-facing перекладу;
- `compatibility` — підтримувана історична поверхня, прив'язана до semantic identity.

`unclassified` = RED.

## 5. Повний inventory

Старий `lib/surface/uk-inventory.lisp` був вузьким inventory Canon/forms/root/core. Його цифри не можна використовувати як total coverage.

Новий coverage scanner повинен дивитися щонайменше на:

- усі `lib/*.lisp`;
- root/runtime public builtins;
- host-facing functions, які документовані або реально використовуються як user API;
- macros;
- compatibility spellings.

Приклади бібліотек, які вже доводять, що 140 недостатньо:

- `lib/time.lisp`;
- `lib/tcp.lisp`;
- `lib/utf8.lisp`;
- `lib/unify.lisp`;
- `lib/guard.lisp`;
- `lib/reason.lisp`;
- `lib/si.lisp`;
- `lib/fs.lisp`;
- інші `lib/*.lisp` з public definitions.

Число total coverage не хардкодиться в prose. Воно генерується з живого inventory.

## 6. Приклади очікуваного переходу

Це приклади форми, а не автоматична ратифікація конкретних слів:

```text
semantic ID | uk               | ukr                          | en
------------|------------------|------------------------------|-------------------
...         | текст-порожній?  | порожній-текст?              | string-empty?
...         | слухати-мережу   | слухати-мережеві-з'єднання   | tcp-listen-on
...         | логічна-змінна   | створити-логічну-змінну      | logic-var
...         | рішення-захисту? | рішення-механізму-захисту?   | guard-decision?
```

Остаточні `uk` compact names проходять naming review/blind-decoding. `ukr` оптимізується на ясність і однозначність, не на мінімальну довжину.

## 7. Runtime semantics

Додавання peer-surface не повинно створювати другу реалізацію функції.

Для Lisp-owned public definitions bootstrap/loading має зв'язувати stable peer spellings тієї самої numeric identity з одним початковим value, зберігаючи подальше незалежне lexical shadowing spellings.

Для Rust/host-owned builtins той самий принцип діє через semantic registry / builtin installation path.

Обов'язковий executable witness перевіряє не лише lookup metadata, а реальний виклик українського spelling.

## 8. Generated function table

Generated function table має охоплювати **всі public semantic identities**, а не лише старий documented subset.

Базові колонки:

```text
ID | uk | ukr | ukr-status | en | sa | sym | kind | source | public-status
```

`source`/`kind` — provenance projection із visibility inventory, а не частина semantic identity.

Не створювати третю українську колонку `full-uk`: `ukr` і є повна українська форма.

## 9. Український API reference

`docs/ukrainian-api.md` генерується з machine sources і охоплює весь public inventory.

Для кожної public identity reference показує:

- numeric semantic ID;
- `uk`;
- `ukr`;
- статус `ukr` (`stable`, `candidate`, `compatibility-only`, `missing` якщо це тимчасово дозволено міграцією);
- `en`/іншу основу для зіставлення;
- kind/signature;
- короткий опис поведінки;
- implementation source/module;
- executable/stability status.

Опис поведінки зберігається один раз за numeric ID. Не дублювати окремий опис для `uk` і `ukr`.

Reference не має хардкодити `140` як повний розмір surface. Лічильники генеруються автоматично, наприклад:

```text
public identities: N
uk stable: X/N
ukr stable: Y/N
ukr candidate: Z
missing Ukrainian: M
```

## 10. README

README — людський вхід, а не каталог усіх сотень functions.

Він обов'язково пояснює:

- `uk` = компактне, інтуїтивне;
- `ukr` = повне українське;
- обидві назви ведуть до того самого semantic ID;
- повне API значно ширше за історичні 140 назв;
- coverage counts генеруються з живої public surface;
- де відкрити повний український API reference;
- як побачити generated function table;
- що `candidate` не означає stable/executable promise.

README показує кілька репрезентативних пар `uk ↔ ukr`, але не дублює весь API.

## 11. Документація як Definition of Done

Public translation slice **не завершений**, якщо одночасно не виконані всі умови:

1. identity присутня в `semantic-registry.lisp`;
2. visibility = `public` або `compatibility`;
3. `uk`/`ukr` мають явний status;
4. generated function table синхронний;
5. `docs/ukrainian-api.md` синхронний;
6. README не містить застарілих total-count claims і веде до живого reference;
7. executable witness доводить stable spellings;
8. no-Latin gate проходить для `ukr` identifiers;
9. surface drift gate не бачить unclassified public definitions;
10. generated-files check дає zero diff.

Документація не є постфактум-задачею; вона входить у той самий commit/PR slice, що й public surface change.

## 12. CI gates

Потрібні щонайменше такі перевірки:

### A. Full public inventory

Discover all top-level library/runtime candidates і fail, якщо definition не класифікована `public/internal/compatibility`.

### B. Public identity completeness

Кожна `public` definition мусить мати numeric semantic ID у registry.

### C. Ukrainian coverage

Для кожної public identity:

- `uk` має явний status;
- `ukr` має явний status;
- stable `ukr` не містить ASCII Latin letters;
- stable spellings не колізують між різними semantic IDs.

### D. Runtime peer witness

Репрезентативні та/або generated executable tests доводять, що stable `uk`/`ukr` реально викликають implementation тієї самої identity.

### E. Docs coherence

Повторна генерація function table + Ukrainian API дає zero diff. README contract check забороняє повернення до формули “140 = вся українська surface”.

## 13. Migration strategy

Не робити один giant rename на сотні functions без review.

Міграція йде вертикальними domain slices:

1. inventory infrastructure та fail-closed classification;
2. core/collections/text/vector;
3. time;
4. filesystem/process/host;
5. TCP/network/serialization;
6. UTF-8/text internals, якщо public;
7. logic/unification/reasoning/knowledge/guard;
8. SI/quantity/units;
9. решта public libraries, виявлених inventory.

Для кожного slice:

```text
TECH-SCAN
→ classify public/internal
→ assign/reuse semantic IDs
→ propose uk/ukr
→ naming review
→ RED tests
→ registry/runtime implementation
→ regenerate tables/docs
→ executable witness
→ GREEN CI
```

Старі stable user-facing spellings після ратифікованого rename не видаляються відразу: вони переходять у compatibility alias тієї самої semantic identity, якщо немає окремого рішення про breaking removal.

## 14. Non-goals

Ця робота не означає:

- переклад кожного локального variable name;
- переклад internal helpers;
- додавання semantic ID кожній приватній implementation detail;
- автоматичний машинний переклад без naming review;
- перетворення `ukr` на третю незалежну runtime-мову;
- зміну семантики функції заради красивішої української назви.

## 15. Success criteria

Робота завершена, коли:

1. живий scanner знаходить усю public surface у runtime + `lib/*.lisp`;
2. немає `unclassified` public candidates;
3. кожна public definition має одну numeric semantic identity;
4. кожна public identity має `uk` і `ukr` з явним status;
5. stable `ukr` identifiers не потребують латинської розкладки;
6. stable `uk` compact names проходять правила інтуїтивної читабельності;
7. function table, Ukrainian API reference і README генеруються/перевіряються проти живої authority;
8. цифра coverage походить з inventory, не з prose;
9. stable spellings мають executable witnesses;
10. повний CI зелений.

## 16. Ключовий інваріант

> Якщо ім'я є підтримуваною публічною частиною my-lisp, воно має numeric semantic identity. Якщо identity публічна для українського програміста, її `uk` і `ukr` належать Canon registry. Якщо ім'я є лише деталлю реалізації, воно не роздуває український API.

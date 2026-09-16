# Truth-sentinel inventory / Інвентаризація носіїв істини `t`/`Value::Bool` (#220)

Статус: живий аудит use-site'ів для [#220][220]. Це інвентаризація, не реалізація: жодна класифікація тут не є остаточним семантичним рішенням — рішення про подальшу долю `t` належить власнику мови (#216/#220), не цьому документу.

[220]: https://github.com/juv4uk/my-lisp/issues/220

## Методологія

Кожен сайт нижче перевірений безпосередньо в джерелі (не лише grep-збігом) і, де можливо, підтверджений живим виконанням через `./target/release/my-lisp.exe`. Класифікація — за шістьма категоріями з #220:

```
language semantics | compatibility | host boundary | transport | test-only | dead/stale
```

---

## 1. `Value::truth(...)` — call sites

| # | Місце | Контекст | Класифікація | Примітка |
|---|---|---|---|---|
| 1 | `crates/my-lisp/src/eval/arithmetic.rs:336` | Результат `<`/`=`/`>` числового порівняння | **legacy-semantic** | Прямий предмет #216: "Mathematical YES is `1/1`". Живий, гарячий шлях — щоразу, коли Lisp-код порівнює числа. |
| 2 | `crates/my-lisp/src/eval/builtins.rs:165` (`atom` builtin) | `Value::truth(args[0].is_atom())` | **dead/stale** | Перевірено живим виконанням: `(atom 'x)` → `(structural-kind atom)`, не `t`. Канонічний `atom` резолвиться через `canon.rs`'s reserved-identity dispatch (`atom_value`, #218), який ніколи не викликає цю `define!`-реєстрацію. Кандидат на видалення разом із мертвим імпортом, якщо підтвердиться відсутність інших шляхів виклику. |
| 3 | `crates/my-lisp/src/eval/builtins.rs:343` (`numeric-buffer?`) | Тип-предикат над `Value::NumericBuffer` | **legacy-semantic** (вузький) | Не входить у жодну з #216/#217/#218/#219 ліній явно, але той самий t/() патерн. Малий, ізольований — легкий кандидат для швидкої міграції окремо. |
| 4 | `crates/my-lisp/src/eval/special_forms/strings.rs:144` (`string<?`) | Порівняння рядків | **legacy-semantic** | Той самий клас, що арифметичні порівняння — предикат, що повертає t/(). |
| 5 | `crates/my-lisp/src/eval/special_forms/strings.rs:152` (`string?`) | Тип-предикат | **legacy-semantic** | Той самий клас, що #218's atom/eq — структурний тип-предикат, ще не мігрований. |
| 6-7 | `crates/my-lisp-embed/src/lib.rs:109,163` | Перетворення `MY_LISP_EMBED_TRUE` (сирий `u32`-тег FFI) → `Value::truth(true)` | **host boundary** | FFI-межа коректно ізольована (сирий тег ніколи не витікає в Lisp-простір напряму), але результат після конвертації все одно потрапляє в старий t/()-домен. Коли #220 визначить нову canonical truth-форму, ця конвертація має цілитися в неї, а не залишатися застряглою в t/(). |
| — | `crates/my-lisp/tests/epistemic.rs`, `crates/my-lisp/tests/mccarthy.rs` | Тестові очікування | **test-only** | Класифікується разом із #114's test-authority migration, не окремо тут. |

## 2. `Value::Bool` — call sites (не рахуючи внутрішнього визначення типу)

| # | Місце | Контекст | Класифікація | Примітка |
|---|---|---|---|---|
| 1 | `crates/my-lisp-host/src/lib.rs:424` (`tcp-close`) | `Ok(Value::Bool(true))` — прямий, неконвертований Lisp-видимий результат host-виклику | **host boundary — ПОРУШЕННЯ policy target** | Це точно те, що #220 забороняє: "boundary booleans must be explicitly converted into a named language-domain result rather than leaking into generic truthiness." Зараз `tcp-close` повертає сирий `Value::Bool(true)` напряму як результат Lisp-виразу — жодної named-конверсії. Найконкретніша знахідка цього аудиту, що потребує дії. |
| 2 | `crates/my-lisp-lsp/src/server.rs:502` | Читання `Value::Bool` з LSP-повідомлення | **mechanism** | Інструментальна межа (LSP-протокол), не мовна семантика. |
| 3 | `crates/my-lisp/src/eval/closures.rs:340-341` | `Value::Bool(true) → Symbol("t")`, `Value::Bool(false) → List([])` | **compatibility (потребує явного імені)** | Це буквально жива "t/() пара", яку #220 хоче демонтувати ("`()` is never the paired FALSE for `t`"), закодована в перетворенні на serialization/quote-рівні. Наразі неявна — не позначена як compatibility bridge, на відміну від `migration_only_cond_truthy`. |
| 4-5 | `crates/my-lisp/src/eval/special_forms/json.rs:123-124` | Парсинг JSON `true`/`false` → `Value::Bool` | **transport** | Точний, здоровий приклад transport-межі з #220's policy: зовнішній протокол (JSON) сам є boolean, конвертація в `Value::Bool` тут доречна й не потребує змін. |
| 6-7 | `crates/my-lisp/src/layout.rs:44-45` | NaN-boxing представлення для fpga-lisp бекенду | **mechanism** | Внутрішнє представлення значень для окремого субстрату, не мовна семантика. |
| 8-9 | `crates/my-lisp/src/presentation.rs:117-118` | Друк `Value::Bool` як `"()"` / `"істина"` | **legacy-semantic (непослідовний)** | Примітка: тут `true` друкується як укр. `"істина"`, а не канонічне `"t"` — розбігається з `value.rs:717`, де `Value::Bool(true)` стрінгіфікується як `"t"`. Два різні, неузгоджені друкувальні шляхи для того самого значення — окрема дрібна знахідка, варта окремого issue, якщо #220 не покриє її явно. |
| 10 | `crates/my-lisp/src/value.rs:514` | `PartialEq` для `Value::Bool` | **mechanism** | Звичайна структурна рівність типу даних, не семантичне рішення. |
| 11 | `crates/my-lisp/src/value.rs:717-718` | `Value::Bool(true) → "t"`, `Value::Bool(false) → "()"` (стрінгіфікація для друку/помилок) | **legacy-semantic** | Ще одна жива t/()-пара, окрема від closures.rs. |
| — | `crates/my-lisp/tests/*.rs` | Тестові очікування | **test-only** | Разом із #114. |

## 3. `is_truthy()` — call sites

`is_truthy` (визначений `crates/my-lisp/src/value.rs:625`, `!matches!(self, Value::Nil | Value::Bool(false))`) має рівно **один** живий виклик у некод-семантиці ядра:

| Місце | Контекст | Класифікація |
|---|---|---|
| `crates/my-lisp/src/eval/special_forms/core.rs:59`, усередині `migration_only_cond_truthy` | Fallback-гілка для значень, що не збігаються з жодним відомим structural/identity-record | **compatibility — вже правильно назване** |

Це найкраща новина цього розділу: `migration_only_cond_truthy` — явно задокументований, іменований, тимчасовий міст ("Temporary bridge for historical two-part `cond` only… Once two-part `cond` is retired, this adapter disappears with it"). Саме так має виглядати compatibility-категорія з #114/#220 — це вже зразок, з яким варто звіряти будь-які майбутні bridge.

Решта викликів `is_truthy` — тести (`forward.rs`, `mccarthy.rs`) і license-policy fixture (`xtask/tests/license_policy.rs`, яка лише перевіряє назву тесту як рядок) — **test-only**.

## 4. Символ `t` — не в semantic-registry

`t` зв'язаний у `Environment::root()` (`crates/my-lisp/src/environment.rs:69`) як звичайний self-evaluating symbol: `environment.define("t", Value::Symbol(Rc::from("t")))`. Перевірено: **`t` відсутній у `lib/surface/semantic-registry.lisp`** — тобто, на відміну від зарезервованих написань Canon0+7 (contract 6.0), `t` НЕ захищений від затінення `def`/`lambda`. Це прямо підтверджує рамку #220: `t` — звичайний символ, а не мовний авторитет істини, і будь-яке рішення про його подальшу долю (ordinary symbol / deprecated spelling / compatibility alias) — відкрите семантичне питання, не вирішене цим аудитом.

Пряме перелічення кожного літерального `t` у `.lisp`-джерелах (їх тисячі — очікуваний t/() як результат предикатів по всьому core/test corpus) не додає користі без структурної класифікації; ця робота природно збігається з #114's test-authority migration (де кожен тест і кожна семантична форма вже проходять через ту саму класифікацію по "хто визначає очікувану істину").

## Підсумок — найважливіші дії

1. **`atom` builtin у `builtins.rs` — мертвий код.** Підтверджено живим виконанням. Перевірено й `lib/meta-eval-first-class.lisp`'s `(atom (car args))` — він також лише викликає символ `atom` через звичайний environment-lookup, тобто резолвиться до того самого canonical reserved-identity шляху, не до мертвої реєстрації. Кандидат на видалення разом із мертвим імпортом `Value::truth` у цьому файлі (лише якщо саме ця реєстрація не потрібна як fallback для середовищ без canonical-identity dispatch — це варто підтвердити перед видаленням, а не цим документом).
2. **`tcp-close` повертає сирий `Value::Bool(true)` без named-конверсії — найконкретніше порушення policy target.**
3. **Дві незалежні, неявні t/()-пари** (`closures.rs:340-341`, `value.rs:717-718`) роблять те саме перетворення різними шляхами — варто об'єднати чи явно позначити обидві як компонент однієї compatibility-межі, коли #220 вирішить долю `t`.
4. **`presentation.rs` друкує `true` як `"істина"`, а `value.rs` — як `"t"`** — неузгодженість, знайдена побічно, не предмет #220 напряму, але варта окремого запису.
5. **`migration_only_cond_truthy` — зразковий приклад** того, як має виглядати compatibility bridge; нові bridge-и варто моделювати за ним.

## Не входить у цей аудит

- Повний перелік кожного літерального `t`/`()` у `.lisp`-корпусі (тисячі входжень) — покривається класифікаційною роботою #114, не тут.
- Swarm/JSON-протокол за межами вже перевіреного `json.rs` — grep не знайшов додаткових `Value::truth`/`Value::Bool`/`is_truthy` сайтів у `crates/swarm-node` чи `crates/my-lisp-wasm`.
- Семантичне рішення про майбутнє `t` (символ/deprecated/compatibility) — явно залишається за #220's власником, не цим документом.

## English summary

This document inventories every current use of the self-evaluating symbol `t`, `Value::truth(...)`, `Value::Bool`, and `is_truthy()` across the `my-lisp` workspace, classified per issue #220's six-bucket schema. The most significant finding: the `atom` builtin registered in `crates/my-lisp/src/eval/builtins.rs` is **dead code** — canonical evaluation resolves `atom` through the reserved-identity dispatch in `canon.rs` (returning `(structural-kind ...)`), never reaching the `Value::truth`-based registration. `eq` has already migrated to `(identity-relation same|distinct)`. A single, already-named compatibility bridge (`migration_only_cond_truthy` in `core.rs`) exists specifically to keep historical two-part `cond` working during this transition, and is the correct model for any further compatibility shims this migration needs.

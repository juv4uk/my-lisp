# ADR-011 — Межа контрактності деталей помилки

Статус: прийнято для поточного scope evidence мета-evaluator-а · 2026-09-10

## Контекст

Issue #26 вимагає `error-detail-parity` **там, де деталі є спостережуваною частиною контракту**, але окремо забороняє мовчки відкидати розбіжності нормалізацією. Якщо деталі навмисно неконтрактні, це треба сказати прямо.

Поточне мовне правило вже проводить цю межу. S2 вимагає, щоб кожна помилка мала названу спостережувану категорію, і прямо каже, що формулювання може відрізнятися: **контрактом є категорія**. `LanguageError` зберігає `kind`, `message` і `span`, але саме `ErrorKind` є контрактною віссю; presentation/classification дані можуть еволюціонувати окремо.

Lisp meta-evaluator представляє помилки як Lisp-дані:

```lisp
(error kind detail)
```

Його `detail` корисний як діагностична структура: наприклад, точна/мінімальна арність або причина некоректного lambda-list. Rust reference evaluator не має ратифікованого структурованого поля, яке відповідало б цьому payload; він має людинозрозумілий `message` та source `span`.

Якби Rust prose або байтові offsets стали мовною семантикою, self-hosting parity залежав би від формулювань, локалізації та конкретного написання source. Але й видаляти структуровані Lisp-деталі лише заради зовнішньої схожості теж неправильно.

## Рішення

Для поточного мовного контракту й meta-eval evidence matrix:

1. **Контрактним cross-runtime спостереженням є названа категорія помилки.** Reference `ErrorKind` порівнюється з відповідною meta категорією (`UnknownSymbol ↔ unbound-symbol`, `Type ↔ not-callable`, `Arity ↔ arity`, `InvalidForm ↔ invalid-form`).
2. **`LanguageError.message` — діагностичний/presentation текст, а не cross-runtime semantic field.** Differential тести не повинні парсити або копіювати prose, щоб штучно створити parity.
3. **`LanguageError.span` — metadata місця в source, а не cross-runtime semantic field.** Різні surface spellings та кодування природно дають різні byte ranges для тієї самої категорії помилки.
4. **`detail` meta-evaluator-а залишається доступним як Lisp diagnostic data, але спільної cross-runtime detail schema сьогодні не ратифіковано.** Exact shape можна захищати implementation-level regression tests, не роблячи його мовною conformance-вимогою.
5. Майбутній контракт може ратифікувати structured detail schema. Тоді вона мусить бути явною, машинно-читаною й implementation-neutral. Додавати зараз Rust `ErrorDetail` лише для закриття evidence row — зайве розширення механізму.

Коротко:

```text
мовна семантика        діагностичні поверхні
----------------       ---------------------
error category         Rust message
                       Rust source span
                       meta Lisp detail payload
```

Праву колонку можна й треба тестувати локально. Але вона не нормалізується мовчки у фальшиве твердження про cross-runtime equality.

## Виконуваний доказ

`crates/my-lisp/tests/meta_eval_error_detail_boundary.rs` фіксує цю межу:

- перевіряє, що S2 і далі явно говорить про category-over-wording;
- перевіряє, що reference implementation називає `kind` контрактною віссю;
- показує, що еквівалентні EN/UK Canon-rebinding помилки можуть мати різні source spans, але одну категорію `InvalidForm`;
- зберігає structured arity/lambda details у meta-evaluator замість їх видалення.

Існуючі paired error tests продовжують доводити correspondence категорій. Їхні вручну нормалізовані Lisp detail expectations є regression witnesses для meta representation, а не твердженням, що Rust human prose має ту саму структуру.

## Наслідки

- Не додається parser для native error messages.
- Localization text не копіюється в `lib/meta-eval.my`.
- Source-span equivalence між reference evaluator та meta-evaluator не вимагається.
- Корисні структуровані Lisp diagnostics зберігаються.
- `error-detail-parity` може бути підтверджений лише як **явний результат визначення межі**: зараз немає додаткових ратифікованих cross-runtime detail fields понад уже підтверджену категорію.
- Це рішення саме по собі **не дозволяє** називати проєкт `complete self-hosting`; claim vocabulary керується окремо evidence matrix та політикою проєкту.

## Правило майбутнього розширення

Якщо реальному consumer-у знадобляться стабільні machine-readable details — наприклад `expected=2, received=1` або host `operation/reason` — спочатку треба додати явний contract/ADR та implementation-neutral representation. Лише після цього поле може стати частиною cross-runtime parity.

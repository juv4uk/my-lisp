# Карта міграції identity `Value::Builtin` — актуальний статус

Цей шлях збережено як стабільний покажчик, бо на нього посилаються чинні плани й коментарі в коді.

Оригінальний документ від 2026-09-11 описував **дослідницький стан до реалізації PR #95**, коли канонічні callable-значення ще були представлені Rust-об’єктами `Value::Builtin`, а `Value::SemanticRef` ще не існував. Повний історичний текст тепер збережено тут:

`docs/archive/historical/BUILTIN-IDENTITY-MIGRATION-MAP-2026-09-11.md`

## Поточний стан — 2026-09-13

PR #95 (`Canon SemanticRef: RED witness for semantic callable identity`) зробив реалізаційні твердження старого документа неактуальними.

Чинні спостережувані й архітектурні факти:

- identity канонічного callable — це числовий semantic ID з мовного реєстру, а не Rust allocation/pointer;
- канонічні callable-значення матеріалізуються як `Value::SemanticRef(id)`;
- admitted peer surfaces одного Canon semantic ID є `eq`, а різні semantic IDs залишаються різними;
- evaluator під час application розв’язує `SemanticRef` у поточну implementation projection;
- `TAG_PRIMITIVE` несе числовий semantic ID, а host-only builtin pointers мають окремий host tag;
- невідомий semantic callable ID завершується fail-closed;
- legacy non-Canon `Value::Builtin` може лишатися host-механізмом, але Rust identity такого об’єкта не є Canon semantic identity.

Архівна research map лишається корисною як provenance того, **чому pointer identity було відкинуто** і які migration seams були знайдені до реалізації. Вона не є нормативною й не може перекривати чинні Canon, `semantic-registry.lisp`, language contracts або executable tests.

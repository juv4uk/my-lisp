# Аудит поточної runtime-моделі GC — #153

Статус: лише evidence/audit; без реалізації колектора.
Дата: 2026-09-15

## Навіщо цей аудит

Консенсус GC від 2026-08-23 обрав Pair-only як перший managed slice. Відтоді runtime суттєво змінився: callable semantic identity рухається до `SemanticRef`, рекурсивні визначення використовують спільні lexical environments, а machine/backend напрямок уже потребує переносимого розділення між semantic identity, runtime object identity і фізичною адресою.

Цей документ фіксує поточні факти володіння до того, як #154 змінить storage.

## Поточний граф володіння

```text
Value
├─ immediate / non-graph semantic values
│  └─ SemanticRef(id)          semantic identity; never heap identity
├─ Pair                        Rc<Value> × Rc<Value>
├─ Closure / Macro             Rc<Closure> → captured Environment
├─ Vector                      Rc<RefCell<Vec<Value>>>
├─ Symbol / String             Rc<str>
├─ Rational / BigInt           immutable numeric payload
└─ host/resource payloads      explicit host lifecycle; not GC-finalizer authority

Environment
└─ Rc<RefCell<Frame>>
   ├─ values: name → Value
   └─ parent: Option<Environment>
```

Поточний `Environment` уже має ітеративний `Drop` і regression test на 300 000 frame-рівнів. Тому deep parent-chain destruction **не є** актуальною причиною вводити tracing GC.

## Актуальні причини для tracing GC

1. **Цикли.** Closure захоплює `Environment`; цей environment може містити binding на сам closure. Shared recursive definition frames роблять такий граф природним, а не винятковим.
2. **Мутабельні графові об'єкти.** `Vector` може утримувати `Value` і після появи back-edge брати участь у циклах.
3. **Переносима runtime identity.** Майбутня CML/FPGA object memory не повинна успадковувати Rust `Rc` pointer identity як факт мови.

Канонічне розділення identity:

```text
SemanticRef(id) != ObjectId(slot,generation) != physical address
```

- `SemanticRef` відповідає на питання **що це означає / яка це операція?**
- `ObjectId` має відповідати **який саме це managed runtime object?**
- фізична адреса є лише backend-механізмом.

## Класифікація для #153

| Runtime class | Поточне storage | M0-класифікація | Причина |
|---|---|---|---|
| Nil/Bool/simple numeric immediates | inline | non-traced | немає graph edges |
| SemanticRef | semantic ID | non-traced semantic identity | не повинен отримати heap/pointer authority |
| Pair | Rc graph | managed candidate | рекурсивний graph node |
| Closure/Macro | Rc + captured Environment | managed candidate | бере участь у природних env cycles |
| Environment/Frame | Rc/RefCell | managed candidate | володіє bindings і parent edges |
| Vector | Rc/RefCell | managed candidate | mutable aggregate; можливі cycles |
| Symbol/String | Rc<str> | defer | immutable payload; немає graph edges |
| Rational/BigInt | owned immutable payload | defer + measure | exact-arithmetic pressure окремий від cycle collection |
| Host/resource handles | host-owned | explicit-resource | nondeterministic finalization не повинна володіти close semantics |

## Decision gate

Старий Pair-only consensus зберігається як historical design capital, але вже не є достатнім evidence для реалізації. #153 має обрати перший managed slice на основі поточних cycle witnesses і вартості міграції.

Провідний кандидат — **graph-core** slice:

```text
Pair + Closure + Environment + Vector
```

Це кандидат, а не implementation authority. #154 не повинен переносити ці типи, доки #153 не буде ратифіковано.

### Що ще треба довести до #154

- executable witness для `Environment ↔ Closure` strong-reference cycle;
- executable witness або точний API-аудит, що показує, чи `Vector` сьогодні реально може утворити back-edge;
- inventory усіх `Value` variants, щоб managed/deferred/resource класифікація була повною, а не вибірковою;
- підтвердження, що host resource handles мають explicit close/drop policy поза semantic GC authority.

Без цих чотирьох пунктів graph-core лишається гіпотезою, а не дозволом на переписування storage.

## Обов'язкові portability constraints

1. Жоден raw Rust pointer не є semantic identity.
2. Managed references повинні допускати stable backend-neutral handles; `slot + generation` лишається провідним представленням.
3. Host handles зберігають explicit lifecycle semantics; GC може робити diagnostics/fallback cleanup, але не визначати language-visible момент `close`.
4. Увімкнення/вимкнення collector або зміна stress-frequency не повинні змінювати Lisp value/output/error semantics.
5. Той самий reachability/object contract має бути представимий у native Rust, CML і майбутній FPGA Lisp-machine.

## Порядок реалізації, що відповідає поточній архітектурі

```text
#153 current ownership/cycle evidence
  ↓
#154 ValueStorage facade (behavior-preserving)
  ↓
#155 ManagedHeap + ObjectId/generation
  ↓
#156 explicit roots + safe points
  ↓
#157 exact non-moving mark/sweep + stress/metamorphic proof
  ↓
#158 backend-neutral object contract
```

Жодного collector-коду не повинно потрапити в #153.

---

## English mirror

This audit records the current ownership model before #154 changes storage. The old Pair-only M0 decision is retained as historical design capital but is no longer sufficient implementation authority after `SemanticRef`, shared recursive environments, and the machine/backend portability work. The current leading managed-graph candidate is `Pair + Closure + Environment + Vector`, subject to executable cycle evidence, complete `Value` inventory, and explicit resource-lifecycle verification in #153. The key invariant is `SemanticRef(id) != ObjectId(slot,generation) != physical address`; host resources retain explicit lifecycle semantics, and GC stress must not change observable Lisp semantics.
# CODE-SURVEY-2026-09-01 — огляд коду my-lisp

**Виконавець:** wsl-nidana-1
**Метод:** реальне читання (не grep) найбільших/найцентральніших файлів:
`crates/my-lisp/src/eval/mod.rs`, `value.rs`, `environment.rs`,
`lib/world.my`, `lib/reason.my`. Не exhaustive.

## `eval/mod.rs` — справжній trampoline, не рекурсія Rust

`evaluate_step` повертає `EvalStep::Value` або
`EvalStep::TailCall{expression, environment}`; `evaluate` крутить цикл на
останньому, поки не отримає значення. Так tail calls реально уникають
росту Rust-стеку — genuine TCO, не декларація про намір. Special forms
(`quote`/`lambda`/`def`/`defmacro`/`cond`/io/string ops) матчаться напряму
на head symbol ДО будь-якого environment lookup — той самий CANON-суміжний
порядок диспетчеризації, вже верифікований раніше цієї сесії в інших
місцях. Все інше падає у звичайне застосування: builtins отримують
попередньо обчислені аргументи (contract 2.1, shadowable), user-closures
йдуть через `closures::apply`.

## `value.rs` — плаский enum, `Rc`-based cons, без GC/арени

`Value` — плаский Rust enum (Nil/Bool/Number/Rational/String/Symbol/
Pair/Closure/Macro/Builtin/Vector/NumericBuffer/TcpConnection/TcpListener).
`Pair(Rc<Value>, Rc<Value>)` — cons-клітинки це просто дві `Rc`, без
арени/GC. Точні числа — `Rational` над hand-rolled bignum (`bignum.rs`) з
типізованою `DecimalLiteralError`, щоб відмова через ресурсний ліміт ніколи
не могла тихо деградувати числовий літерал у звичайний symbol. `Builtin` —
`Rc<dyn Fn(&[Value], &Environment, Span) -> Result<Value, LanguageError>>` —
genuinely first-class, зберігається як звичайний environment binding, не
запис у спеціальній head-таблиці.

**Реальний engineering-виверт:** `impl Drop for Value` вручну ітеративно
обходить `Pair`-ланцюжки через worklist + `unsafe { ptr::read }`/
`ManuallyDrop` — свідомий обхід stack overflow від дефолтного рекурсивного
`Drop` на довгих списках (drop 100k-елементного списку інакше зніс би
стек).

## `environment.rs` — той самий клас ризику, ще не виправлений

`Environment` = `(Rc<RefCell<Frame>>, Rc<RefCell<Transcript>>,
Rc<RefCell<Limits>>)`; `Frame{values: HashMap, parent: Option<Environment>}`.
Lexical scoping — чесний linked list hashmap-фреймів. `Transcript` і
`Limits` — спільні `Rc` через ВСЕ дерево environment сесії, не per-frame —
тобто `print` closure-а й далі падає в один session-wide transcript.

**Коментар у самому файлі вже самопозначає той самий ризик**, який `Value`
вже виправив: дроп глибокого `Environment`-ланцюжка рекурсує через
дефолтний `Drop` `Rc<RefCell<Frame>>` і може переповнити стек — **на
відміну від `Value::Pair`, тут це НЕ обійдено, лише задокументовано** як
відомий, зараз неактивний ризик (стає реальним лише при глибокому
`let`/currying-вкладенні). **Заведено як
`WSM-ENVIRONMENT-DEEP-DROP-STACK-SAFETY` у `tasks.my`.**

## `lib/world.my` — immutable World це просто `cons`

"Immutable World" — не Rust-структура взагалі: `(world parent journal
metadata)` — звичайний WSM-список. `world-tell`/`world-retract` ніколи не
мутують — `world-record` cons-ить новий запис журналу на ІСНУЮЧИЙ журнал
(`(cons event (world-journal world))`), тож старий і новий світи ділять
увесь хвіст структурно, безкоштовно, через звичайний `cons`. Жодного
Rc-cycle бухгалтерства не треба — `Rc<Value::Pair>` під капотом уже дає це
поділення.

## `lib/reason.my` — справжній micro-Prolog

`prove-goal` рекурсує по списку правил, роблячи backward-chaining
унифікацію (`lib/unify.my`), з `rename-vars`, що тегує кожну змінну
лічильником `depth` (`(var x)` → `(var (x . 3))`) для standardize-apart
рекурсивних інстанціацій правил — коректна, мінімальна реалізація
класичного Prolog-трюку ізоляції змінних, не hand-waved.

## Позначені занепокоєння

1. Ручний unsafe-обхід `Value::Drop` — крихкий boilerplate: майбутнє
   додавання варіанту `Value` могло б мовчки його обійти, якщо не
   розширити разом.
2. `Environment`'s deep-chain-drop ризик — задокументований, але
   genuinely не пом'якшений — той самий клас бага, що (1), просто ще не
   спрацював.
3. `HashMap`-per-frame lexical lookup — O(depth) лінійний обхід
   parent-ланцюжка на кожен miss. Нормально при поточному масштабі,
   мало б значення для глибоко вкладених closures у гарячому циклі.

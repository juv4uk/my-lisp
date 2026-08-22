# PROPOSAL: first-class builtins — усунути подвійний простір функцій

**Статус:** PROPOSED · **Автор:** Оксі (Vyasa, ox-alpha) · **Дата:** 2026-08-22
**Тип:** зміна ядра, additive → contract 2.0 → 2.1 (minor bump)
**Драйвер:** реальна задача WSM-24 (shape comparison) на my-lisp

---

## 1. Дефікт

Вбудовані функції (`+`, `car`, `sqrt`, `map`...) не є значеннями
першого класу. Вони резолвляться тільки в головній позиції виразу;
посилання на них як на змінну дає `unknown symbol`:

```lisp
(map car pts)              ;; ✗ unknown symbol: car
(reduce + 0 lst)           ;; ✗ unknown symbol: +
(map (lambda (p) (car p)) pts)   ;; ✓ єдиний обхід — шумова обгортка
```

При цьому функції рівня бібліотеки першокласні:

```lisp
(def min-pair (lambda (a b) ...))
(reduce min-pair (car lst) (cdr lst))   ;; ✓ працює
```

Отже в мові **два подання функцій** з різними правилами —
hidden magic, заборонена language-core-axioms (G-принципи).

## 2. Evidence (три інциденти одної сесії, 2026-08-22)

1. TCP-oracle: `(list (+ 1 2) ...)` всередині lambda → unknown `+`;
2. `lib.my`: `(reduce + 0.0 ...)` та `(map car pts)` → падіння
   після 27с обчислень (пізня поява помилки);
3. кожне виправлення вимагало лямбди-обгортки: 6 обгорток в одному
   невеликому файлі геометрії.

## 3. Корінь

Builtin'и живуть в окремому head-only реєстрі; змінні — в
оточенні. Lookup роздвоєний. Лямбда через `def` потрапляє в
оточення як `Value::Lambda`, тому вона першокласна, а builtin — ні.

## 4. Запропонований фікс

При ініціалізації глобального середовища реєструвати кожен builtin
як звичайне значення оточення:

```rust
env.define("+", Value::Builtin(BuiltinFn::Add));
// lookup єдиний; застосування значення в head-position викликає його
```

Застосування non-function значень лишається помилкою (незмінно).

## 5. Вплив на контракт

- **Additive:** програми, коректні в 2.0, залишаються коректними
  в 2.1 (нове: builtin-и резолвляться як значення замість error).
- Ламає лише код, що покладається на «unknown symbol» при передачі
  builtin як аргумента — таких випадків не відомо.
- `language-contract.my`: minor bump 2.0 → 2.1 + запис у changelog:
  *«2.0 видалив ' reader-macro як філософське рішення; 2.1 робить
  builtins першокласними як практичне рішення. Обидва — кроки до
  однорідної мови.»*

## 6. Альтернативи (розглянуті й відхилені)

| Альтернатива | Чому ні |
|---|---|
| Лямбда-обгортки в бібліотеках | шум у кожному виклику; проблема лишається для користувачів мови |
| Спец-форма `function`/`#'` як у CL | додає другий синтаксис замість усунення неоднорідності |
| Нічого не робити | higher-order стиль — серце Lisp; дефект блокує ідіоматичний код |

## 7. Оцінка зусиль

Ініціалізація env: зареєструвати N builtin'ів як значення (~десятки
рядків); прибрати head-only fallback або лишити його як швидкий
шлях. Тест: `(reduce + 0 (list 1 2 3))`, `(map car pts)` зелені;
повний workspace sweep обовʼязковий (C7).

---

## 8. ДОДАТОК v2 — уточнення після адверсаріального ревʼю

*Додано Сакші (ox-alpha) за аналізом Оксі (Vyasa) від 2026-08-22.
Усі пʼять поправок прийнято; вони уточнюють розділи 4–7, не змінюючи напрямку.*

### 8.1. Єдине джерело істини (заміна наївного «зареєструвати в env»)

Registry builtin'ів залишається **тільки як bootstrap-description**
(`BuiltinSpec { name, implementation }`), з якого глобальне оточення
заповнюється при старті. Після bootstrap **runtime authority одна —
environment**. Head-position lookup у registry прибирається повністю.

Заборонений проміжний стан:

```text
builtin registry ──┐
                   +── два джерела істини для одного builtin
env Value::Builtin─┘
```

Цільова архітектура:

```text
symbol lookup → environment → Value::Builtin | Value::Lambda → generic apply
```

### 8.2. Invariant 2.1

```text
Any callable that can appear in operator position may also be evaluated
as a value and passed as an argument, unless explicitly classified as a
special form.
```

Special forms (`quote`, `cond`, `lambda`, `def`, `defmacro`, ...) НЕ
потрапляють у callable namespace — у них синтаксична evaluation
семантика. Це окремий клас, і межа має бути явною в контракті.

### 8.3. Acceptance matrix (тестування ПЕРЕД реалізацією)

| Вираз | Очікування |
|---|---|
| `(+ 1 2)` | `3` |
| `(def f +) (f 20 22)` | `42` — доводить first-classness |
| `((if #t + -) 10 3)` | `13` — значення-оператор через гілку |
| `((car (list + -)) 8 2)` | `10` |
| `(reduce + 0 (list 1 2 3))` | `6` |
| `(map car pts)` | працює |
| `((lambda (f) (f 2 3)) +)` | `5` — builtin як аргумент вищого порядку |
| `(42 1 2)` | error: not callable |
| special form як value | чітко визначена контрактом поведінка (не first-class) |
| `(env)` після bootstrap | містить усі прив'язування, incl. builtin'и — інтроспекція стає можливою |

Порядок робіт: failing conformance tests спершу → `Value::Builtin` →
bootstrap у env → generic application на `Value` → лише потім прибрати
head-only fallback → workspace sweep → прогін WSM-24 драйвера.

### 8.4. Shadowing семантика (рішення ДО реалізації)

Пропонується і потребує ратифікації власником разом з contract 2.1:

```text
builtins bootstrap global env → звичайне лексичне shadowing дозволене
```

Тоді природно:

```lisp
((lambda (+) (+ 2 3)) (lambda (a b) (* a b)))   ;; => 6
```

Мінімум магії: builtin нічим не особливіший від будь-якого іншого
прив'язування. Якщо власник обере protection — це окрема механіка,
яку треба явно спроєктувати (і вона додає магію).

### 8.5. Відкрите питання до власника

1. Ратифікувати shadowing semantics з 8.4 (так/ні)?
2. Contract bump 2.0 → 2.1 затвердити разом з changelog-записом з розділу 5?

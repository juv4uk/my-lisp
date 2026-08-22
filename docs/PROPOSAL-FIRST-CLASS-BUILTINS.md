# PROPOSAL: first-class builtins — усунути подвійний простір функцій

**Статус:** PROPOSED v2 · **Автор:** Оксі (Vyasa, ox-alpha) · **Дата:** 2026-08-22
**Тип:** зміна ядра → contract 2.0 → 2.1 (minor bump, див. §7)
**Драйвер:** реальна задача WSM-24 (shape comparison) на my-lisp
**Ревʼю:** другий агент (opencode pts/2), 2026-08-22 — прийнято з
поправками §2/§5/§8 цього документа

---

## 1. Дефікт

Вбудовані функції (`+`, `car`, `sqrt`...) резолвляться тільки в
головній позиції виразу; посилання на них як на значення дає
`unknown symbol`. Функції бібліотеки (`def` + `lambda`) —
першокласні. Два подання функцій з різними правилами = hidden magic.

```lisp
(map car pts)              ;; ✗ unknown symbol: car
(reduce + 0 lst)           ;; ✗ unknown symbol: +
(map (lambda (p) (car p)) pts)   ;; ✓ обхід лямбдою (6 обгорток за вечір у WSM-24)
```

## 2. Цільова архітектура (виправлено після ревʼю)

НЕ копіювати builtin registry в оточення (інакше отримаємо два
джерела істини замість двох просторів). Registry лишається як
**bootstrap-description**, runtime authority — одна:

```text
symbol lookup
     ↓
environment            ← єдиний runtime authority
     ↓
Value::Builtin / Value::Lambda / ...
     ↓
generic apply
```

```text
BuiltinSpec { name: "+", implementation: Add, ... }
```
існує тільки як дані для bootstrap, після старту в runtime його нема.

## 3. Інваріант 2.1

> Any callable that can appear in operator position may also be
> evaluated as a value and passed as an argument, **unless explicitly
> classified as a special form.**

Special forms (quote, cond, lambda, def, defmacro, ...) НЕ стають
першокласними — вони синтаксичні правила eval, а не callable values.
Перевірка випадкового потрапляння — окремий крок реалізації (§8.2).

## 4. Acceptance matrix (зелений = 2.1 готовий)

| Вираз | Очікування |
|---|---|
| `(+ 1 2)` | `3` |
| `(def f +) (f 20 22)` | `42` |
| `(reduce + 0 (list 1 2 3))` | `6` |
| `(map car pts)` | працює |
| `((lambda (f) (f 2 3)) +)` | `5` |
| `((car (list + -)) 8 2)` | `10` |
| `((if cond + -) 10 3)` | за cond |
| `(42 1 2)` | помилка «not callable» |
| special form як value | чітко визначена контрактом поведінка |

Мінімальний доказ first-classness: `(def f +) (f 20 22)` → 42.
`(reduce + ...)` сам по собі може пройти через спеціальний код reduce —
недостатній тест.

## 5. Shadowing semantics (вирішено ДО реалізації, per ревʼю)

Питання: що робить `(def + ...)` після 2.1?

**Рішення:** builtins bootstrap global env → далі діє нормальне
лексичне затінення, без захисту:

```lisp
((lambda (+) (+ 2 3)) (lambda (a b) (* a b)))   ;; => 6, природно
```

Обґрунтування: «мінімум магії». Захищені builtin'и = ще один
спеціальний режим; лексичне затінення — вже існуюча семантика мови.

## 6. Відхилені альтернативи

| Альтернатива | Чому ні |
|---|---|
| Лямбда-обгортки в бібліотеках | шум; проблема лишається для користувачів |
| `function`/`#'` як у CL | додає третій механізм замість усунення неоднорідності |
| Копія registry в env | два джерела істини для одного builtin (ревʼю §2) |
| Нічого не робити | higher-order стиль — серце Lisp |

## 7. Контракт

Additive: програми, коректні в 2.0, коректні в 2.1 — КРІМІ випадків,
що покладаються на «unknown symbol» при передачі builtin (невідомі).
Minor bump 2.0 → 2.1 **підтверджується ПІСЛЯ** тесту shadowing
семантики на реальному дереві (conformance fixtures + yantra + core.my),
не автоматично.

## 8. Порядок реалізації (per ревʼю)

1. Failing conformance tests ПЕРШИМИ (матриця §4), не код;
2. Аудит: жодна special form не потрапляє в callable namespace;
3. Запровадити `Value::Builtin`;
4. Bootstrap builtins в environment (registry → bootstrap-only);
5. Перевести generic application на `Value`;
6. Лише тепер прибрати head-only fallback;
7. Повний workspace sweep (C7);
8. Прогнати WSM-24 driver — задачу, через яку дефект відкрився.

## 9. Evidence (три інциденти сесії 2026-08-22)

1. TCP-oracle: `+` невідомий всередині lambda;
2. `lib.my`: `(reduce + ...)`, `(map car pts)` — падіння після 27с
   обчислень;
3. 6 lambda-обгорток в одному невеликому geometry-файлі.

## 10. Оцінка зусиль

Bootstrap-реєстрація (~десятки рядків) + уніфікація apply + тести.
Філософський підсумок (per ревʼю): якщо щось поводиться як функція,
мова не повинна змушувати користувача знати, де саме в Rust воно
було реалізовано.

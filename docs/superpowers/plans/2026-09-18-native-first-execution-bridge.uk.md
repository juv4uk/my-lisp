# План Native-First execution bridge

> **Для агентних виконавців:** використовуйте `superpowers:subagent-driven-development` або `superpowers:executing-plans`.

**Мета:** виконувати native plans з #505 на CPU, а evaluator fallback — через reference evaluator, з явним provenance маршруту в Lisp.

**Архітектура:** Lisp-функція отримує expression data і викликає `native-first-plan`. `native-plan` іде через чинний closed admission/native-call; `evaluator-fallback` виконує незмінний вираз через `eval`. Помилки вже прийнятого native plan не маскуються fallback-ом.

## Обмеження

- Жодного нового Rust semantic matcher/code.
- Не змінювати semantic registry/Canon.
- Native виконується лише через чинний closed admission.
- Fallback лише після явного `evaluator-fallback`.
- Результат: `(execution-route native VALUE)` або `(execution-route evaluator VALUE)`.

## RED

Створити `tests/fixtures/native-first-execution-witness.lisp`:

- `(car (cons 2 3))` -> native route, 2;
- `(+ 2 3)` -> evaluator route, 5;
- dynamic CAR/CONS -> evaluator route, 2;
- quoted expression-data `radio` -> evaluator route, radio.

Запустити verification-only workflow до появи bridge-файлу.

## GREEN

Створити `lib/machine/dispatch/native-first-exec.lisp`:
- `native-first-plan`;
- explicit plan-tag dispatch;
- native -> `x86-call-admitted-u64`;
- fallback -> `eval`;
- route provenance;
- без catch-and-silent-fallback для native errors.

Потім exact-head CI+bilingual і передача evidence в #504/#506/#509.

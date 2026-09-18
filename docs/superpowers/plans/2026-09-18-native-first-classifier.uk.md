# План реалізації Native-First classifier

> **Для агентних виконавців:** використовуйте `superpowers:subagent-driven-development` або `superpowers:executing-plans`.

**Мета:** додати тотальний Lisp-owned native-plan classifier з явним evaluator fallback і одним доведеним native island.

**Архітектура:** classifier отримує expression data, не source text і не host AST. Він повертає structured machine forms або незмінний вираз для fallback. Використовуються чинні Lisp-owned lowering/operand/layout шари; байти тут не виконуються.

**Специфікація:** `docs/superpowers/specs/2026-09-18-native-first-execution-design.uk.md`

## Обмеження

- Немає semantic matcher у Rust.
- Не змінювати semantic registry/Canon.
- У #505 немає raw native execution.
- Не редагувати machine-form файли #483/#484.
- Unsupported -> fallback, не error.
- Production code лише після RED.

## Завдання 1 — RED

Створити `crates/my-lisp/tests/native_first_dispatch.rs`, який умовно завантажує dispatcher, якщо файл уже існує.

Перевірити:
- `(car (cons 2 3))` -> native plan + existing forms + 16-byte arena;
- addition, dynamic CONS fields, negative field, atom і malformed data -> exact evaluator fallback.

Запустити focused test і зафіксувати RED через відсутній `native-first-plan`.

## Завдання 2 — мінімальна реалізація

Створити `lib/machine/dispatch/native-first.lisp`.

- Result constructors.
- Exact CAR(CONS u64 u64) shape.
- Reuse `x86-as-u64-imm` та `x86-lower-cons-car-u64-forms`.
- У fallback завжди зберігати original expression.
- Не encoding bytes і не host call.

Після цього focused test має стати GREEN.

## Завдання 3 — integration

Diff hygiene, exact-head CI, bilingual gate, draft PR і coordination notes до #504/#506/#509.

# План differential parity gate для Native-First

> **Для агентних виконавців:** використовуйте `superpowers:subagent-driven-development` або `superpowers:executing-plans`.

**Мета:** кожен admitted native-first island мусить довести точну parity одночасно з reference evaluator та незалежним Lisp-owned expected evidence, а також довести, що маршрут справді був native.

**Архітектура:** gate залишається Lisp-owned. Corpus містить expression data, незалежне expected value та явні effect/error classes. Runner виконує кожен рядок через evaluator і через `native-first-execute-expression`; native success приймається лише якщо обидва збігаються з independent expected value, а provenance рівно `execution-route native (status completed)`.

**Специфікація:** `docs/superpowers/specs/2026-09-18-native-first-execution-design.uk.md`.

## Обмеження

- Жодного Rust semantic matcher.
- Canon і semantic registry не змінюються.
- Native failure після вибору native-plan не маскується evaluator fallback.
- Native та evaluator не можуть бути єдиними oracle одне для одного.
- Поточний native island: bounded `(car (cons <u64-literal> <u64-literal>))`.
- Поточний island pure, error-class = `not-applicable`.
- Поточний native bridge повертає exact integer максимум до `9007199254740991`; це значення входить у corpus як boundary.
- #507 компілює encoder, #508 веде coverage ledger; цей task їхні файли не чіпає.

## Завдання 1 — RED

Створити `tests/fixtures/native-first-parity-witness.lisp` з чотирма cases:

```text
(car (cons 0 1)) -> 0
(car (cons 2 3)) -> 2
(car (cons 42 99)) -> 42
(car (cons 9007199254740991 7)) -> 9007199254740991
```

Кожен рядок має `effect pure` та `error not-applicable`. Fixture завантажує ще відсутній `native-first-parity.lisp`. Додати witness до `scripts/test-current-semantic-slice.sh` і зафіксувати RED через відсутній runner.

## Завдання 2 — GREEN

Створити `lib/machine/dispatch/native-first-parity.lisp`:

- direct `eval` для reference value;
- `native-first-execute-expression` для native route;
- порівняння обох із independent expected value;
- вимога exact route provenance `native/completed`;
- named fail kind для evaluator mismatch та native/route mismatch.

Після цього focused witness має дати `(native-first-parity-witness (status pass) (cases 4))`.

## Завдання 3 — integration

Diff hygiene, exact-head CI, bilingual gate, fresh PR до `main`. Старий stacked #514 не merge-ити. Після GREEN передати #509/#504 правило: кожен новий native island додає value/error/effect/provenance evidence.

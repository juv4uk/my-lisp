# Дизайн Native-First виконання

**Статус:** затверджено запитом власника 2026-09-18  
**Батьківська задача:** #504  
**Перший implementation slice:** #505

## Мета

Зробити policy виконання явною для кожного Lisp-виразу:

```text
вираз
  |
  v
Lisp-owned native-plan classifier
  |----------------------|
 native-plan       evaluator-fallback
  |                      |
 admitted forms          reference evaluator
  |
 bytes
  |
 CPU
```

Policy є тотальною навіть тоді, коли native coverage часткове. Відсутність native lowering — нормальний routing outcome, а не помилка мови.

## Авторитети

- Сенс мови: semantic registry / Canon.
- Класифікація native-plan: Lisp-owned machine projection.
- ISA та encoding: чинний `lib/machine/**`.
- Оптимізація: CML.
- Executable-memory mechanism: host.
- Evaluator: reference/fallback шлях.

Rust semantic matcher не додається.

## Перший slice (#505)

Новий `lib/machine/dispatch/native-first.lisp` дає:

```lisp
(native-first-plan expression-data)
```

Результат:

```text
(native-plan <structured-machine-forms> <arena-bytes>)
(evaluator-fallback <original-expression>)
```

Перший native island — вже доведена bounded literal форма:

```lisp
(car (cons <u64-literal> <u64-literal>))
```

Вона перевикористовує `x86-lower-cons-car-u64-forms` і `x86-pair-cell-bytes`. Classifier не генерує raw bytes і не викликає `native-call-u64-raw`.

Усе інше повертається evaluator-у без змін, включно з dynamic operands.

## Fail-Closed правила

- Лише точний shape match.
- Обидва поля CONS мусять бути admitted exact u64.
- Malformed/dotted source data -> fallback.
- Unsupported semantics -> fallback.
- Після того як майбутній execution bridge прийняв `native-plan`, machine error не можна тихо маскувати evaluator fallback.

## Розвиток

Native coverage росте додаванням окремо доведених classifier routes. Source program не обирає backend і не змінюється при появі нового lowering.

## Перевірка

#505 доводить лише classification. #506 відповідає за виконання route. #509 — differential parity для кожного native island.

#483/#484 можуть паралельно розширювати machine atoms; #505 їхні файли не редагує.

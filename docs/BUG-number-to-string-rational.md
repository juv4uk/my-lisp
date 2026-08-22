# BUG: number->string crashes on non-integer rationals

**Статус:** CONFIRMED (мінімальне репро нижче) · **Знайдено:** 2026-08-22,
vyasa, під час WSM-24 shape comparison · **Серйозність:** середня

## Репро

```lisp
;; file mode:
(print (number->string 1/3))
;; => Error: cdr expects a non-empty list ... (nth 1/3 <digit-table>)
```

`(print 1/3)` працює; `(print (number->string 1))` працює; будь-який
НЕцілий раціональний у number->string — краш.

## Корінь

`lib/core.my::number->string` обробляє лише цілі: `digit->string`
робить `(nth d <таблиця цифр>)`, а для d = 1/3 дробовий індекс
спускається по cdr до порожнього списку (той самий latent-патерн, що
й у subsample з дробовим кроком). Помилка поверхнею вводить в оману —
виглядає як пошкодження памʼяті, хоча це звичайний unsupported-input.

## Обхід (застосовано в WSM-24/mylisp/mylisp-lib.my)

`rat->string` — довге ділення: ціла частина через quotient,
дробові розряди циклом rem*10 → digit → новий залишок.

## Правильний фікс (пропозиція)

Або number->string підтримує раціональні (параметр кількості
розрядів + довге ділення в бібліотекy), або чіткий error
"number->string expects an integer" замість cdr-краху.
Довгостроково: numerator/denominator примітиви відкриють точний
раціональний друк скрізь.

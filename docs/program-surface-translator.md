# Перекладач програмних поверхонь · Program surface translator

`scripts/translate-program.py` перекладає вихідний код my-lisp між трьома
поверхнями: англійською (`en`), українською (`uk`) і санскритською (`sa`).
Підтримуються всі шість напрямків.

```bash
python3 scripts/translate-program.py --from en --to uk program.wsm
python3 scripts/translate-program.py --from uk --to sa program.wsm -o program.sa.wsm
python3 scripts/translate-program.py --from sa --to en -
```

Скрипт не має власного прихованого словника. Він читає машинну таблицю
`lib/surface/uk-sa-coverage.wsm`, тому перекладає лише відомі публічні назви.
Користувацькі символи, числа, відступи й дужки зберігаються. Коментарі та
текстові рядки не перекладаються, бо це дані програми.

```lisp
; вхід / input
(car (cons 'кіт 'пес))

; en → uk
(перше (сполучити 'кіт 'пес))

; uk → sa
(ādi (saṃyuj 'кіт 'пес))
```

Цитовані зареєстровані назви також перекладаються. Для Lisp це важливо:
цитована форма може бути кодом, який пізніше виконає `eval`. Текст, який не є
кодом, варто подавати рядком — наприклад, `"car"` лишиться `"car"`.

The translator rewrites registered program symbols and preserves formatting,
comments, string data, numbers, and unknown user identifiers. Its vocabulary
comes only from the machine-readable coverage table. Tests cover all six
directions, round trips, preservation boundaries, and equal execution results
for translated English, Ukrainian, and Sanskrit programs.

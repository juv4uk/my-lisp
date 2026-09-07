# Десяткова кома в reader-і

Статус: **ратифіковано, language-contract 5.0**.

Українська клавіатурна поверхня дозволяє писати десятковий роздільник як комою, так і крапкою. Обидва написання позначають **те саме точне математичне значення**:

```lisp
(eq 12,455 12.455)   ; t
(eq -0,25 -0.25)     ; t
(eq 1,5e3 1500)      ; t
(+ 1,5 2,5)          ; 4
```

## Межа

Кома не стає загальною пунктуацією Lisp. Вона отримує роль десяткового роздільника лише тоді, коли **весь токен** після заміни коми на крапку є коректним скінченним десятковим або base-10 scientific числом.

Тому ці токени лишаються символами:

```text
а,б
версія1,2
1,2,3
1,2.3
```

Reader не змінює вихідний текст довільно й не розділяє такі символи на частини.

## Семантика

```text
12,455 ─┐
        ├──► одне exact Rational value
12.455 ─┘
```

Кома є лише альтернативним **surface spelling** десяткового роздільника. Вона не створює нового числового типу, нового примітива чи окремої арифметичної семантики.

Правило успадковує S1: скінченні десяткові та base-10 scientific literals читаються точно, без проміжного `f64`-наближення.

## Чому contract 5.0

До цієї зміни `12,455` було коректним символом. Після зміни те саме джерельне представлення читається як число. Це observable reader semantics, тому за власними правилами [`language-contract.my`](../language-contract.my) зміна є major: **4.0 → 5.0**.

Семантика апострофа з contract 4.0 не змінюється:

```lisp
'кіт       ; QUOTE reader sugar
об'єкт     ; один символ
```

## Допоміжне резюме · English

Contract 5.0 accepts `.` and `,` as equivalent decimal separators only when the whole token is otherwise a valid decimal/base-10 scientific number. Non-numeric comma-containing tokens remain symbols. Both numeric spellings produce the same exact value.

## Ergänzende Zusammenfassung · Deutsch

Contract 5.0 akzeptiert `.` und `,` als gleichwertige Dezimaltrennzeichen nur dann, wenn das gesamte Token ansonsten eine gültige Dezimal- bzw. wissenschaftliche Zahl zur Basis 10 ist. Nichtnumerische Tokens mit Komma bleiben Symbole. Beide Zahlenschreibweisen ergeben denselben exakten Wert.

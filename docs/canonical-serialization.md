# Canonical serialization · Канонічна серіалізація · Kanonische Serialisierung

## English

`write-to-string` defines my-lisp's implementation-independent data wire format. For every value in the serializable domain, `read(write-to-string(value))` is structurally `equal?` to `value`, and equal values produce byte-for-byte equal text. The executable authority is the Tier-2 “Canonical serialization law” block in [`tests/fixtures/conformance.my`](../tests/fixtures/conformance.my), not Rust's `Display` implementation.

The domain is `()`/`t`, readable symbols, strings, pairs (proper and dotted lists), exact integers and reduced rationals, and finite inexact numbers. Closures, macros, TCP handles, and other live capabilities are deliberately outside it: diagnostic forms such as `<lambda>` are not data and must not be persisted or exchanged.

- `()` and `t` represent false/nil and true; symbols use their readable token.
- Strings are quoted. Quote, backslash, newline, and tab are escaped as `\"`, `\\`, `\n`, and `\t`; other Unicode scalar values remain literal.
- Proper lists use parentheses and one ASCII space between items: `(a b c)`. An improper tail uses one space on each side of the dot: `(a b . c)`.
- Exact integers use base-10 digits with no suffix. Exact rationals are reduced with a positive denominator and use `numerator/denominator`; an integral rational prints as an integer.
- Finite inexact numbers use the shortest reader-compatible decimal form that preserves the represented value. A whole inexact value retains one fractional digit (`3.0`), so exactness is never erased.

This text—not a Rust layout or digest algorithm—is semantic identity. `knowledge-content-address` and `world-content-address` use it directly. Software or FPGA adapters may hash its UTF-8 bytes for transport, but canonical text must match before hashing.

## Українська

`write-to-string` визначає незалежний від реалізації data wire format my-lisp. Для кожного значення із серіалізованого домену `read(write-to-string(value))` структурно `equal?` до `value`, а рівні значення дають побайтово однаковий текст. Виконуване джерело істини — Tier-2 блок “Canonical serialization law” у [`tests/fixtures/conformance.my`](../tests/fixtures/conformance.my), а не Rust `Display`.

Домен: `()`/`t`, читабельні символи, рядки, пари (proper і dotted lists), точні цілі та скорочені раціональні числа, скінченні неточні числа. Замикання, макроси, TCP handles та інші живі capabilities навмисно поза ним: діагностичні форми на кшталт `<lambda>` не є даними й не мають зберігатися чи передаватися.

- `()` і `t` представляють false/nil та true; символ використовує свій читабельний token.
- Рядок береться в лапки. Quote, backslash, newline і tab екрануються як `\"`, `\\`, `\n` і `\t`; інші Unicode scalar values лишаються буквальними.
- Proper list має дужки й один ASCII-пробіл між елементами: `(a b c)`. Неправильний хвіст має по одному пробілу навколо крапки: `(a b . c)`.
- Точне ціле використовує десяткові цифри без suffix. Точний rational скорочений, має додатний знаменник і форму `numerator/denominator`; цілий rational друкується як ціле.
- Скінченне неточне число використовує найкоротшу сумісну з reader десяткову форму, що зберігає представлене значення. Ціле неточне значення зберігає одну дробову цифру (`3.0`), тому exactness не стирається.

Саме цей текст — не Rust layout і не digest-алгоритм — є semantic identity. `knowledge-content-address` та `world-content-address` використовують його напряму. Software- чи FPGA-adapter може хешувати UTF-8 bytes для передачі, але спочатку має збігтися канонічний текст.

## Deutsch

`write-to-string` definiert my-lisps implementierungsunabhängiges Datenformat. Für jeden Wert im serialisierbaren Bereich ist `read(write-to-string(value))` strukturell `equal?` zu `value`, und gleiche Werte erzeugen bytegleich denselben Text. Die ausführbare Autorität ist der Tier-2-Block „Canonical serialization law“ in [`tests/fixtures/conformance.my`](../tests/fixtures/conformance.my), nicht Rusts `Display`-Implementierung.

Der Bereich umfasst `()`/`t`, lesbare Symbole, Strings, Paare (echte und Dotted Lists), exakte Ganzzahlen und gekürzte rationale Zahlen sowie endliche inexakte Zahlen. Closures, Makros, TCP-Handles und andere lebende Capabilities liegen bewusst außerhalb: Diagnoseformen wie `<lambda>` sind keine Daten und dürfen weder gespeichert noch ausgetauscht werden.

- `()` und `t` stehen für falsch/NIL und wahr; Symbole verwenden ihr lesbares Token.
- Strings stehen in Anführungszeichen. Quote, Backslash, Zeilenumbruch und Tabulator werden als `\"`, `\\`, `\n` und `\t` escaped; andere Unicode-Skalarwerte bleiben wörtlich.
- Echte Listen verwenden Klammern und genau ein ASCII-Leerzeichen zwischen Elementen: `(a b c)`. Ein unechter Schwanz verwendet je ein Leerzeichen um den Punkt: `(a b . c)`.
- Exakte Ganzzahlen verwenden Dezimalziffern ohne Suffix. Exakte rationale Zahlen sind gekürzt, haben einen positiven Nenner und erscheinen als `Zähler/Nenner`; ein ganzzahliges Rational erscheint als Ganzzahl.
- Endliche inexakte Zahlen verwenden die kürzeste reader-kompatible Dezimalform, welche den dargestellten Wert erhält. Ein ganzzahliger inexakter Wert behält eine Nachkommastelle (`3.0`), sodass Exaktheit nie verloren geht.

Dieser Text—nicht ein Rust-Layout oder Digest-Algorithmus—ist semantische Identität. `knowledge-content-address` und `world-content-address` verwenden ihn direkt. Software- oder FPGA-Adapter dürfen seine UTF-8-Bytes für den Transport hashen; zuerst muss jedoch der kanonische Text übereinstimmen.

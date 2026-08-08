# my-lisp language core · Ядро мови my-lisp · my-lisp-Sprachkern

> **A small language that grows itself. · Маленька мова, що вирощує себе. · Eine kleine Sprache, die sich selbst wachsen lässt.**

The language is named **my-lisp** and is independent from the IDE. Its canonical implementation lives in `crates/my-lisp` (Rust), which powers both the desktop shell and the Web build via WebAssembly. The initial ClojureScript prototype has been fully replaced.

Мова має назву **my-lisp** і є незалежною від IDE. Її канонічна реалізація міститься у `crates/my-lisp` (Rust), яка забезпечує роботу як десктопної оболонки, так і веб-збірки через WebAssembly. Початковий прототип на ClojureScript повністю замінено.

Die Sprache heißt **my-lisp** und ist von der IDE unabhängig. Ihre kanonische Implementierung liegt in `crates/my-lisp` (Rust), welche sowohl die Desktop-Hülle als auch den Web-Build über WebAssembly antreibt. Der anfängliche ClojureScript-Prototyp wurde vollständig ersetzt.

The canonical source-file extension is **`.my`** (for example, `welcome.my`). The generic `.lisp` extension remains a compatible alias.

Канонічне розширення файлів початкового коду — **`.my`** (наприклад, `welcome.my`). Загальне розширення `.lisp` залишається сумісним псевдонімом.

Die kanonische Dateiendung für Quellcode ist **`.my`** (zum Beispiel `welcome.my`). Die allgemeine Endung `.lisp` bleibt ein kompatibler Alias.

The current product direction is a DrRacket-like language environment: a definitions editor, program execution, an interactions/REPL area, readable diagnostics, and tools for exploring parsed forms. It serves our own language rather than implementing Racket itself. The UI and the language engine remain separate components.

Поточний напрям продукту — середовище мови на кшталт DrRacket: редактор визначень, запуск програми, область взаємодії/REPL, зрозумілі помилки та інструменти для дослідження розібраних форм. Воно обслуговує нашу власну мову, а не реалізує Racket. Інтерфейс і рушій мови залишаються окремими компонентами.

Die aktuelle Produktrichtung ist eine DrRacket-ähnliche Sprachumgebung: Definitionseditor, Programmausführung, Interaktions-/REPL-Bereich, verständliche Diagnosen und Werkzeuge zur Untersuchung geparster Formen. Sie dient unserer eigenen Sprache und implementiert nicht Racket selbst. Oberfläche und Sprachengine bleiben getrennte Komponenten.

## McCarthy foundation · Основа Маккарті · McCarthy-Grundlage

The first contract contains the seven elementary Lisp operations described by John McCarthy:

- `quote` returns data without evaluating it;
- `atom` recognizes atoms, including the empty list;
- `eq` compares two atoms;
- `car` returns the first element of a non-empty list;
- `cdr` returns the rest of a non-empty list;
- `cons` prepends an element to a list;
- `cond` evaluates clauses in order and selects the first true one.

`t` is the canonical true value. Invalid list operations produce explicit errors. Automated semantic tests are the compatibility contract for the future Rust core.

Перший контракт містить сім елементарних операцій Lisp: `quote`, `atom`, `eq`, `car`, `cdr`, `cons` і `cond`. Значення `t` є канонічною істиною. Некоректні операції зі списками повертають явні структуровані помилки, а автоматичні семантичні тести є контрактом сумісності.

Der erste Vertrag umfasst die sieben elementaren Lisp-Operationen `quote`, `atom`, `eq`, `car`, `cdr`, `cons` und `cond`. `t` ist der kanonische Wahrheitswert. Ungültige Listenoperationen liefern explizite strukturierte Fehler; automatisierte Semantiktests bilden den Kompatibilitätsvertrag.

The crate contains its own UTF-8 parser, value model, lexical environment frames, source spans, and structured errors. It has no Tauri dependency and no direct access to files, the network, or UI capabilities.

Exact rational arithmetic is a kernel mechanism: `/` accepts exact integers and rational values, reduces every fraction, and prints results such as `5/336` without floating-point rounding.

Крейт має власний UTF-8-парсер, модель значень, фрейми лексичного середовища, діапазони початкового коду та структуровані помилки. Він не залежить від Tauri й не має прямого доступу до файлів, мережі чи можливостей інтерфейсу.

Точна раціональна арифметика є механізмом ядра: `/` приймає точні цілі та раціональні значення, скорочує кожен дріб і виводить результати на кшталт `5/336` без floating-point округлення.

Das Crate besitzt einen eigenen UTF-8-Parser, ein Wertmodell, lexikalische Umgebungsframes, Quellbereiche und strukturierte Fehler. Es hängt nicht von Tauri ab und hat keinen direkten Zugriff auf Dateien, Netzwerk oder UI-Funktionen.

Exakte rationale Arithmetik ist ein Kernmechanismus: `/` akzeptiert exakte Ganzzahlen und rationale Werte, kürzt jeden Bruch und gibt Ergebnisse wie `5/336` ohne Gleitkommarundung aus.

## Bootstrap boundary · Межа саморозгортання · Bootstrap-Grenze

Rust provides only the mechanisms it implements particularly well: memory-safe runtime values, UTF-8 reading, lexical closures, deterministic evaluation, stack control, structured diagnostics, and an explicit capability boundary. Higher-level language features and the standard library should be written in the small Lisp itself whenever the existing core can express them.

Rust надає лише ті механізми, які він виконує особливо добре: безпечні щодо пам’яті значення, читання UTF-8, лексичні замикання, детерміноване обчислення, контроль стека, структуровану діагностику та явну межу системних можливостей. Високорівневі можливості й стандартну бібліотеку слід писати самою маленькою Lisp-мовою щоразу, коли наявне ядро вже може їх виразити.

Rust stellt nur die Mechanismen bereit, die es besonders gut umsetzt: speichersichere Laufzeitwerte, UTF-8-Lesen, lexikalische Closures, deterministische Auswertung, Stack-Kontrolle, strukturierte Diagnosen und eine explizite Capability-Grenze. Höhere Sprachfunktionen und die Standardbibliothek sollen in der kleinen Lisp-Sprache selbst geschrieben werden, sobald der vorhandene Kern sie ausdrücken kann.

`lambda` belongs to the Rust semantic kernel because it makes user-defined functions and self-hosted libraries possible. Derived forms such as `defn`, list helpers, logical combinators, and teaching examples belong in a bootstrapped Lisp library rather than as Rust built-ins.

The first bootstrapped library is `lib/core.my`. Its definitions (`identity`, `not`, `pair`, `second`, `third`, `caar`, and `cadr`) are ordinary my-lisp code. Rust supplies only `def` and lexical binding as the mechanism needed to load persistent named definitions.

`lambda` належить до семантичного Rust-ядра, бо робить можливими користувацькі функції та саморозгорнуті бібліотеки. Похідні форми на кшталт `defn`, допоміжні функції списків, логічні комбінатори й навчальні приклади мають жити у bootstrap-бібліотеці Lisp, а не бути вбудованими у Rust.

Перша bootstrap-бібліотека — `lib/core.my`. Її визначення (`identity`, `not`, `pair`, `second`, `third`, `caar` і `cadr`) є звичайним кодом my-lisp. Rust надає лише `def` і лексичне зв’язування як механізм завантаження постійних іменованих визначень.

`lambda` gehört zum semantischen Rust-Kern, weil es benutzerdefinierte Funktionen und selbst gehostete Bibliotheken ermöglicht. Abgeleitete Formen wie `defn`, Listenhilfen, logische Kombinatoren und Lernbeispiele gehören in eine gebootstrappte Lisp-Bibliothek statt in Rust-Built-ins.

Die erste Bootstrap-Bibliothek ist `lib/core.my`. Ihre Definitionen (`identity`, `not`, `pair`, `second`, `third`, `caar` und `cadr`) sind gewöhnlicher my-lisp-Code. Rust stellt nur `def` und lexikalische Bindung als Mechanismus zum Laden dauerhafter benannter Definitionen bereit.

## Available Operations and Forms · Доступні операції та форми · Verfügbare Operationen und Formen

### Built-in Forms (Rust core) · Вбудовані форми · Eingebaute Formen
- `quote` (`'`) — Returns the expression unevaluated · Повертає вираз без обчислення · Gibt den Ausdruck unberechnet zurück
- `lambda` — Creates an anonymous function (closure) · Створює анонімну функцію (замикання) · Erstellt eine anonyme Funktion (Closure)
- `def` — Defines a variable or function in the current environment · Визначає змінну або функцію в поточному середовищі · Definiert eine Variable oder Funktion in der aktuellen Umgebung
- `defmacro` — Defines a macro (compile-time expansion) · Створює макрос (розкривається на етапі оцінки) · Definiert ein Makro (Erweiterung zur Kompilierzeit)
- `list` — Creates a list from evaluated arguments · Створює список з обчислених аргументів · Erstellt eine Liste aus ausgewerteten Argumenten
- `cond` — Conditional logic · Умовна логіка · Bedingte Logik
- `atom` — Checks if a value is an atom (not a list/pair) · Перевіряє, чи є значення атомом (не списком/парою) · Prüft, ob ein Wert ein Atom ist (keine Liste/Paar)
- `eq` — Checks equality between two atoms · Перевіряє рівність двох атомів · Prüft die Gleichheit zwischen zwei Atomen
- `car` — Returns the first element of a list/pair · Повертає перший елемент списку/пари · Gibt das erste Element einer Liste/eines Paares zurück
- `cdr` — Returns the tail of a list/pair · Повертає хвіст списку/пари · Gibt den Rest einer Liste/eines Paares zurück
- `cons` — Constructs a pair or adds an element to a list · Створює пару або додає елемент до списку · Erstellt ein Paar oder fügt ein Element zu einer Liste hinzu
- `+`, `-`, `*`, `/` — Arithmetic operations. `/` produces exact rational fractions (e.g. `1/3`) for integers. · Арифметичні операції. `/` для цілих чисел створює точні раціональні дроби (напр. `1/3`). · Arithmetische Operationen. `/` erzeugt für Ganzzahlen exakte rationale Brüche (z. B. `1/3`).

### Standard Library (`lib/core.my`) · Стандартна бібліотека · Standardbibliothek
- `identity` — Returns its argument · Повертає переданий аргумент · Gibt sein Argument zurück
- `not` — Logical NOT · Логічне заперечення · Logisches NICHT
- `pair` — Creates a list of two elements · Створює список із двох елементів · Erstellt eine Liste aus zwei Elementen
- `second` — Returns the second element of a list · Повертає другий елемент списку · Gibt das zweite Element einer Liste zurück
- `third` — Returns the third element of a list · Повертає третій елемент списку · Gibt das dritte Element einer Liste zurück
- `caar` — `(car (car x))`
- `cadr` — `(car (cdr x))`
- `length` — Counts the elements of a list · Рахує елементи списку · Zählt die Elemente einer Liste
- `reverse` — Reverses a list (tail-recursive via a `reverse-onto` accumulator) · Розвертає список (хвостово-рекурсивно, через акумулятор `reverse-onto`) · Kehrt eine Liste um (endrekursiv über einen `reverse-onto`-Akkumulator)
- `append` — Concatenates two lists · З'єднує два списки · Verkettet zwei Listen
- `map` — Applies a function to every element, returning a new list · Застосовує функцію до кожного елемента, повертає новий список · Wendet eine Funktion auf jedes Element an, gibt eine neue Liste zurück
- `filter` — Keeps elements for which a predicate is truthy · Лишає елементи, для яких предикат істинний · Behält Elemente, für die ein Prädikat wahr ist
- `reduce` — Left-folds a list with a function and a starting accumulator · Ліво-згортає список функцією та початковим акумулятором · Faltet eine Liste linksseitig mit einer Funktion und einem Start-Akkumulator

`length`, `map`, `filter`, and `reduce` are not tail-recursive (`reverse` is, via its accumulator); each builds its result after the recursive call returns, so very deep lists still grow the Rust call stack for these specific operations even though `cons`/`cdr`-style tail recursion does not.

`length`, `map`, `filter` та `reduce` не хвостово-рекурсивні (`reverse` — хвостово-рекурсивна, через акумулятор); кожна будує результат після повернення з рекурсивного виклику, тож дуже глибокі списки все ще ростять Rust call stack саме для цих операцій, хоча хвостова рекурсія у стилі `cons`/`cdr` — ні.

`length`, `map`, `filter` und `reduce` sind nicht endrekursiv (`reverse` ist es, über seinen Akkumulator); jede baut ihr Ergebnis erst nach der Rückkehr des rekursiven Aufrufs auf, sodass sehr tiefe Listen bei genau diesen Operationen weiterhin den Rust-Call-Stack wachsen lassen, auch wenn endrekursion im `cons`/`cdr`-Stil das nicht tut.

### Literal syntax · Синтаксис літералів · Literalsyntax

- Symbols: any run of non-whitespace, non-`()`/`;` characters that isn't a number or a `n/d` rational — UTF-8 throughout, so Cyrillic identifiers (`радіо`, `довжина`) are first-class. · Символи: будь-яка послідовність непробільних символів поза `()`/`;`, що не є числом чи `n/d`-раціональним — UTF-8 наскрізь, тож кириличні ідентифікатори (`радіо`, `довжина`) рівноправні. · Symbole: jede Folge von Nicht-Leerraum-Zeichen außerhalb von `()`/`;`, die keine Zahl oder ein `n/d`-Rational ist — durchgehend UTF-8, kyrillische Bezeichner sind gleichberechtigt.
- Numbers: `42`, `-4.5` parse as inexact `f64` (`Value::Number`). A bare `n/d` token (e.g. `5/6`) parses directly as an exact `Rational`, the same value `/` would produce — no need to write `(/ 5 6)` for a literal fraction. · Числа: `42`, `-4.5` парсяться як неточний `f64`. Голий токен `n/d` (напр. `5/6`) парситься напряму як точний `Rational` — не потрібно писати `(/ 5 6)` для дробового літералу. · Zahlen: `42`, `-4.5` werden als inexaktes `f64` geparst. Ein nackter `n/d`-Token (z. B. `5/6`) wird direkt als exaktes `Rational` geparst.
- Strings: `"..."` with `\n`, `\t`, `\"`, `\\` escapes; any other escaped character passes through literally. · Рядки: `"..."` з escape-послідовностями `\n`, `\t`, `\"`, `\\`; будь-який інший escape-символ проходить буквально. · Zeichenketten: `"..."` mit den Escapes `\n`, `\t`, `\"`, `\\`; jedes andere escapete Zeichen wird unverändert übernommen.
- Lists: `(a b c)`, built from `cons` cells ending in `()`/`Nil`. An improper (dotted) tail prints as `(a . b)`. · Списки: `(a b c)`, побудовані з `cons`-комірок, що закінчуються `()`/`Nil`. Неправильний (крапковий) хвіст друкується як `(a . b)`. · Listen: `(a b c)`, aus `cons`-Zellen aufgebaut, die mit `()`/`Nil` enden. Ein unechter (gepunkteter) Tail wird als `(a . b)` ausgegeben.
- `'expr` is reader sugar for `(quote expr)`, desugared before the evaluator ever sees it. There is no quasiquote/unquote (no `` ` ``/`,`) — macro templates are built by hand from `list`/`cons`, as `lib/core.my`'s bootstrap style and the `unless` macro example in `docs/quote-tutorial.md` show. · `'вираз` — синтаксичний цукор для `(quote вираз)`, розкривається до того, як обчислювач його побачить. Немає quasiquote/unquote (немає `` ` ``/`,`) — шаблони макросів будуються вручну з `list`/`cons`. · `'ausdruck` ist Reader-Zucker für `(quote ausdruck)`. Es gibt kein Quasiquote/Unquote — Makro-Vorlagen werden manuell aus `list`/`cons` gebaut.
- `; comment` runs to end of line. · `; коментар` триває до кінця рядка. · `; Kommentar` läuft bis zum Zeilenende.

### Truthiness, `t`, and `cond` · Істинність, `t` і `cond` · Wahrheitswert, `t` und `cond`

Every value is truthy except the empty list `'()`/`Nil` and the boolean `false` produced by primitives like `atom`/`eq`; both print as `()`. `t` is the canonical true symbol, printing as `t`. Each `cond` clause is exactly `(test expression)` — one result expression, not an implicit sequence — and `cond` with no matching clause (or an empty clause list) evaluates to `()` rather than raising an error.

Кожне значення істинне, крім порожнього списку `'()`/`Nil` і булевого `false`, який повертають примітиви на кшталт `atom`/`eq`; обидва друкуються як `()`. `t` — канонічний символ істини, друкується як `t`. Кожна гілка `cond` — рівно `(тест вираз)`, один результатний вираз, не неявна послідовність — і `cond` без відповідної гілки (чи з порожнім списком гілок) обчислюється в `()`, а не кидає помилку.

Jeder Wert ist wahr außer der leeren Liste `'()`/`Nil` und dem booleschen `false`, das Primitive wie `atom`/`eq` liefern; beide werden als `()` ausgegeben. `t` ist das kanonische Wahr-Symbol. Jede `cond`-Klausel ist genau `(Test Ausdruck)` — ein Ergebnisausdruck, keine implizite Sequenz — und ein `cond` ohne passende Klausel wertet zu `()` aus statt einen Fehler auszulösen.

### Functions and macros · Функції та макроси · Funktionen und Makros

`lambda` takes a fixed parameter list (no variadic/rest args, no optional parameters, duplicate parameter names rejected at creation) and one or more body expressions evaluated in sequence, returning the last. Calling with the wrong argument count is an `ErrorKind::Arity` error, not silent truncation or padding. `def` binds a name in the current lexical frame *before* evaluating dependent closures are created, so a `lambda` can call itself by name recursively. `defmacro` works the same as `lambda` but the bound value is a `Value::Macro`: called at evaluation time, its body runs first to produce a new expression (typically built with `list`/`cons`/`quote`), which is then evaluated in place of the macro call — a runtime expansion step, not a separate compile pass.

`lambda` приймає фіксований список параметрів (без variadic/rest-аргументів, без опціональних параметрів, повторювані імена параметрів відхиляються при створенні) та одне чи більше тіл-виразів, що обчислюються послідовно, повертаючи останній. Виклик з неправильною кількістю аргументів — помилка `ErrorKind::Arity`, не тиха обрізка чи доповнення. `def` зв'язує ім'я в поточному лексичному фреймі *до* того, як залежні замикання створюються, тож `lambda` може викликати сама себе рекурсивно за іменем. `defmacro` працює як `lambda`, але зв'язане значення — `Value::Macro`: викликається під час обчислення, тіло спершу виконується, щоб побудувати новий вираз (зазвичай через `list`/`cons`/`quote`), який тоді обчислюється замість виклику макроса — крок розгортання під час виконання, не окремий compile-прохід.

`lambda` nimmt eine feste Parameterliste (keine variadischen/Rest-Argumente, keine optionalen Parameter, doppelte Parameternamen werden bei der Erstellung abgelehnt) und einen oder mehrere Rumpfausdrücke, sequenziell ausgewertet, der letzte wird zurückgegeben. Ein Aufruf mit falscher Argumentanzahl ist ein `ErrorKind::Arity`-Fehler, kein stilles Abschneiden oder Auffüllen. `def` bindet einen Namen im aktuellen lexikalischen Frame *bevor* abhängige Closures erstellt werden, sodass ein `lambda` sich selbst rekursiv beim Namen aufrufen kann. `defmacro` funktioniert wie `lambda`, aber der gebundene Wert ist ein `Value::Macro`: zur Auswertungszeit aufgerufen, läuft sein Rumpf zuerst, um einen neuen Ausdruck zu erzeugen (üblicherweise mit `list`/`cons`/`quote`), der dann anstelle des Makroaufrufs ausgewertet wird — ein Laufzeit-Expansionsschritt, kein separater Compile-Durchgang.

### Known gaps · Відомі прогалини · Bekannte Lücken

There are currently no comparison operators (`<`, `>`, `=`, `<=`, `>=`) and no I/O primitives (no `print`/`display`/`read`) at any layer — the CLI only ever shows the *last* top-level form's value, everything before it runs purely for `def`/`defmacro` side effects. `Session`/`EvalResult` already carry an `output` field reserved for a future print-like builtin, but nothing populates it yet. These are open bootstrap-boundary work, not accidents: any of them, once added, belongs in `lib/core.my` if expressible there, or as a minimal Rust primitive only if it genuinely needs host capability the language can't provide itself.

Наразі немає операторів порівняння (`<`, `>`, `=`, `<=`, `>=`) і немає I/O-примітивів (немає `print`/`display`/`read`) на жодному рівні — CLI показує лише значення *останньої* верхньорівневої форми, все перед нею виконується лише заради побічних ефектів `def`/`defmacro`. `Session`/`EvalResult` вже мають поле `output`, зарезервоване під майбутній print-подібний built-in, але поки нічого його не заповнює. Це відкрита робота на межі bootstrap, не випадковість: будь-що з цього, коли з'явиться, має жити в `lib/core.my`, якщо там виразне, або мінімальним Rust-примітивом лише якщо справді потребує host-можливості, яку мова сама надати не може.

Es gibt derzeit keine Vergleichsoperatoren (`<`, `>`, `=`, `<=`, `>=`) und keine I/O-Primitive (kein `print`/`display`/`read`) auf irgendeiner Ebene — die CLI zeigt nur den Wert der *letzten* Top-Level-Form, alles davor läuft rein wegen `def`/`defmacro`-Nebeneffekten. `Session`/`EvalResult` tragen bereits ein `output`-Feld für einen künftigen print-artigen Built-in, aber nichts füllt es bisher. Das ist offene Bootstrap-Grenz-Arbeit, kein Versehen: alles davon gehört, sobald hinzugefügt, nach `lib/core.my`, sofern dort ausdrückbar, oder nur als minimales Rust-Primitiv, wenn es echte Host-Fähigkeit braucht, die die Sprache selbst nicht bieten kann.

### Tooling around the language · Інструментарій навколо мови · Werkzeuge rund um die Sprache

- **Errors**: every `LanguageError` carries an `ErrorKind` (`Parse`, `UnknownSymbol`, `Arity`, `Type`, `InvalidForm`) and a byte-offset `Span`. `LanguageError::render` turns that into a line:column location, the offending source line, and a `^^^` caret underline (char-counted, so multi-byte UTF-8 source stays aligned) — what `my-lisp-cli` prints on parse/eval failure. · **Помилки**: кожна `LanguageError` несе `ErrorKind` і byte-offset `Span`. `LanguageError::render` перетворює це на рядок:стовпець, вихідний рядок і підкреслення `^^^`. · **Fehler**: jede `LanguageError` trägt einen `ErrorKind` und einen `Span`. `LanguageError::render` macht daraus eine Zeile:Spalte-Position, die Quellzeile und eine `^^^`-Unterstreichung.
- **Literate Markdown** (`crates/my-lisp-literate`): a `.md` source can mix prose with ` ```my-lisp ` fenced code blocks; only the fenced blocks execute, concatenated, with error spans remapped back to their original position in the Markdown file — see `docs/quote-tutorial.md` for the format in practice. · **Literate Markdown**: `.md`-джерело може змішувати прозу з блоками ` ```my-lisp `; виконуються лише огороджені блоки, зі зміщеннями помилок, ремапленими назад у Markdown-файл. · **Literate Markdown**: eine `.md`-Quelle kann Prosa mit ` ```my-lisp `-Codeblöcken mischen; nur diese Blöcke werden ausgeführt.
- **REPL** (`my-lisp-cli`, no file argument): history persists across sessions to `~/.my-lisp-history` (falls back to no persistence, not a crash, if neither `HOME` nor `USERPROFILE` resolves). · **REPL**: історія зберігається між сесіями в `~/.my-lisp-history` (без HOME/USERPROFILE — без персистенції, не крах). · **REPL**: der Verlauf bleibt sitzungsübergreifend in `~/.my-lisp-history` erhalten.
- **`tests/fixtures/conformance.json`** is the implementation-independent contract: plain `expr`/`expected` pairs (plus an optional `"mode": "markdown"` for literate fixtures), run against `lib/core.my` preloaded, meant to be reproducible by any future my-lisp implementation. · **`tests/fixtures/conformance.json`** — незалежний від реалізації контракт, відтворюваний будь-якою майбутньою реалізацією my-lisp. · **`tests/fixtures/conformance.json`** ist der implementierungsunabhängige Vertrag, reproduzierbar von jeder künftigen my-lisp-Implementierung.

## Unified execution · Уніфіковане виконання · Einheitliche Ausführung

The migration from the ClojureScript prototype to the Rust core is complete. The Rust implementation conforms to the primitive behavior specified by implementation-independent examples and tests.

Міграція з прототипу на ClojureScript до Rust-ядра завершена. Rust-реалізація відповідає примітивній поведінці, визначеній незалежними від реалізації прикладами та тестами.

Die Migration vom ClojureScript-Prototyp zum Rust-Kern ist abgeschlossen. Die Rust-Implementierung entspricht dem primitiven Verhalten, das durch implementierungsunabhängige Beispiele und Tests spezifiziert ist.

`crates/my-lisp-wasm` runs the canonical Rust engine directly in the browser via WebAssembly, powering the standalone web REPL (`public/my-lisp-cli-web.html`). Both it and `my-lisp-cli` use single-pass parsing (`eval_parsed_expressions`) — source is parsed once to produce both an AST view and the evaluation result, rather than parsing twice for each concern.

`crates/my-lisp-wasm` запускає канонічний Rust-рушій напряму в браузері через WebAssembly, живлячи автономний web-REPL (`public/my-lisp-cli-web.html`). І він, і `my-lisp-cli` використовують однопрохідний парсинг (`eval_parsed_expressions`) — код парситься один раз для AST-подання та результату обчислення, а не двічі під кожну потребу окремо.

`crates/my-lisp-wasm` führt die kanonische Rust-Engine direkt im Browser über WebAssembly aus und betreibt den eigenständigen Web-REPL (`public/my-lisp-cli-web.html`). Sowohl er als auch `my-lisp-cli` nutzen Single-Pass-Parsing (`eval_parsed_expressions`) — der Quellcode wird einmal geparst, um sowohl eine AST-Ansicht als auch das Auswertungsergebnis zu liefern, statt zweimal getrennt zu parsen.

The Rust evaluator executes the final expression of a closure and the selected `cond` branch through an explicit trampoline. Tail-recursive programs therefore use constant Rust call-stack space. Closure bodies share immutable AST nodes through `Rc`, and `Value::Pair` cons cells already share tail structure the same way — `cons` is O(1) regardless of list length (see `docs/benchmarks.md`); deep-list `Drop` is iterative and stack-safe (`crates/my-lisp/tests/stack_safety.rs`).

Rust evaluator виконує останній вираз closure та вибрану гілку `cond` через явний trampoline. Тому хвостово-рекурсивні програми використовують сталий обсяг Rust call stack. Тіла closure спільно використовують незмінні AST-вузли через `Rc`, і cons-комірки `Value::Pair` так само вже ділять хвостову структуру — `cons` є O(1) незалежно від довжини списку (див. `docs/benchmarks.md`); drop глибоких списків ітеративний і stack-safe (`crates/my-lisp/tests/stack_safety.rs`).

Der Rust-Evaluator führt den letzten Closure-Ausdruck und den gewählten `cond`-Zweig über ein explizites Trampolin aus. Tail-rekursive Programme benötigen dadurch konstanten Rust-Call-Stack. Closure-Rümpfe teilen unveränderliche AST-Knoten über `Rc`, und `Value::Pair`-Cons-Zellen teilen ebenso bereits Tail-Struktur — `cons` ist O(1) unabhängig von der Listenlänge (siehe `docs/benchmarks.md`); das Droppen tiefer Listen ist iterativ und stack-sicher (`crates/my-lisp/tests/stack_safety.rs`).

# my-lisp capabilities — how complete is the language today? · Можливості my-lisp — наскільки повна мова сьогодні? · my-lisp-Fähigkeiten — wie vollständig ist die Sprache heute?

## English

**What this document is.** `docs/language-core.md` deliberately scopes to Tier 1 (CORE SEMANTICS) and Tier 2 (LANGUAGE CONTRACT) — the seven McCarthy primitives, the bootstrap kernel, arithmetic, comparisons, I/O. This document is the full picture across all three tiers (`docs/conformance-tier-map.md`), including Tier 3 (ECOSYSTEM CONFORMANCE — the symbolic-reasoning layer), which `language-core.md` doesn't cover by design. Verified against the actual dispatcher (`crates/my-lisp/src/eval/mod.rs`) and every `lib/*.my` file's top-level definitions on 2026-08-09, not written from memory.

### Tier 1 — the seven primitives + bootstrap kernel

`quote`, `atom`, `eq`, `car`, `cdr`, `cons`, `cond` (McCarthy's original seven), plus `lambda`/`def`/`defmacro` (the minimal semantic kernel needed to bootstrap everything else). Full rationale and boundaries in `docs/language-core.md`/`docs/language-core-axioms.md`.

### Tier 2 — the language contract

- **Arithmetic**: `+`, `-`, `*`, `/`. Exact rationals (arbitrary-precision `BigInt`-backed, `crates/my-lisp/src/bignum.rs`) with a real numeric-exactness model (`Exactness::Exact`/`Inexact` as a property of the value, not its written form) — `(eq 3 3.0)` → `()`, `(= 3 3.0)` → `t`. One inexact operand contaminates a result to inexact.
- **Comparisons**: `<`, `>`, `=`, `<=`, `>=`, variadic and chained (`(< 1 2 3)`), same exact/inexact promotion rule as arithmetic.
- **I/O and reflection**: `print`/`princ` (session output transcript, `write`/`prin1` vs. `display` semantics), `read`/`read-file`/`read-all` (data reader, not evaluator), `eval` (closes the read-eval loop by hand), `load` (reads and evaluates a file).
- **Introspection**: `symbol?`, `string?`, `symbol->string`, `string->symbol`, `string-first`, `string-rest`.
- **Structured errors**: seven named `ErrorKind` categories (`Parse`, `UnknownSymbol`, `Arity`, `Type`, `InvalidForm`, `NumericOverflow`, `OutOfMemory`) — every failure is named and observable (S2), never silent. `NumericOverflow`/`OutOfMemory` are opt-in resource limits (`Environment::with_cons_limit`/`with_numeric_bit_limit`) — the default session, and every `conformance.my` fixture, stays unbounded.

### Standard library (`lib/core.my`)

`identity`, `not`, `pair`, `second`, `third`, `caar`, `cadr`, `equal?` (structural equality), `length`, `reverse`, `append`, `map`, `filter`, `reduce`, `let`, `let*`. `length`/`map`/`filter`/`reverse`/`append` are all tail-recursive (stack-safe on a 100,000-element list, `crates/my-lisp/tests/stack_safety.rs`). Variadic `lambda`/`defmacro` parameters (`(a b . rest)`, bare `args`) are how `list` itself moved from a Rust special form into `lib/core.my`.

### Tier 3 — the symbolic-reasoning layer (the actual point of the project, principle 3)

This is where most of the language's real capability lives, and where `docs/language-core.md` intentionally doesn't go.

- **`lib/unify.my`** — the unification primitive: logic variables (`(var name)`), `walk`/`walk-resolved` (dereferencing), structural (compound-term) unification, occurs-check (prevents infinite structures), `apply-subst` (reading out results), `thread-conjunction`/`thread-conjunction-branches` — the shared conjunction-walking kernel both engines below build on.
- **`lib/reason.my`** — a backward-chaining engine (micro-Prolog, the "Advice Taker" itself): `reason` takes a goal and a rule list, returns every valid substitution. Standardizing apart via depth-tagged variable renaming enables real recursive rules. Negation as failure (`not`). `explain-proof`/`reason-explain` render a human-readable trace and distinguish "proved" from "cannot prove." `count-usage`/`provenance` turn a proof tree into usage tallies and `(statement goal (source fact|rule) ...)` records — explicit provenance, not just an answer.
- **`lib/forward.my`** (1204 lines — the largest library file) — a CLIPS-style forward-chaining engine: `assert-fact!`/`retract-fact!` on a global working memory, `run`/`run-multi` firing a rule set to a fixpoint, `not`/`or`/`and`/`test`/`exists`/`forall` quantified/compound conditions, a single-justification truth-maintenance system and a full multi-justification JTMS (`run-jtms-multi!`) — retracting a fact correctly retracts everything that depended on it, transitively.
- **`lib/knowledge.my`** — modular knowledge packages (`defmodule`) on an append-only `*knowledge-journal*` (a flat `tell`/`retract` event log; a module's current clauses are a projection computed on demand, never a mutated snapshot). `reason-in`/`forward-in` query a named module through either engine above. `describe` collects every known fact about a symbol (atom-as-concept-entry-point). `record-usage!`/`usage-of` track rule firings across separate top-level queries.
- **`lib/understand.my`** / **`lib/narrate.my`** — a controlled-natural-language bridge, no LLM: fixed word-list shapes (`X is a Y`, `X V Y`, `all X have Y`) map to knowledge clauses and back, with `narrate-fact` as the exact structural inverse of `understand-is`/`understand-relation`.
- **`lib/clips-import.my`** — a real importer for CLIPS (NASA JSC, 1985) source files: `deffacts`/`defrule`/`deftemplate` → `defmodule`-ready clauses, `?x` variable syntax, salience/docstring preamble stripping, multislot conditions. Verified against nine real external `.clp` files including CLIPS's own Sudoku solver (`tests/fixtures/sudoku-external.clp`), not synthetic examples.
- **`lib/meta-eval.my`** — a metacircular evaluator (`my-eval`/`my-apply`) written in my-lisp itself: self-evaluation, `quote`, arithmetic/list-primitive dispatch, `cond`, closures capturing a passed-in environment, `def`/`defmacro` via `my-eval-program`. Has one documented, tested boundary: a recursive top-level `def` can't see itself (the host environment is immutable — see G4's category-boundary note in `docs/language-core-axioms.md`).

### Knowledge domains loaded on top (`knowledge/*.my`)

Three example domains, all growing over time, not frozen snapshots: `physics.my`/`astronomy.my` (small, single-hop rules), `family.my` (a three-generation family tree with a genuinely recursive `ancestor` rule, added 2026-08-09 specifically to exercise the engine's recursive-logic capability beyond one hop).

### What's honestly not there yet

No vectors or hash tables (only pairs/lists). No mutable state beyond `def`/`defmacro` rebinding a name in the current frame — no `set!`, no mutable cells. No file-write primitive (`read-file` exists, nothing writes). No string manipulation beyond `symbol->string`/`string->symbol`/`string-first`/`string-rest` (no `string-append`, no `string-length`). No first-class continuations. No multi-threading or concurrency. The free-text/LLM half of the natural-language bridge (`private/lisp-to-knowledge.md`, principle 5) is deliberately deferred — the controlled-natural-language half above is real and tested, the LLM-facing half doesn't exist yet. `fpga-lisp`, the second implementation, has hardware-verified 5 of 7 primitives (M01–M05) but doesn't run `conformance.my` itself yet (`PLAN.md`, "Відкрито" section). None of this is a defect list — see `docs/language-core-axioms.md` principle 1 ("write about possibilities, not limitations"); it's an honest inventory so a reader knows what's verified today versus planned.

---

## Українська

**Що це за документ.** `docs/language-core.md` свідомо обмежується Рівнем 1 (CORE SEMANTICS) і Рівнем 2 (LANGUAGE CONTRACT) — сім примітивів Маккарті, bootstrap-ядро, арифметика, порівняння, I/O. Цей документ — повна картина по всіх трьох рівнях (`docs/conformance-tier-map.md`), включно з Рівнем 3 (ECOSYSTEM CONFORMANCE — символьний reasoning-шар), якого `language-core.md` свідомо не покриває. Перевірено проти реального диспетчера (`crates/my-lisp/src/eval/mod.rs`) і верхньорівневих визначень кожного `lib/*.my`-файлу 2026-08-09, не написано з пам'яті.

### Рівень 1 — сім примітивів + bootstrap-ядро

`quote`, `atom`, `eq`, `car`, `cdr`, `cons`, `cond` (оригінальна сімка Маккарті), плюс `lambda`/`def`/`defmacro` (мінімальне семантичне ядро, потрібне для саморозгортання решти). Повне обґрунтування й межі — в `docs/language-core.md`/`docs/language-core-axioms.md`.

### Рівень 2 — мовний контракт

- **Арифметика**: `+`, `-`, `*`, `/`. Точні раціональні числа (на основі `BigInt` довільної точності, `crates/my-lisp/src/bignum.rs`) з реальною моделлю числової точності (`Exactness::Exact`/`Inexact` як властивість значення, не форми запису) — `(eq 3 3.0)` → `()`, `(= 3 3.0)` → `t`. Один неточний операнд робить результат неточним.
- **Порівняння**: `<`, `>`, `=`, `<=`, `>=`, варіативні й ланцюгові (`(< 1 2 3)`), те саме правило promotion, що й арифметика.
- **I/O та рефлексія**: `print`/`princ` (транскрипт виводу сесії, семантика `write`/`prin1` проти `display`), `read`/`read-file`/`read-all` (reader даних, не evaluator), `eval` (замикає read-eval цикл вручну), `load` (читає й обчислює файл).
- **Інтроспекція**: `symbol?`, `string?`, `symbol->string`, `string->symbol`, `string-first`, `string-rest`.
- **Структуровані помилки**: сім названих категорій `ErrorKind` (`Parse`, `UnknownSymbol`, `Arity`, `Type`, `InvalidForm`, `NumericOverflow`, `OutOfMemory`) — кожен провал названий і спостережуваний (S2), ніколи не мовчазний. `NumericOverflow`/`OutOfMemory` — опційні межі ресурсу (`Environment::with_cons_limit`/`with_numeric_bit_limit`) — типова сесія, і кожна фікстура `conformance.my`, лишається необмеженою.

### Стандартна бібліотека (`lib/core.my`)

`identity`, `not`, `pair`, `second`, `third`, `caar`, `cadr`, `equal?` (структурна рівність), `length`, `reverse`, `append`, `map`, `filter`, `reduce`, `let`, `let*`. `length`/`map`/`filter`/`reverse`/`append` — усі хвостово-рекурсивні (stack-safe на списку зі 100 000 елементів, `crates/my-lisp/tests/stack_safety.rs`). Варіативні параметри `lambda`/`defmacro` (`(a b . rest)`, голий `args`) — те, завдяки чому сам `list` перейшов зі спеціальної форми Rust у `lib/core.my`.

### Рівень 3 — символьний reasoning-шар (реальна суть проєкту, принцип 3)

Тут живе більшість реальної спроможності мови, і саме сюди `docs/language-core.md` свідомо не заходить.

- **`lib/unify.my`** — примітив унікації: логічні змінні (`(var name)`), `walk`/`walk-resolved` (розіменування), структурна (composite-term) унікація, occurs-check (запобігає нескінченним структурам), `apply-subst` (читання результатів), `thread-conjunction`/`thread-conjunction-branches` — спільне ядро обходу кон'юнкції, на якому будують обидва рушії нижче.
- **`lib/reason.my`** — backward-chaining рушій (micro-Prolog, сам "Advice Taker"): `reason` бере ціль і список правил, повертає кожну валідну підстановку. Standardizing apart через перейменування змінних, теговане глибиною, вмикає реальні рекурсивні правила. Negation as failure (`not`). `explain-proof`/`reason-explain` рендерять зрозумілу людині трасу й розрізняють "доведено" від "не можу довести". `count-usage`/`provenance` перетворюють дерево доведення на лічильники використання й записи `(statement ціль (source fact|rule) ...)` — явне походження, не лише відповідь.
- **`lib/forward.my`** (1204 рядки — найбільший файл бібліотеки) — forward-chaining рушій у стилі CLIPS: `assert-fact!`/`retract-fact!` над глобальною working memory, `run`/`run-multi` до fixpoint, квантифіковані/складені умови `not`/`or`/`and`/`test`/`exists`/`forall`, система підтримки істинності з одним обґрунтуванням і повний JTMS із множинними обґрунтуваннями (`run-jtms-multi!`) — прибирання факту коректно прибирає все, що від нього залежало, транзитивно.
- **`lib/knowledge.my`** — модульні пакети знань (`defmodule`) над append-only `*knowledge-journal*` (плаский лог подій `tell`/`retract`; поточні clause модуля — проекція, обчислена на вимогу, ніколи не мутований знімок). `reason-in`/`forward-in` запитують названий модуль через будь-який із рушіїв вище. `describe` збирає всі відомі факти про символ (атом як вхід у поняття). `record-usage!`/`usage-of` рахують застосування правил між окремими запитами верхнього рівня.
- **`lib/understand.my`** / **`lib/narrate.my`** — місток контрольованої природної мови, без LLM: фіксовані форми списку слів (`X is a Y`, `X V Y`, `all X have Y`) зіставляються зі знаннєвими clause й назад, `narrate-fact` — точна структурна обернена функція до `understand-is`/`understand-relation`.
- **`lib/clips-import.my`** — реальний імпортер CLIPS-файлів (NASA JSC, 1985): `deffacts`/`defrule`/`deftemplate` → clause, готові для `defmodule`, синтаксис змінних `?x`, обрізання salience/докстрінгу, мультислот-умови. Перевірено на дев'яти реальних зовнішніх `.clp`-файлах, включно з власним Sudoku-solver-ом CLIPS (`tests/fixtures/sudoku-external.clp`), не на синтетичних прикладах.
- **`lib/meta-eval.my`** — метациркулярний evaluator (`my-eval`/`my-apply`), написаний самою my-lisp: self-evaluation, `quote`, диспетчеризація арифметики/list-примітивів, `cond`, замикання, що захоплюють передане середовище, `def`/`defmacro` через `my-eval-program`. Має одну задокументовану, протестовану межу: рекурсивний верхньорівневий `def` не бачить сам себе (host-середовище незмінне — див. примітку про межу категорії G4 в `docs/language-core-axioms.md`).

### Домени знань поверх цього (`knowledge/*.my`)

Три приклади доменів, усі ростуть з часом, не застиглі знімки: `physics.my`/`astronomy.my` (маленькі, односкокові правила), `family.my` (родинне дерево з трьох поколінь зі справді рекурсивним правилом `ancestor`, додане 2026-08-09 саме для перевірки рекурсивної спроможності рушія за межами одного скоку).

### Чого чесно ще немає

Ні векторів, ні хеш-таблиць (лише пари/списки). Ні мутабельного стану поза перезв'язуванням імені в поточному фреймі через `def`/`defmacro` — ні `set!`, ні мутабельних комірок. Ні примітиву запису у файл (`read-file` є, нічого не пише). Ні маніпуляції рядками поза `symbol->string`/`string->symbol`/`string-first`/`string-rest` (ні `string-append`, ні `string-length`). Ні continuations першого класу. Ні багатопотоковості чи конкурентності. Вільнотекстова/LLM-половина мосту природної мови (`private/lisp-to-knowledge.md`, принцип 5) свідомо відкладена — половина з контрольованою природною мовою вище реальна й протестована, LLM-звернена половина ще не існує. `fpga-lisp`, друга реалізація, апаратно перевірила 5 із 7 примітивів (M01–M05), але ще не запускає сам `conformance.my` (`PLAN.md`, секція "Відкрито"). Ніщо з цього — не список дефектів — див. принцип 1 у `docs/language-core-axioms.md` ("писати про можливості, не про обмеження"); це чесний інвентар, щоб читач знав, що перевірено сьогодні, а що заплановано.

---

## Deutsch

**Was dieses Dokument ist.** `docs/language-core.md` beschränkt sich bewusst auf Stufe 1 (CORE SEMANTICS) und Stufe 2 (LANGUAGE CONTRACT) — die sieben McCarthy-Primitive, den Bootstrap-Kern, Arithmetik, Vergleiche, I/O. Dieses Dokument ist das vollständige Bild über alle drei Stufen (`docs/conformance-tier-map.md`), einschließlich Stufe 3 (ECOSYSTEM CONFORMANCE — die symbolische Schlussfolgerungsschicht), die `language-core.md` bewusst nicht abdeckt. Geprüft gegen den tatsächlichen Dispatcher (`crates/my-lisp/src/eval/mod.rs`) und die Top-Level-Definitionen jeder `lib/*.my`-Datei am 2026-08-09, nicht aus dem Gedächtnis geschrieben.

### Stufe 1 — die sieben Primitive + Bootstrap-Kern

`quote`, `atom`, `eq`, `car`, `cdr`, `cons`, `cond` (McCarthys ursprüngliche sieben), plus `lambda`/`def`/`defmacro` (der minimale semantische Kern, der zum Bootstrap des Restes nötig ist). Vollständige Begründung und Grenzen in `docs/language-core.md`/`docs/language-core-axioms.md`.

### Stufe 2 — der Sprachvertrag

- **Arithmetik**: `+`, `-`, `*`, `/`. Exakte rationale Zahlen (auf Basis eines beliebig genauen `BigInt`, `crates/my-lisp/src/bignum.rs`) mit einem echten Exactness-Modell (`Exactness::Exact`/`Inexact` als Eigenschaft des Werts, nicht seiner Schreibweise) — `(eq 3 3.0)` → `()`, `(= 3 3.0)` → `t`. Ein inexakter Operand macht ein Ergebnis inexakt.
- **Vergleiche**: `<`, `>`, `=`, `<=`, `>=`, variadisch und verkettet (`(< 1 2 3)`), dieselbe Promotionsregel wie die Arithmetik.
- **I/O und Reflexion**: `print`/`princ` (Ausgabetranskript der Sitzung, `write`/`prin1`- vs. `display`-Semantik), `read`/`read-file`/`read-all` (Daten-Reader, kein Evaluator), `eval` (schließt die Read-Eval-Schleife von Hand), `load` (liest und wertet eine Datei aus).
- **Introspektion**: `symbol?`, `string?`, `symbol->string`, `string->symbol`, `string-first`, `string-rest`.
- **Strukturierte Fehler**: sieben benannte `ErrorKind`-Kategorien (`Parse`, `UnknownSymbol`, `Arity`, `Type`, `InvalidForm`, `NumericOverflow`, `OutOfMemory`) — jeder Fehlschlag ist benannt und beobachtbar (S2), nie stumm. `NumericOverflow`/`OutOfMemory` sind Opt-in-Ressourcengrenzen (`Environment::with_cons_limit`/`with_numeric_bit_limit`) — die Standardsitzung, und jede `conformance.my`-Fixture, bleibt unbeschränkt.

### Standardbibliothek (`lib/core.my`)

`identity`, `not`, `pair`, `second`, `third`, `caar`, `cadr`, `equal?` (strukturelle Gleichheit), `length`, `reverse`, `append`, `map`, `filter`, `reduce`, `let`, `let*`. `length`/`map`/`filter`/`reverse`/`append` sind alle endrekursiv (stack-sicher bei einer 100.000-Elemente-Liste, `crates/my-lisp/tests/stack_safety.rs`). Variadische `lambda`/`defmacro`-Parameter (`(a b . rest)`, nacktes `args`) sind der Grund, warum `list` selbst von einer Rust-Sonderform in `lib/core.my` wechseln konnte.

### Stufe 3 — die symbolische Schlussfolgerungsschicht (der eigentliche Zweck des Projekts, Prinzip 3)

Hier lebt der größte Teil der echten Fähigkeit der Sprache, und genau hier geht `docs/language-core.md` bewusst nicht hin.

- **`lib/unify.my`** — das Unifikations-Primitiv: logische Variablen (`(var name)`), `walk`/`walk-resolved` (Dereferenzierung), strukturelle (zusammengesetzte) Unifikation, Occurs-Check (verhindert unendliche Strukturen), `apply-subst` (Ergebnisse auslesen), `thread-conjunction`/`thread-conjunction-branches` — der gemeinsame Konjunktions-Durchlaufkern, auf dem beide Engines unten aufbauen.
- **`lib/reason.my`** — eine Backward-Chaining-Engine (Micro-Prolog, der "Advice Taker" selbst): `reason` nimmt ein Ziel und eine Regelliste, liefert jede gültige Substitution. Standardizing apart über tiefenmarkierte Variablenumbenennung ermöglicht echte rekursive Regeln. Negation als Fehlschlag (`not`). `explain-proof`/`reason-explain` rendern eine menschenlesbare Spur und unterscheiden "bewiesen" von "nicht beweisbar". `count-usage`/`provenance` wandeln einen Beweisbaum in Nutzungszähler und `(statement ziel (source fact|rule) ...)`-Datensätze um — explizite Herkunft, nicht nur eine Antwort.
- **`lib/forward.my`** (1204 Zeilen — die größte Bibliotheksdatei) — eine CLIPS-artige Forward-Chaining-Engine: `assert-fact!`/`retract-fact!` auf einem globalen Arbeitsspeicher, `run`/`run-multi` bis zum Fixpunkt, quantifizierte/zusammengesetzte Bedingungen `not`/`or`/`and`/`test`/`exists`/`forall`, ein Truth-Maintenance-System mit einer Begründung und ein vollständiges JTMS mit mehreren Begründungen (`run-jtms-multi!`) — das Zurücknehmen eines Fakts nimmt korrekt alles zurück, was transitiv davon abhing.
- **`lib/knowledge.my`** — modulare Wissenspakete (`defmodule`) auf einem append-only `*knowledge-journal*` (ein flaches `tell`/`retract`-Ereignisprotokoll; die aktuellen Clauses eines Moduls sind eine auf Abruf berechnete Projektion, nie ein mutierter Schnappschuss). `reason-in`/`forward-in` fragen ein benanntes Modul über eine der beiden Engines oben ab. `describe` sammelt alle bekannten Fakten über ein Symbol (Atom als Konzept-Einstiegspunkt). `record-usage!`/`usage-of` verfolgen Regelauslösungen über getrennte Top-Level-Anfragen hinweg.
- **`lib/understand.my`** / **`lib/narrate.my`** — eine Brücke kontrollierter natürlicher Sprache, ohne LLM: feste Wortlisten-Formen (`X is a Y`, `X V Y`, `all X have Y`) bilden auf Wissens-Clauses ab und zurück, mit `narrate-fact` als exakter struktureller Umkehrung von `understand-is`/`understand-relation`.
- **`lib/clips-import.my`** — ein echter Importer für CLIPS-Quelldateien (NASA JSC, 1985): `deffacts`/`defrule`/`deftemplate` → `defmodule`-fertige Clauses, `?x`-Variablensyntax, Entfernen von Salience/Docstring-Präambeln, Multislot-Bedingungen. Geprüft an neun echten externen `.clp`-Dateien, einschließlich CLIPS' eigenem Sudoku-Löser (`tests/fixtures/sudoku-external.clp`), nicht an synthetischen Beispielen.
- **`lib/meta-eval.my`** — ein metazirkulärer Evaluator (`my-eval`/`my-apply`), in my-lisp selbst geschrieben: Selbstauswertung, `quote`, Dispatch von Arithmetik-/Listen-Primitiven, `cond`, Closures, die eine übergebene Umgebung erfassen, `def`/`defmacro` über `my-eval-program`. Hat eine dokumentierte, getestete Grenze: ein rekursives Top-Level-`def` sieht sich selbst nicht (die Host-Umgebung ist unveränderlich — siehe G4s Kategoriegrenzen-Notiz in `docs/language-core-axioms.md`).

### Darauf aufbauende Wissensdomänen (`knowledge/*.my`)

Drei Beispieldomänen, alle mit der Zeit wachsend, keine eingefrorenen Schnappschüsse: `physics.my`/`astronomy.my` (klein, einstufige Regeln), `family.my` (ein dreigenerationaler Familienbaum mit einer echt rekursiven `ancestor`-Regel, hinzugefügt am 2026-08-09 speziell um die rekursive Fähigkeit der Engine über einen Schritt hinaus zu prüfen).

### Was ehrlich noch fehlt

Keine Vektoren oder Hash-Tabellen (nur Paare/Listen). Kein veränderlicher Zustand über das Neubinden eines Namens im aktuellen Frame durch `def`/`defmacro` hinaus — kein `set!`, keine mutierbaren Zellen. Kein Datei-Schreib-Primitiv (`read-file` existiert, nichts schreibt). Keine String-Manipulation über `symbol->string`/`string->symbol`/`string-first`/`string-rest` hinaus (kein `string-append`, kein `string-length`). Keine erstklassigen Continuations. Keine Multithreading- oder Nebenläufigkeitsfähigkeit. Die Freitext-/LLM-Hälfte der Sprachbrücke (`private/lisp-to-knowledge.md`, Prinzip 5) ist bewusst aufgeschoben — die kontrollierte Hälfte oben ist real und getestet, die LLM-zugewandte Hälfte existiert noch nicht. `fpga-lisp`, die zweite Implementierung, hat 5 von 7 Primitiven hardwareverifiziert (M01–M05), führt aber `conformance.my` selbst noch nicht aus (`PLAN.md`, Abschnitt "Відкрито"). Nichts davon ist eine Mängelliste — siehe Prinzip 1 in `docs/language-core-axioms.md` ("über Möglichkeiten schreiben, nicht über Einschränkungen"); es ist ein ehrliches Inventar, damit Leser wissen, was heute verifiziert ist und was geplant.

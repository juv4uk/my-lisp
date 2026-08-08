# my-lisp test results · Результати тестів my-lisp · my-lisp-Testergebnisse

## English

This repository has one test layer: the four Rust crates under `crates/`, run with `cargo test --workspace`. There is no separate JS/web test suite here — the standalone web REPL (`public/my-lisp-cli-web.html`) is built from the same WASM crate covered below, and the previous Node/Playwright suite stayed behind in the `my-idea` IDE repo this crate set was extracted from (see [`docs/versioning.md`](versioning.md)). This table is the current source of truth and should be refreshed whenever a suite gains or loses tests.

| Crate | Suite | Tests | Covers | Result (last run) |
|---|---|---:|---|---|
| `my-lisp` | unit tests (`src/parser.rs`, `src/environment.rs`, `src/eval/mod.rs`, `src/error.rs`, `src/bignum.rs`) | 35 | reader/parser edge cases, lexical-scope isolation, single-pass evaluation, macro expansion, char-based line/column and caret rendering for structured errors, the hand-rolled arbitrary-precision `BigInt` (add/sub/mul/div/gcd/ordering/decimal parsing and formatting) backing `Rational` | ok |
| `my-lisp` | `tests/mccarthy.rs` | 27 | the seven McCarthy primitives, exact/inexact arithmetic, `<`/`>`/`=` comparison chaining, `print`'s session-wide output transcript (including through closures), `read`/`eval` closing the read-eval loop by hand, lambda semantics, structured errors, `lib/core.my` list utilities (`length`, `reverse`, `append`, `map`, `filter`, `reduce`), `let`/`let*`, `equal?`, exact arithmetic past `i64` range (factorial of 30, computed exactly) | ok |
| `my-lisp` | `tests/stack_safety.rs` | 5 | tail recursion and deep list clone/drop use constant Rust stack, `lib/core.my`'s `append`/`filter`/`map`/`length` stay stack-safe on a 100,000-element list | ok |
| `my-lisp-cli` | `tests/cli.rs` | 10 | the compiled binary end-to-end: `--version`/`--help`, file execution, parse/eval error exit codes, missing-file handling, `lib/core.my` preloading, REPL history persisting to `~/.my-lisp-history` across separate sessions, `(read)` reading one line from real piped stdin in file mode | ok |
| `my-lisp` | `tests/meta_eval.rs` | 9 | `lib/meta-eval.my`, the metacircular evaluator: self-evaluation, `quote`, arithmetic/list-primitive dispatch, `cond`, lambda application (single- and multi-expression bodies), closures capturing free variables from a passed-in env, higher-order functions | ok |
| `my-lisp` | `tests/unify.rs` | 10 | `lib/unify.my`, the unification primitive: atom matching, variable binding/resolution, structural (compound-term) unification, mismatch failure, transitive chained-variable resolution, same-variable unification creating no binding, full-query `apply-subst`, occurs-check | ok |
| `my-lisp` | `tests/reason.rs` | 13 | `lib/reason.my`, the backward-chaining engine: simple facts, recursive rules, standardizing apart, negation as failure, proof trees and `explain-proof` trace output, `reason-explain`'s "provable" vs "cannot prove" distinction, `count-usage` walking a proof tree into a `(rule-head . times-used)` tally, `provenance` turning a proof node into a `(statement goal (source fact\|rule) (rule ...) (derived-from ...))` record | ok |
| `my-lisp` | `tests/knowledge.rs` | 8 | `lib/knowledge.my`, modular knowledge packages: `defmodule`, isolated per-module queries via `reason-in`, conflict detection in `tell-knowledge`, `describe` collecting every known fact about a symbol (the "atom as concept entry point" idea), `record-usage!`/`usage-of` accumulating rule-usage counts across separate top-level queries | ok |
| `my-lisp` | `tests/understand.rs` | 5 | `lib/understand.my`, the controlled-natural-language bridge: fixed word-list shapes (`X is a Y`, `X V Y`, `all X have Y`) mapped to knowledge clauses, and the result fed straight into `reason` with no hand-editing step | ok |
| `my-lisp` | `tests/forward.rs` | 3 | `lib/forward.my`, Step 1 of a CLIPS-style forward-chaining engine: `assert-fact!` growing the global working memory, `fire-rule` unifying a rule's pattern against one fact and substituting into its template, or returning `no-match` on failure | ok |
| `my-lisp-literate` | `tests/literate_offsets.rs` | 4 | literate-Markdown source-offset mapping | ok |
| `my-lisp-wasm` | unit test (`src/lib.rs`) | 1 | the WASM adapter produces the same exact/single-pass evaluation struct as the native core | ok |
| **Total** | | **130** | | **130 passed, 0 failed, 0 ignored** |

The implementation-independent conformance fixture at [`tests/fixtures/conformance.json`](../tests/fixtures/conformance.json) — its own format and rules are in [`tests/fixtures/README.md`](../tests/fixtures/README.md) — is included directly into `crates/my-lisp/tests/mccarthy.rs` via `include_str!` and is exercised as part of that suite's 27 tests, not counted separately.

```bash
cargo test --workspace
```

Last recorded run: 2026-08-08, Windows x86_64 — all passing, 0 failed, 0 ignored.

**Reader caveat surfaced while writing the usage-count tests**: the reader has no dotted-pair literal syntax — `'(p . 0)` parses `.` as an ordinary symbol, producing a 3-element proper list rather than a real dotted pair, even though the printer renders true dotted pairs exactly that way (`(x . alice)` in [`tests/reason.rs`](../crates/my-lisp/tests/reason.rs)'s `variable_binding_from_fact`, for instance). So a quoted `'(p . 0)` and an actual `(cons 'p 0)` are not `equal?`. Not fixed here — noted so the next person doesn't lose an hour to it the way this session did.

## Українська

Цей репозиторій має один шар тестів: чотири Rust-крейти в `crates/`, що запускаються через `cargo test --workspace`. Окремого JS/web тест-набору тут немає — автономний web-REPL (`public/my-lisp-cli-web.html`) збирається з того самого WASM-крейта, покритого нижче, а попередній Node/Playwright-набір лишився в репозиторії IDE `my-idea`, з якого виділено цей набір крейтів (див. [`docs/versioning.md`](versioning.md)). Ця таблиця є поточним джерелом правди й має оновлюватися щоразу, коли набір отримує або втрачає тести.

| Крейт | Набір | Тестів | Покриває | Результат (останній запуск) |
|---|---|---:|---|---|
| `my-lisp` | unit-тести (`src/parser.rs`, `src/environment.rs`, `src/eval/mod.rs`, `src/error.rs`, `src/bignum.rs`) | 35 | межові випадки reader/parser, ізоляцію лексичного скоупу, однопрохідне обчислення, розкриття макросів, char-based рядок/стовпець і рендер "^" для структурованих помилок, власноруч написаний `BigInt` довільної точності (add/sub/mul/div/gcd/порівняння/парсинг і форматування десяткового рядка), що лежить під `Rational` | ok |
| `my-lisp` | `tests/mccarthy.rs` | 27 | сім примітивів Маккарті, точну/неточну арифметику, ланцюгове порівняння `<`/`>`/`=`, транскрипт виводу `print`, спільний на сесію (включно через замикання), `read`/`eval`, що замикають read-eval цикл вручну, семантику lambda, структуровані помилки, list-утиліти `lib/core.my` (`length`, `reverse`, `append`, `map`, `filter`, `reduce`), `let`/`let*`, `equal?`, точну арифметику за межею `i64` (факторіал 30, обчислений точно) | ok |
| `my-lisp` | `tests/stack_safety.rs` | 5 | хвостову рекурсію та clone/drop глибоких списків зі сталим Rust-стеком, `append`/`filter`/`map`/`length` з `lib/core.my` лишаються stack-safe на списку зі 100 000 елементів | ok |
| `my-lisp-cli` | `tests/cli.rs` | 10 | скомпільований бінарник наскрізно: `--version`/`--help`, виконання файлу, коди виходу при помилках парсингу/обчислення, відсутній файл, попереднє завантаження `lib/core.my`, збереження історії REPL у `~/.my-lisp-history` між окремими сесіями, `(read)` читає один рядок зі справжнього переданого через pipe stdin у файловому режимі | ok |
| `my-lisp` | `tests/meta_eval.rs` | 9 | `lib/meta-eval.my`, метациркулярний evaluator: self-evaluation, `quote`, диспетчеризацію арифметики/list-примітивів, `cond`, застосування lambda (одно- й багатовиразові тіла), замикання, що захоплюють вільні змінні з переданого env, функції вищого порядку | ok |
| `my-lisp` | `tests/unify.rs` | 10 | `lib/unify.my`, примітив unification: зіставлення атомів, зв'язування/розв'язування змінних, структурну (composite-term) унікацію, провал при невідповідності, транзитивне розв'язування ланцюжка змінних, унікацію змінної з собою без зв'язку, повний `apply-subst` запиту, occurs-check | ok |
| `my-lisp` | `tests/reason.rs` | 13 | `lib/reason.my`, рушій логічного висновування: прості факти, рекурсивні правила, standardizing apart, negation as failure, дерева доведень і трасування `explain-proof`, розрізнення "доведено"/"не можу довести" в `reason-explain`, `count-usage` — обхід дерева доведення в таблицю `(голова-правила . скільки-разів-використано)`, `provenance` — перетворення вузла дерева доведення на запис `(statement ціль (source fact\|rule) (rule ...) (derived-from ...))` | ok |
| `my-lisp` | `tests/knowledge.rs` | 8 | `lib/knowledge.my`, модульні пакети знань: `defmodule`, ізольовані запити по модулю через `reason-in`, виявлення конфліктів у `tell-knowledge`, `describe` — збір усіх відомих фактів про символ (ідея "атом як вхід у поняття"), `record-usage!`/`usage-of` — накопичення лічильників використання правил між окремими запитами верхнього рівня | ok |
| `my-lisp` | `tests/understand.rs` | 5 | `lib/understand.my`, місток контрольованої природної мови: фіксовані форми списку слів (`X is a Y`, `X V Y`, `all X have Y`), зіставлені зі знаннєвими clause, і результат, що напряму йде в `reason` без ручного редагування | ok |
| `my-lisp` | `tests/forward.rs` | 3 | `lib/forward.my`, Крок 1 forward-chaining рушія в стилі CLIPS: `assert-fact!` — розширення глобальної working memory, `fire-rule` — унікація шаблону правила з одним фактом і підстановка в template, або `no-match` при невдачі | ok |
| `my-lisp-literate` | `tests/literate_offsets.rs` | 4 | зіставлення зміщень початкового коду literate-Markdown | ok |
| `my-lisp-wasm` | unit-тест (`src/lib.rs`) | 1 | WASM-адаптер видає ту саму точну/однопрохідну структуру обчислення, що й нативне ядро | ok |
| **Разом** | | **130** | | **130 пройдено, 0 провалів, 0 пропущено** |

Незалежна від реалізації conformance-фікстура [`tests/fixtures/conformance.json`](../tests/fixtures/conformance.json) — власний формат і правила описані в [`tests/fixtures/README.md`](../tests/fixtures/README.md) — підключається напряму в `crates/my-lisp/tests/mccarthy.rs` через `include_str!` і перевіряється в межах тих 27 тестів набору, окремо не рахується.

```bash
cargo test --workspace
```

Останній зафіксований запуск: 2026-08-08, Windows x86_64 — усі проходять, 0 провалів, 0 пропущено.

**Застереження щодо reader, знайдене під час написання тестів для лічильника використання**: reader не має синтаксису dotted-pair-літералів — `'(p . 0)` парсить `.` як звичайний символ, даючи 3-елементний власний список замість справжньої dotted pair, хоча printer саме так друкує справжні dotted pairs (наприклад `(x . alice)` у `variable_binding_from_fact` в [`tests/reason.rs`](../crates/my-lisp/tests/reason.rs)). Тож quoted `'(p . 0)` і реальний `(cons 'p 0)` не є `equal?`. Не виправлено тут — лишено як нотатка, щоб наступна людина не втратила на цьому годину, як ця сесія.

## Deutsch

Dieses Repository hat eine Testebene: die vier Rust-Crates unter `crates/`, ausgeführt mit `cargo test --workspace`. Eine separate JS/Web-Testsuite gibt es hier nicht — der eigenständige Web-REPL (`public/my-lisp-cli-web.html`) wird aus demselben WASM-Crate gebaut, das unten abgedeckt ist, und die frühere Node/Playwright-Suite blieb im `my-idea`-IDE-Repository zurück, aus dem dieser Crate-Satz extrahiert wurde (siehe [`docs/versioning.md`](versioning.md)). Diese Tabelle ist die aktuelle Quelle der Wahrheit und sollte aktualisiert werden, sobald eine Suite Tests gewinnt oder verliert.

| Crate | Suite | Tests | Deckt ab | Ergebnis (letzter Lauf) |
|---|---|---:|---|---|
| `my-lisp` | Unit-Tests (`src/parser.rs`, `src/environment.rs`, `src/eval/mod.rs`, `src/error.rs`, `src/bignum.rs`) | 35 | Reader-/Parser-Grenzfälle, Isolation des lexikalischen Scopes, Single-Pass-Auswertung, Makro-Expansion, zeichenbasierte Zeile/Spalte und "^"-Rendering für strukturierte Fehler, das von Hand geschriebene beliebig genaue `BigInt` (add/sub/mul/div/ggT/Ordnung/Dezimal-Parsing und -Formatierung) hinter `Rational` | ok |
| `my-lisp` | `tests/mccarthy.rs` | 27 | die sieben McCarthy-Primitive, exakte/inexakte Arithmetik, verkettete Vergleiche `<`/`>`/`=`, `print`s sitzungsweites Ausgabetranskript (auch durch Closures hindurch), `read`/`eval`, die die Read-Eval-Schleife von Hand schließen, Lambda-Semantik, strukturierte Fehler, `lib/core.my`-Listenwerkzeuge (`length`, `reverse`, `append`, `map`, `filter`, `reduce`), `let`/`let*`, `equal?`, exakte Arithmetik jenseits von `i64` (Fakultät von 30, exakt berechnet) | ok |
| `my-lisp` | `tests/stack_safety.rs` | 5 | Tail-Rekursion und Clone/Drop tiefer Listen mit konstantem Rust-Stack, `append`/`filter`/`map`/`length` aus `lib/core.my` bleiben stack-sicher bei einer 100.000-Elemente-Liste | ok |
| `my-lisp-cli` | `tests/cli.rs` | 10 | die kompilierte Binärdatei durchgängig: `--version`/`--help`, Dateiausführung, Exit-Codes bei Parse-/Eval-Fehlern, fehlende Datei, Vorladen von `lib/core.my`, REPL-Verlauf, der über getrennte Sitzungen in `~/.my-lisp-history` erhalten bleibt, `(read)`, das im Dateimodus eine Zeile aus echtem, per Pipe übergebenem stdin liest | ok |
| `my-lisp` | `tests/meta_eval.rs` | 9 | `lib/meta-eval.my`, der metazirkuläre Evaluator: Selbstauswertung, `quote`, Dispatch von Arithmetik-/Listen-Primitiven, `cond`, Lambda-Anwendung (ein- und mehrausdrucksweise Rümpfe), Closures, die freie Variablen aus einer übergebenen env erfassen, Funktionen höherer Ordnung | ok |
| `my-lisp` | `tests/unify.rs` | 10 | `lib/unify.my`, das Unifikations-Primitiv: Atom-Abgleich, Variablenbindung/-auflösung, strukturelle (zusammengesetzte) Unifikation, Fehlschlag bei Nichtübereinstimmung, transitive Auflösung verketteter Variablen, Unifikation einer Variable mit sich selbst ohne Bindung, vollständiges `apply-subst` einer Anfrage, occurs-check | ok |
| `my-lisp` | `tests/reason.rs` | 13 | `lib/reason.my`, die Inferenz-Engine: einfache Fakten, rekursive Regeln, Standardizing apart, negation as failure, Beweisbäume und `explain-proof`-Trace-Ausgabe, die Unterscheidung "beweisbar" vs. "nicht beweisbar" in `reason-explain`, `count-usage` — Durchlauf eines Beweisbaums zu einer `(regelkopf . anzahl-verwendungen)`-Tabelle, `provenance` — wandelt einen Beweisbaum-Knoten in einen Datensatz `(statement ziel (source fact\|rule) (rule ...) (derived-from ...))` um | ok |
| `my-lisp` | `tests/knowledge.rs` | 8 | `lib/knowledge.my`, modulare Wissenspakete: `defmodule`, isolierte Anfragen pro Modul über `reason-in`, Konflikterkennung in `tell-knowledge`, `describe` sammelt alle bekannten Fakten über ein Symbol (die Idee "Atom als Konzept-Einstiegspunkt"), `record-usage!`/`usage-of` akkumulieren Regel-Nutzungszähler über getrennte Top-Level-Anfragen hinweg | ok |
| `my-lisp` | `tests/understand.rs` | 5 | `lib/understand.my`, die Brücke kontrollierter natürlicher Sprache: feste Wortlisten-Formen (`X is a Y`, `X V Y`, `all X have Y`), abgebildet auf Wissens-Clauses, deren Ergebnis direkt ohne manuelle Bearbeitung in `reason` einfließt | ok |
| `my-lisp` | `tests/forward.rs` | 3 | `lib/forward.my`, Schritt 1 einer CLIPS-artigen Forward-Chaining-Engine: `assert-fact!` erweitert die globale Working Memory, `fire-rule` unifiziert das Muster einer Regel mit einem Fakt und setzt in die Vorlage ein, oder liefert `no-match` bei Fehlschlag | ok |
| `my-lisp-literate` | `tests/literate_offsets.rs` | 4 | Offset-Zuordnung von literate-Markdown-Quellcode | ok |
| `my-lisp-wasm` | Unit-Test (`src/lib.rs`) | 1 | der WASM-Adapter liefert dieselbe exakte/Single-Pass-Auswertungsstruktur wie der native Kern | ok |
| **Gesamt** | | **130** | | **130 bestanden, 0 fehlgeschlagen, 0 übersprungen** |

Die implementierungsunabhängige Konformitäts-Fixture [`tests/fixtures/conformance.json`](../tests/fixtures/conformance.json) — eigenes Format und eigene Regeln stehen in [`tests/fixtures/README.md`](../tests/fixtures/README.md) — wird direkt über `include_str!` in `crates/my-lisp/tests/mccarthy.rs` eingebunden und im Rahmen der 27 Tests dieser Suite geprüft, nicht separat gezählt.

```bash
cargo test --workspace
```

Letzter erfasster Lauf: 08.08.2026, Windows x86_64 — alle bestanden, 0 fehlgeschlagen, 0 übersprungen.

**Reader-Falle, entdeckt beim Schreiben der Nutzungszähler-Tests**: Der Reader kennt keine Syntax für Dotted-Pair-Literale — `'(p . 0)` parst `.` als gewöhnliches Symbol und erzeugt eine dreielementige echte Liste statt eines echten Dotted Pair, obwohl der Printer echte Dotted Pairs genau so ausgibt (z. B. `(x . alice)` in `variable_binding_from_fact` in [`tests/reason.rs`](../crates/my-lisp/tests/reason.rs)). Ein quotiertes `'(p . 0)` und ein tatsächliches `(cons 'p 0)` sind daher nicht `equal?`. Hier nicht behoben — als Hinweis festgehalten, damit die nächste Person nicht dieselbe Stunde verliert wie diese Sitzung.

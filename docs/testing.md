# my-lisp test results · Результати тестів my-lisp · my-lisp-Testergebnisse

## English

This repository has one test layer: the four Rust crates under `crates/`, run with `cargo test --workspace`. There is no separate JS/web test suite here — the standalone web REPL (`public/my-lisp-cli-web.html`) is built from the same WASM crate covered below, and the previous Node/Playwright suite stayed behind in the `my-idea` IDE repo this crate set was extracted from (see [`docs/versioning.md`](versioning.md)). This table is the current source of truth and should be refreshed whenever a suite gains or loses tests.

| Crate | Suite | Tests | Covers | Result (last run) |
|---|---|---:|---|---|
| `my-lisp` | unit tests (`src/parser.rs`, `src/environment.rs`, `src/eval/mod.rs`, `src/error.rs`) | 27 | reader/parser edge cases, lexical-scope isolation, single-pass evaluation, macro expansion, char-based line/column and caret rendering for structured errors | ok |
| `my-lisp` | `tests/mccarthy.rs` | 13 | the seven McCarthy primitives, exact/inexact arithmetic, lambda semantics, structured errors, `lib/core.my` list utilities (`length`, `reverse`, `append`, `map`, `filter`, `reduce`) | ok |
| `my-lisp` | `tests/stack_safety.rs` | 4 | tail recursion and deep list clone/drop use constant Rust stack | ok |
| `my-lisp-cli` | `tests/cli.rs` | 8 | the compiled binary end-to-end: `--version`/`--help`, file execution, parse/eval error exit codes, missing-file handling, `lib/core.my` preloading | ok |
| `my-lisp-literate` | `tests/literate_offsets.rs` | 4 | literate-Markdown source-offset mapping | ok |
| `my-lisp-wasm` | unit test (`src/lib.rs`) | 1 | the WASM adapter produces the same exact/single-pass evaluation struct as the native core | ok |
| **Total** | | **57** | | **57 passed, 0 failed, 0 ignored** |

The implementation-independent conformance fixture at [`tests/fixtures/conformance.json`](../tests/fixtures/conformance.json) is included directly into `crates/my-lisp/tests/mccarthy.rs` via `include_str!` and is exercised as part of that suite's 13 tests, not counted separately.

```bash
cargo test --workspace
```

Last recorded run: 2026-08-08, Windows x86_64 — all passing, 0 failed, 0 ignored.

## Українська

Цей репозиторій має один шар тестів: чотири Rust-крейти в `crates/`, що запускаються через `cargo test --workspace`. Окремого JS/web тест-набору тут немає — автономний web-REPL (`public/my-lisp-cli-web.html`) збирається з того самого WASM-крейта, покритого нижче, а попередній Node/Playwright-набір лишився в репозиторії IDE `my-idea`, з якого виділено цей набір крейтів (див. [`docs/versioning.md`](versioning.md)). Ця таблиця є поточним джерелом правди й має оновлюватися щоразу, коли набір отримує або втрачає тести.

| Крейт | Набір | Тестів | Покриває | Результат (останній запуск) |
|---|---|---:|---|---|
| `my-lisp` | unit-тести (`src/parser.rs`, `src/environment.rs`, `src/eval/mod.rs`, `src/error.rs`) | 27 | межові випадки reader/parser, ізоляцію лексичного скоупу, однопрохідне обчислення, розкриття макросів, char-based рядок/стовпець і рендер "^" для структурованих помилок | ok |
| `my-lisp` | `tests/mccarthy.rs` | 13 | сім примітивів Маккарті, точну/неточну арифметику, семантику lambda, структуровані помилки, list-утиліти `lib/core.my` (`length`, `reverse`, `append`, `map`, `filter`, `reduce`) | ok |
| `my-lisp` | `tests/stack_safety.rs` | 4 | хвостову рекурсію та clone/drop глибоких списків зі сталим Rust-стеком | ok |
| `my-lisp-cli` | `tests/cli.rs` | 8 | скомпільований бінарник наскрізно: `--version`/`--help`, виконання файлу, коди виходу при помилках парсингу/обчислення, відсутній файл, попереднє завантаження `lib/core.my` | ok |
| `my-lisp-literate` | `tests/literate_offsets.rs` | 4 | зіставлення зміщень початкового коду literate-Markdown | ok |
| `my-lisp-wasm` | unit-тест (`src/lib.rs`) | 1 | WASM-адаптер видає ту саму точну/однопрохідну структуру обчислення, що й нативне ядро | ok |
| **Разом** | | **57** | | **57 пройдено, 0 провалів, 0 пропущено** |

Незалежна від реалізації conformance-фікстура [`tests/fixtures/conformance.json`](../tests/fixtures/conformance.json) підключається напряму в `crates/my-lisp/tests/mccarthy.rs` через `include_str!` і перевіряється в межах тих 13 тестів набору, окремо не рахується.

```bash
cargo test --workspace
```

Останній зафіксований запуск: 2026-08-08, Windows x86_64 — усі проходять, 0 провалів, 0 пропущено.

## Deutsch

Dieses Repository hat eine Testebene: die vier Rust-Crates unter `crates/`, ausgeführt mit `cargo test --workspace`. Eine separate JS/Web-Testsuite gibt es hier nicht — der eigenständige Web-REPL (`public/my-lisp-cli-web.html`) wird aus demselben WASM-Crate gebaut, das unten abgedeckt ist, und die frühere Node/Playwright-Suite blieb im `my-idea`-IDE-Repository zurück, aus dem dieser Crate-Satz extrahiert wurde (siehe [`docs/versioning.md`](versioning.md)). Diese Tabelle ist die aktuelle Quelle der Wahrheit und sollte aktualisiert werden, sobald eine Suite Tests gewinnt oder verliert.

| Crate | Suite | Tests | Deckt ab | Ergebnis (letzter Lauf) |
|---|---|---:|---|---|
| `my-lisp` | Unit-Tests (`src/parser.rs`, `src/environment.rs`, `src/eval/mod.rs`, `src/error.rs`) | 27 | Reader-/Parser-Grenzfälle, Isolation des lexikalischen Scopes, Single-Pass-Auswertung, Makro-Expansion, zeichenbasierte Zeile/Spalte und "^"-Rendering für strukturierte Fehler | ok |
| `my-lisp` | `tests/mccarthy.rs` | 13 | die sieben McCarthy-Primitive, exakte/inexakte Arithmetik, Lambda-Semantik, strukturierte Fehler, `lib/core.my`-Listenwerkzeuge (`length`, `reverse`, `append`, `map`, `filter`, `reduce`) | ok |
| `my-lisp` | `tests/stack_safety.rs` | 4 | Tail-Rekursion und Clone/Drop tiefer Listen mit konstantem Rust-Stack | ok |
| `my-lisp-cli` | `tests/cli.rs` | 8 | die kompilierte Binärdatei durchgängig: `--version`/`--help`, Dateiausführung, Exit-Codes bei Parse-/Eval-Fehlern, fehlende Datei, Vorladen von `lib/core.my` | ok |
| `my-lisp-literate` | `tests/literate_offsets.rs` | 4 | Offset-Zuordnung von literate-Markdown-Quellcode | ok |
| `my-lisp-wasm` | Unit-Test (`src/lib.rs`) | 1 | der WASM-Adapter liefert dieselbe exakte/Single-Pass-Auswertungsstruktur wie der native Kern | ok |
| **Gesamt** | | **57** | | **57 bestanden, 0 fehlgeschlagen, 0 übersprungen** |

Die implementierungsunabhängige Konformitäts-Fixture [`tests/fixtures/conformance.json`](../tests/fixtures/conformance.json) wird direkt über `include_str!` in `crates/my-lisp/tests/mccarthy.rs` eingebunden und im Rahmen der 13 Tests dieser Suite geprüft, nicht separat gezählt.

```bash
cargo test --workspace
```

Letzter erfasster Lauf: 08.08.2026, Windows x86_64 — alle bestanden, 0 fehlgeschlagen, 0 übersprungen.

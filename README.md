# my-lisp

**A small language that grows itself · Маленька мова, що вирощує себе · Eine kleine Sprache, die sich selbst wachsen lässt**

[English](#english) · [Українська](#українська) · [Deutsch](#deutsch)

## Quick try · Швидко спробувати · Schnell ausprobieren

No installation, no account: **[download the standalone web REPL](https://github.com/juv4uk/my-lisp/releases/latest/download/my-lisp-cli-web.html)** — a single portable `.html` file with a terminal-style REPL for the my-lisp core, running entirely in your browser. See [`docs/quote-tutorial.md`](docs/quote-tutorial.md) for a first walkthrough.

Без встановлення й облікового запису: **[завантажте автономний web-REPL](https://github.com/juv4uk/my-lisp/releases/latest/download/my-lisp-cli-web.html)** — один portable-файл `.html` з термінальним REPL для ядра my-lisp, що працює повністю в браузері. Перший огляд — у [`docs/quote-tutorial.md`](docs/quote-tutorial.md).

Ohne Installation und Benutzerkonto: **[den eigenständigen Web-REPL herunterladen](https://github.com/juv4uk/my-lisp/releases/latest/download/my-lisp-cli-web.html)** — eine einzelne portable `.html`-Datei mit einem terminalartigen REPL für den my-lisp-Kern, die vollständig im Browser läuft. Ein erster Rundgang steht in [`docs/quote-tutorial.md`](docs/quote-tutorial.md).

Or build the native CLI from source:

```bash
cargo run -p my-lisp-cli
cargo run -p my-lisp-cli -- path/to/file.my
```

## English

`my-lisp` is a Lisp built around McCarthy's seven primitives — `quote`, `atom`, `eq`, `car`, `cdr`, `cons`, `cond` — plus the minimal semantic kernel needed to bootstrap everything else: `lambda`, `def`, `defmacro`. Everything derivable from that kernel is written in my-lisp itself ([`lib/core.my`](lib/core.my)), not added as Rust built-ins. Full rationale: [`docs/language-core.md`](docs/language-core.md).

Exact rational arithmetic is a core purpose, not a nice-to-have: `/` on integers/rationals stays exact (`5/336`, not `0.0148...`), following Racket's exact/inexact distinction. `.my` is the canonical source extension; `.lisp` is a compatible alias.

### Philosophy

Rust provides only what it does exceptionally well — safe values, parsing, lexical closures, deterministic evaluation, stack control, diagnostics — and stops there. Every derived form, every stdlib function, grows in my-lisp itself instead of the Rust surface, whenever the existing kernel can already express it: `<=`/`>=` were a two-line addition to an existing dispatch table, not new machinery; `eval` reused the same data→code conversion macro expansion already needed, instead of duplicating it. This is the same instinct that kept John McCarthy from finishing M-expressions once S-expressions turned out to be enough.

That instinct is tested, not just stated: [`lib/meta-eval.my`](lib/meta-eval.my) is a metacircular evaluator — `eval`/`apply` written in my-lisp itself, the same relationship McCarthy's own 1960 paper had to its primitives, dispatching to `car`/`cdr`/`cons`/`atom`/`eq` rather than reimplementing them. [`lib/unify.my`](lib/unify.my) is a small unification engine in the spirit of McCarthy's 1958 "Advice Taker" proposal — symbolic pattern-matching, not statistics. Both are proof that "a small language that grows itself" is a working claim about this codebase, not a slogan.

[`tests/fixtures/conformance.json`](tests/fixtures/conformance.json) exists because Lisp's history is also a history of dialects drifting apart — MacLisp, InterLisp, a dozen Scheme variants, Common Lisp's attempt to reunify them. This file is the one thing a future C core or HDL Lisp-machine core must agree with, so a second implementation never becomes "just another dialect."

This repository is the canonical Rust implementation:

- [`crates/my-lisp`](crates/my-lisp) — the core: parser, evaluator, environments, exact-rational arithmetic.
- [`crates/my-lisp-cli`](crates/my-lisp-cli) — the `my-lisp` binary (REPL + file runner).
- [`crates/my-lisp-wasm`](crates/my-lisp-wasm) — WebAssembly bindings powering the browser REPL above.
- [`crates/my-lisp-literate`](crates/my-lisp-literate) — literate-Markdown source-offset mapping.
- [`lib/core.my`](lib/core.my) — the bootstrapped standard library.

### Build and test

```bash
cargo build --workspace
cargo test --workspace
```

### Docs

- [`docs/language-core.md`](docs/language-core.md) — the language contract: primitives, bootstrap boundary, exact arithmetic.
- [`docs/quote-tutorial.md`](docs/quote-tutorial.md) — a beginner walkthrough of homoiconicity.
- [`docs/unify-tutorial.md`](docs/unify-tutorial.md) — a small symbolic-AI example: unification, written in my-lisp itself.
- [`docs/testing.md`](docs/testing.md) — current test inventory.
- [`docs/benchmarks.md`](docs/benchmarks.md) — benchmark methodology and a local baseline.
- [`docs/versioning.md`](docs/versioning.md) — why this repo's version history looks the way it does.

`my-lisp` began inside a broader IDE project, [`my-idea`](https://github.com/juv4uk/my-idea), and was extracted here to stand on its own. Future implementations of the same language — a C core for embedded targets, and a from-scratch Lisp-machine HDL core — are planned as separate, parallel repositories.

## Українська

`my-lisp` — це Lisp, побудований навколо семи примітивів Маккарті — `quote`, `atom`, `eq`, `car`, `cdr`, `cons`, `cond` — плюс мінімальне семантичне ядро, потрібне для розгортання всього іншого: `lambda`, `def`, `defmacro`. Усе, що можна вивести з цього ядра, написане самою my-lisp ([`lib/core.my`](lib/core.my)), а не додане як Rust built-in. Повне обґрунтування — [`docs/language-core.md`](docs/language-core.md).

Точна раціональна арифметика — базова мета, а не бонус: `/` над цілими/раціональними лишається точним (`5/336`, а не `0.0148...`), за зразком розрізнення exact/inexact у Racket. `.my` — канонічне розширення початкового коду; `.lisp` — сумісний псевдонім.

### Філософія

Rust надає лише те, що робить особливо добре — безпечні значення, парсинг, лексичні замикання, детерміноване обчислення, контроль стека, діагностику — і на цьому зупиняється. Кожна похідна форма, кожна stdlib-функція росте самою my-lisp, а не в Rust-поверхні, щоразу, коли наявне ядро вже може це виразити: `<=`/`>=` були двома рядками до наявної таблиці диспетчеризації, не новою машинерією; `eval` перевикористав те саме перетворення дані→код, яке вже було потрібне для розгортання макросів, замість дублювання. Це той самий інстинкт, що не дав Джону Маккарті добудувати M-expressions, коли S-вирази виявились достатніми.

Цей інстинкт перевірений, не лише задекларований: [`lib/meta-eval.my`](lib/meta-eval.my) — метациркулярний evaluator: `eval`/`apply`, написані самою my-lisp, те саме відношення, яке власна стаття Маккарті 1960 року мала до своїх примітивів — диспетчеризує до `car`/`cdr`/`cons`/`atom`/`eq`, а не переписує їх заново. [`lib/unify.my`](lib/unify.my) — маленький механізм унікації в дусі пропозиції "Advice Taker" Маккарті 1958 року — символьне зіставлення з шаблоном, не статистика. Обидва — доказ того, що "маленька мова, що вирощує себе" — робоче твердження про цей код, не гасло.

[`tests/fixtures/conformance.json`](tests/fixtures/conformance.json) існує, бо історія Lisp — це також історія діалектів, що розбігались — MacLisp, InterLisp, десяток варіантів Scheme, спроба Common Lisp їх возз'єднати. Цей файл — те єдине, з чим має погоджуватись майбутнє C-ядро чи HDL-ядро Lisp-машини, щоб друга реалізація ніколи не стала "ще одним діалектом".

Цей репозиторій — канонічна реалізація на Rust:

- [`crates/my-lisp`](crates/my-lisp) — ядро: парсер, обчислювач, середовища, точна раціональна арифметика.
- [`crates/my-lisp-cli`](crates/my-lisp-cli) — бінарник `my-lisp` (REPL + запуск файлів).
- [`crates/my-lisp-wasm`](crates/my-lisp-wasm) — WebAssembly-біндінги для браузерного REPL вище.
- [`crates/my-lisp-literate`](crates/my-lisp-literate) — зіставлення зміщень початкового коду literate-Markdown.
- [`lib/core.my`](lib/core.my) — bootstrapped стандартна бібліотека.

### Збірка та тести

```bash
cargo build --workspace
cargo test --workspace
```

### Документація

- [`docs/language-core.md`](docs/language-core.md) — контракт мови: примітиви, межа bootstrap, точна арифметика.
- [`docs/quote-tutorial.md`](docs/quote-tutorial.md) — вступний огляд гомоіконічності.
- [`docs/unify-tutorial.md`](docs/unify-tutorial.md) — маленький приклад символьного AI: унікація, написана самою my-lisp.
- [`docs/testing.md`](docs/testing.md) — поточний перелік тестів.
- [`docs/benchmarks.md`](docs/benchmarks.md) — методологія бенчмарків і локальний baseline.
- [`docs/versioning.md`](docs/versioning.md) — чому історія версій цього репо саме така.

`my-lisp` починалась усередині ширшого IDE-проєкту [`my-idea`](https://github.com/juv4uk/my-idea) й була виділена сюди, щоб існувати самостійно. Майбутні реалізації тієї ж мови — C-ядро для embedded-цілей і власне HDL-ядро Lisp-машини з нуля — плануються як окремі, паралельні репозиторії.

## Deutsch

`my-lisp` ist ein Lisp, aufgebaut um McCarthys sieben Primitive — `quote`, `atom`, `eq`, `car`, `cdr`, `cons`, `cond` — plus den minimalen semantischen Kern, der zum Bootstrap von allem anderen nötig ist: `lambda`, `def`, `defmacro`. Alles, was aus diesem Kern ableitbar ist, ist in my-lisp selbst geschrieben ([`lib/core.my`](lib/core.my)), nicht als Rust-Built-in hinzugefügt. Vollständige Begründung: [`docs/language-core.md`](docs/language-core.md).

Exakte rationale Arithmetik ist ein Kernziel, kein Extra: `/` bleibt bei Ganzzahlen/rationalen Zahlen exakt (`5/336`, nicht `0.0148...`), nach Rackets exakt/inexakt-Unterscheidung. `.my` ist die kanonische Quellcodedateiendung; `.lisp` bleibt ein kompatibler Alias.

### Philosophie

Rust stellt nur das bereit, was es besonders gut kann — sichere Werte, Parsing, lexikalische Closures, deterministische Auswertung, Stack-Kontrolle, Diagnosen — und hört dort auf. Jede abgeleitete Form, jede Stdlib-Funktion wächst in my-lisp selbst statt in der Rust-Oberfläche, sobald der vorhandene Kern sie bereits ausdrücken kann: `<=`/`>=` waren eine Zwei-Zeilen-Ergänzung einer bestehenden Dispatch-Tabelle, keine neue Maschinerie; `eval` nutzte dieselbe Daten→Code-Umwandlung wieder, die die Makro-Expansion bereits brauchte, statt sie zu duplizieren. Es ist derselbe Instinkt, der John McCarthy davon abhielt, M-expressions fertigzustellen, als sich S-expressions als ausreichend erwiesen.

Dieser Instinkt ist geprüft, nicht nur behauptet: [`lib/meta-eval.my`](lib/meta-eval.my) ist ein metazirkulärer Evaluator — `eval`/`apply`, geschrieben in my-lisp selbst, dieselbe Beziehung, die McCarthys eigenes Paper von 1960 zu seinen Primitiven hatte — dispatcht an `car`/`cdr`/`cons`/`atom`/`eq`, statt sie neu zu implementieren. [`lib/unify.my`](lib/unify.my) ist eine kleine Unifikations-Engine im Geiste von McCarthys "Advice Taker"-Vorschlag von 1958 — symbolischer Mustervergleich, keine Statistik. Beide sind der Beweis, dass "eine kleine Sprache, die sich selbst wachsen lässt" eine arbeitende Aussage über diese Codebasis ist, kein Slogan.

[`tests/fixtures/conformance.json`](tests/fixtures/conformance.json) existiert, weil Lisps Geschichte auch eine Geschichte auseinanderdriftender Dialekte ist — MacLisp, InterLisp, ein Dutzend Scheme-Varianten, Common Lisps Versuch, sie wiederzuvereinen. Diese Datei ist das eine, dem ein künftiger C-Kern oder HDL-Lisp-Maschinen-Kern entsprechen muss, damit eine zweite Implementierung nie "nur ein weiterer Dialekt" wird.

Dieses Repository ist die kanonische Rust-Implementierung:

- [`crates/my-lisp`](crates/my-lisp) — der Kern: Parser, Evaluator, Umgebungen, exakte rationale Arithmetik.
- [`crates/my-lisp-cli`](crates/my-lisp-cli) — die `my-lisp`-Binärdatei (REPL + Dateiausführung).
- [`crates/my-lisp-wasm`](crates/my-lisp-wasm) — WebAssembly-Bindings für den Browser-REPL oben.
- [`crates/my-lisp-literate`](crates/my-lisp-literate) — Offset-Zuordnung von literate-Markdown-Quellcode.
- [`lib/core.my`](lib/core.my) — die gebootstrappte Standardbibliothek.

### Bauen und testen

```bash
cargo build --workspace
cargo test --workspace
```

### Dokumentation

- [`docs/language-core.md`](docs/language-core.md) — der Sprachvertrag: Primitive, Bootstrap-Grenze, exakte Arithmetik.
- [`docs/quote-tutorial.md`](docs/quote-tutorial.md) — ein Einsteiger-Rundgang durch Homoikonizität.
- [`docs/unify-tutorial.md`](docs/unify-tutorial.md) — ein kleines symbolisches KI-Beispiel: Unifikation, geschrieben in my-lisp selbst.
- [`docs/testing.md`](docs/testing.md) — aktuelles Testinventar.
- [`docs/benchmarks.md`](docs/benchmarks.md) — Benchmark-Methodik und eine lokale Ausgangsmessung.
- [`docs/versioning.md`](docs/versioning.md) — warum die Versionshistorie dieses Repos so aussieht.

`my-lisp` begann innerhalb eines größeren IDE-Projekts, [`my-idea`](https://github.com/juv4uk/my-idea), und wurde hierher ausgelagert, um eigenständig zu bestehen. Künftige Implementierungen derselben Sprache — ein C-Kern für Embedded-Ziele und ein von Grund auf neuer HDL-Kern für eine Lisp-Maschine — sind als separate, parallele Repositories geplant.

## License · Ліцензія · Lizenz

[MIT](LICENSE)

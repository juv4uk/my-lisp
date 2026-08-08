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

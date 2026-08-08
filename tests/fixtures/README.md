# conformance.json — the implementation-independent contract · Незалежний від реалізації контракт · Der implementierungsunabhängige Vertrag

## English

This directory holds `conformance.json`: the executable specification of my-lisp's *observable behavior* — what a program evaluates to, and what fails and how. It's runnable today by `crates/my-lisp/tests/mccarthy.rs`'s `conformance_tests_from_json`, and it's meant to outlive this one Rust implementation. Per [`PLAN.md`](../../PLAN.md)'s confirmed future direction, `my-lisp` is meant to eventually get a C core (for embedded targets) and a from-scratch HDL core (a Lisp-machine FPGA design). Both, when they exist, should be checked against this same file — not a re-derived, independently-drifting copy of it. A behavioral difference between implementations is a bug in one of them, discovered here, not discovered by a user years later.

### Format

An array of fixture objects. Two shapes:

**Success fixture** — `{ "expr": "...", "expected": "..." }`. `expr` is my-lisp source; `expected` is the exact string `Display`-formatting of the resulting value would produce (`t` for true, `()` for nil/false, `(a b c)` for lists, `5/336` for an unreduced-looking-but-actually-reduced rational, etc — see the `Display` impl for `Value` in `crates/my-lisp/src/value.rs` for the exact grammar). String comparison is intentional: an implementation only needs to match *observable printed output*, not any particular internal representation.

**Error fixture** — `{ "expr": "...", "error": "..." }`. `expr` must fail to evaluate; `error` is one of the five `ErrorKind` variant names — `Parse`, `UnknownSymbol`, `Arity`, `Type`, `InvalidForm` (see `docs/language-core.md`'s error-kind documentation and `crates/my-lisp/src/error.rs`). This checks the *kind* of failure, not the message text — error message wording is allowed to differ across implementations (and across the three languages this project's messages are written in); the *category* of failure is the contract.

**Optional `"mode": "markdown"`** on either shape routes the fixture through `crates/my-lisp-literate`'s literate-Markdown extraction instead of evaluating `expr` directly as source — for testing the offset-remapping behavior documented in `docs/language-core.md`. A future non-Rust implementation that doesn't have a literate-Markdown layer can skip these fixtures; they're testing that *layer*, not the core language.

### Rules an implementation must follow to be conformant

- Fixtures run **in one shared session**, in array order, with `lib/core.my` preloaded first — not each in an isolated environment. A fixture may rely on a `def` from an earlier fixture being visible (none currently do, deliberately, to keep fixtures easy to reorder — but nothing stops a future fixture from depending on this, so preserve the ordering and sharing behavior).
- `expected`/`error` values, once published, are never edited or removed — same immutability promise this project already makes for release tags (see `docs/versioning.md`). If a fixture turns out to encode a bug, add a *new* fixture with the corrected behavior and a comment explaining why the old one still exists, rather than silently changing what "conformant" means retroactively.
- New fixtures are always welcome, especially ones that exercise a primitive's error path, not just its success path — the error-kind fixtures in this file exist because, before they were added, this file only checked "does the right answer come out," never "does the right *kind of wrongness* come out."

## Українська

Ця тека містить `conformance.json`: виконувану специфікацію *спостережуваної поведінки* my-lisp — у що обчислюється програма, і що падає та як. Сьогодні його запускає `conformance_tests_from_json` у `crates/my-lisp/tests/mccarthy.rs`, і задумано його так, щоб він пережив цю одну Rust-реалізацію. За підтвердженим майбутнім напрямом з `PLAN.md`, my-lisp з часом має отримати C-ядро (для embedded-цілей) і HDL-ядро з нуля (FPGA-дизайн Lisp-машини). Обидва, коли з'являться, мають звірятись з цим самим файлом — не з окремо виведеною, незалежно розбіжною копією. Поведінкова розбіжність між реалізаціями — це баг в одній з них, знайдений тут, а не через роки користувачем.

### Формат

Масив об'єктів-фікстур. Дві форми:

**Фікстура успіху** — `{ "expr": "...", "expected": "..." }`. `expr` — код my-lisp; `expected` — точний рядок, який видасть `Display`-форматування результату (`t` для істини, `()` для nil/хиби, `(a b c)` для списків, `5/336` для скороченого раціонального — точну граматику див. у `Display`-реалізації `Value` в `crates/my-lisp/src/value.rs`). Порівняння рядків навмисне: реалізація має збігатись лише за *спостережуваним друкованим виводом*, не за якимось конкретним внутрішнім представленням.

**Фікстура помилки** — `{ "expr": "...", "error": "..." }`. `expr` має провалитись при обчисленні; `error` — одна з п'яти назв варіантів `ErrorKind` — `Parse`, `UnknownSymbol`, `Arity`, `Type`, `InvalidForm` (див. документацію видів помилок у `docs/language-core.md` і `crates/my-lisp/src/error.rs`). Це перевіряє *вид* провалу, не текст повідомлення — формулювання помилки може відрізнятись між реалізаціями (і між трьома мовами, якими написані повідомлення цього проєкту); контракт — саме *категорія* провалу.

**Опційний `"mode": "markdown"`** на будь-якій формі спрямовує фікстуру через literate-Markdown-екстракцію `crates/my-lisp-literate` замість прямого обчислення `expr` як коду — для перевірки поведінки ремапінгу зміщень, задокументованої в `docs/language-core.md`. Майбутня реалізація не на Rust, що не має шару literate-Markdown, може пропустити ці фікстури; вони перевіряють саме *цей шар*, не ядро мови.

### Правила, яких має дотримуватись реалізація для конформності

- Фікстури виконуються **в одній спільній сесії**, у порядку масиву, з попередньо завантаженим `lib/core.my` — не кожна в ізольованому середовищі. Фікстура може покладатись на `def` з попередньої фікстури як видимий (жодна зараз так не робить, навмисно, щоб фікстури лишались легко переставними — але ніщо не заважає майбутній фікстурі покладатись на це, тож зберігати порядок і спільність сесії).
- Значення `expected`/`error`, щойно опубліковані, ніколи не редагуються й не видаляються — та сама обіцянка незмінності, яку проєкт вже дає релізним тегам (див. `docs/versioning.md`). Якщо фікстура виявляється закодованим багом — додати *нову* фікстуру з виправленою поведінкою й коментар, чому стара досі тут, а не тихо міняти, що означає "конформний", заднім числом.
- Нові фікстури завжди вітаються, особливо ті, що перевіряють шлях помилки примітиву, не лише успішний шлях — фікстури видів помилок у цьому файлі існують саме тому, що до їх додавання файл перевіряв лише "чи виходить правильна відповідь", ніколи "чи виходить правильний *вид неправильності*".

## Deutsch

Dieses Verzeichnis enthält `conformance.json`: die ausführbare Spezifikation des *beobachtbaren Verhaltens* von my-lisp — wozu ein Programm auswertet, und was fehlschlägt und wie. Es wird heute von `conformance_tests_from_json` in `crates/my-lisp/tests/mccarthy.rs` ausgeführt und ist dafür gedacht, diese eine Rust-Implementierung zu überdauern. Laut der bestätigten künftigen Richtung in `PLAN.md` soll my-lisp irgendwann einen C-Kern (für Embedded-Ziele) und einen von Grund auf neuen HDL-Kern (ein FPGA-Design einer Lisp-Maschine) erhalten. Beide sollten, sobald sie existieren, gegen genau diese Datei geprüft werden — nicht gegen eine separat abgeleitete, unabhängig abdriftende Kopie davon. Ein Verhaltensunterschied zwischen Implementierungen ist ein Bug in einer davon, hier entdeckt, nicht Jahre später von einem Nutzer.

### Format

Ein Array von Fixture-Objekten. Zwei Formen:

**Erfolgs-Fixture** — `{ "expr": "...", "expected": "..." }`. `expr` ist my-lisp-Quellcode; `expected` ist genau der String, den die `Display`-Formatierung des resultierenden Werts erzeugen würde (`t` für wahr, `()` für nil/falsch, `(a b c)` für Listen, `5/336` für ein gekürztes Rational usw. — die genaue Grammatik siehe die `Display`-Implementierung von `Value` in `crates/my-lisp/src/value.rs`). Der String-Vergleich ist beabsichtigt: eine Implementierung muss nur der *beobachtbaren gedruckten Ausgabe* entsprechen, keiner bestimmten internen Repräsentation.

**Fehler-Fixture** — `{ "expr": "...", "error": "..." }`. `expr` muss bei der Auswertung fehlschlagen; `error` ist einer der fünf `ErrorKind`-Variantennamen — `Parse`, `UnknownSymbol`, `Arity`, `Type`, `InvalidForm` (siehe die Fehlerarten-Dokumentation in `docs/language-core.md` und `crates/my-lisp/src/error.rs`). Dies prüft die *Art* des Fehlschlags, nicht den Nachrichtentext — der Wortlaut der Fehlermeldung darf zwischen Implementierungen abweichen (und zwischen den drei Sprachen, in denen die Meldungen dieses Projekts geschrieben sind); die *Kategorie* des Fehlschlags ist der Vertrag.

**Optionales `"mode": "markdown"`** bei beiden Formen leitet die Fixture durch die literate-Markdown-Extraktion von `crates/my-lisp-literate`, statt `expr` direkt als Quellcode auszuwerten — zur Prüfung des in `docs/language-core.md` dokumentierten Offset-Remapping-Verhaltens. Eine künftige Nicht-Rust-Implementierung ohne literate-Markdown-Schicht kann diese Fixtures überspringen; sie prüfen genau diese *Schicht*, nicht den Sprachkern.

### Regeln, die eine Implementierung für Konformität einhalten muss

- Fixtures laufen **in einer gemeinsamen Sitzung**, in Array-Reihenfolge, mit vorab geladenem `lib/core.my` — nicht jede in einer isolierten Umgebung. Eine Fixture darf sich darauf verlassen, dass ein `def` aus einer früheren Fixture sichtbar ist (derzeit tut das keine, bewusst, damit Fixtures leicht umsortierbar bleiben — aber nichts hindert eine künftige Fixture daran, sich darauf zu verlassen, daher Reihenfolge und gemeinsame Sitzung beibehalten).
- `expected`/`error`-Werte werden, einmal veröffentlicht, nie bearbeitet oder entfernt — dasselbe Unveränderlichkeitsversprechen, das dieses Projekt bereits für Release-Tags gibt (siehe `docs/versioning.md`). Stellt sich heraus, dass eine Fixture einen Bug kodiert, eine *neue* Fixture mit dem korrigierten Verhalten hinzufügen und einen Kommentar, warum die alte weiterhin existiert, statt rückwirkend still zu ändern, was "konform" bedeutet.
- Neue Fixtures sind immer willkommen, besonders solche, die den Fehlerpfad eines Primitivs prüfen, nicht nur seinen Erfolgspfad — die Fehlerart-Fixtures in dieser Datei existieren genau deshalb, weil diese Datei vor ihrer Hinzufügung nur prüfte, "kommt die richtige Antwort heraus", nie "kommt die richtige *Art von Falschheit* heraus".

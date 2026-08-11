# Immutable World · Незмінний світ · Unveränderliche Welt

## English

`lib/world.my` is the first executable slice of the architecture `Expression × World → Value × World`. A world is ordinary S-expression data:

```lisp
(world parent knowledge-journal metadata)
```

`empty-world` creates a root. `world-tell` and `world-retract` return a new world whose `world-parent` is the previous value. They never alter the input world. The newest journal is one `cons` cell whose tail is the complete previous journal, so history uses structural sharing rather than copying every event. `world-clauses` projects a module at any retained version, including a version whose facts were later retracted.

This layer deliberately contains no Rust primitive. Its event representation is byte-for-byte compatible with the existing `(tell module clause)` / `(retract module clause)` journal. Once `world.my` is loaded, legacy `defmodule`, `tell-knowledge`, and `retract-knowledge` become thin wrappers over `world-tell-all`/`world-retract`: they extract the returned journal so existing readers retain their exact behavior. `tell-knowledge` still rejects conflicts before transitioning; retraction remains unconditional.

`reason-in-world` and `forward-in-world` are the first consumers of that explicit state. Backward and forward reasoning now operate on a selected snapshot or branch without reading the global `*knowledge-journal*`; the same query may therefore have a different, reproducible answer in two worlds.

`advise-world` makes guarded ingestion pure as well. It always returns `(decision world)`: accepted knowledge carries a new world, while malformed or conflicting input carries the exact original world. It reuses `lib/knowledge.my`'s clause validation and conflict vocabulary but checks only the supplied snapshot, never global knowledge.

`advise-all-world` applies the same contract atomically to a non-empty clause batch. Proposed facts and rules are validated and conflict-checked together, so they may support one another; success creates exactly one child world, while any malformed clause or internal/derived conflict leaves the original world intact with no partial prefix.

`make-world-knowledge-package` exports one module from one selected snapshot using the existing versioned `my-lisp-knowledge` data envelope. `import-knowledge-package-world` validates that envelope and delegates to `advise-all-world`: valid data seeds an independent child branch; malformed, unsupported, or conflicting data returns the exact target world unchanged. Package data is inspected, never evaluated.

History is navigable without a clock: `world-depth` counts transitions from the root, `world-at-depth` recovers an exact ancestor snapshot, and `world-diff from to` returns chronological journal events when `from` is an ancestor of `to`. Sibling branches return `World-not-ancestor`; merge semantics are not guessed.

`world-common-ancestor` aligns unequal depths and finds the semantic branch point. `world-branch-diff` returns `(base ...)`, `(left events)`, and `(right events)`, giving a future merge policy explicit evidence without choosing conflict or retraction precedence today. Independently reconstructed but structurally equal worlds correctly have no delta.

Content identity is canonical before it is cryptographic. `knowledge-content-address` is exact read-back-safe canonical text; `world-content-address` covers the complete journal and metadata without recursively duplicating parents. Equal reconstructed histories share an address, while equal current clauses reached through different histories do not. A future digest may hash this address without changing identity semantics.

## Українська

`lib/world.my` — перший виконуваний зріз архітектури `Expression × World → Value × World`. Світ є звичайними S-expression-даними:

```lisp
(world батько журнал-знань метадані)
```

`empty-world` створює корінь. `world-tell` і `world-retract` повертають новий світ, де `world-parent` — попереднє значення, і ніколи не змінюють вхідний світ. Новий журнал додає лише одну cons-комірку, хвостом якої є весь попередній журнал: історія використовує structural sharing, а не копіювання всіх подій. `world-clauses` проєктує модуль у будь-якій збереженій версії, включно з версією, факти якої пізніше відкликали.

Шар навмисно не додає Rust-примітивів. Формат подій точно сумісний із чинним журналом `(tell модуль clause)` / `(retract модуль clause)`. Після завантаження `world.my` старі `defmodule`, `tell-knowledge` і `retract-knowledge` стають тонкими обгортками над `world-tell-all`/`world-retract`: вони дістають журнал із поверненого світу, тому чинні читачі зберігають точну поведінку. `tell-knowledge` досі відхиляє конфлікт до переходу; retract лишається безумовним.

`reason-in-world` і `forward-in-world` — перші споживачі цього явного стану. Backward- і forward-reasoning тепер працюють з обраним snapshot чи гілкою, не читаючи глобальний `*knowledge-journal*`; тому одна ціль може мати різну, відтворювану відповідь у двох світах.

`advise-world` так само робить чистим захищене надходження знань. Він завжди повертає `(рішення світ)`: прийняте знання несе новий світ, а malformed чи конфліктний ввід — точно початковий. Функція перевикористовує validation і словник конфліктів із `lib/knowledge.my`, але перевіряє лише переданий snapshot, ніколи глобальне знання.

`advise-all-world` атомарно застосовує той самий контракт до непорожнього пакета clause. Запропоновані факти й правила перевіряються разом і можуть підтримувати одне одного; успіх створює рівно один дочірній світ, а malformed clause чи внутрішній/вивідний конфлікт лишає початковий світ цілим без часткового префікса.

`make-world-knowledge-package` експортує один модуль з обраного snapshot у чинній версіонованій data-оболонці `my-lisp-knowledge`. `import-knowledge-package-world` валідовує її та делегує `advise-all-world`: коректні дані породжують незалежну дочірню гілку; malformed, unsupported чи конфліктні повертають точний цільовий світ. Дані пакета ніколи не виконуються.

Історією можна навігувати без годинника: `world-depth` рахує переходи від кореня, `world-at-depth` повертає точний ancestor snapshot, а `world-diff from to` — хронологічні події, коли `from` є предком `to`. Sibling-гілки повертають `World-not-ancestor`; merge-семантика не вигадується.

`world-common-ancestor` вирівнює різні глибини й знаходить семантичну точку розгалуження. `world-branch-diff` повертає `(base ...)`, `(left події)` і `(right події)`, даючи майбутній merge-policy явні докази без передчасного вибору пріоритету конфліктів чи retract. Незалежно реконструйовані, але структурно рівні світи правильно мають нульову дельту.

Content-ідентичність канонічна раніше, ніж криптографічна. `knowledge-content-address` — точний read-back-safe текст; `world-content-address` охоплює журнал і metadata без рекурсивного дублювання parents. Рівні реконструйовані історії мають одну адресу, однакові поточні clauses із різними історіями — різні. Майбутній digest може хешувати адресу, не змінюючи семантику identity.

## Deutsch

`lib/world.my` ist der erste ausführbare Ausschnitt der Architektur `Expression × World → Value × World`. Eine Welt besteht aus gewöhnlichen S-Expression-Daten:

```lisp
(world vorgänger wissenjournal metadaten)
```

`empty-world` erzeugt die Wurzel. `world-tell` und `world-retract` liefern eine neue Welt, deren `world-parent` der vorige Wert ist, und verändern niemals ihre Eingabe. Das neue Journal ergänzt nur eine Cons-Zelle, deren Ende das vollständige vorige Journal ist: Geschichte nutzt strukturelle Teilung statt alle Ereignisse zu kopieren. `world-clauses` projiziert ein Modul in jeder erhaltenen Version, auch wenn seine Fakten später zurückgenommen wurden.

Diese Schicht fügt bewusst kein Rust-Primitiv hinzu. Ihr Ereignisformat ist mit dem bestehenden Journal `(tell modul clause)` / `(retract modul clause)` vollständig kompatibel. Nach dem Laden von `world.my` werden `defmodule`, `tell-knowledge` und `retract-knowledge` zu dünnen Hüllen um `world-tell-all`/`world-retract`: Sie entnehmen der gelieferten Welt das Journal, sodass bestehende Leser ihr exaktes Verhalten behalten. `tell-knowledge` weist Konflikte weiterhin vor dem Übergang ab; Rücknahme bleibt bedingungslos.

`reason-in-world` und `forward-in-world` sind die ersten Verbraucher dieses expliziten Zustands. Rückwärts- und Vorwärtsschluss arbeiten nun auf einem gewählten Schnappschuss oder Zweig, ohne das globale `*knowledge-journal*` zu lesen; dieselbe Anfrage kann deshalb in zwei Welten eine unterschiedliche, reproduzierbare Antwort haben.

`advise-world` macht auch die geschützte Wissensaufnahme rein. Es liefert immer `(entscheidung welt)`: Akzeptiertes Wissen enthält eine neue Welt, ungültige oder widersprüchliche Eingabe exakt die ursprüngliche. Es nutzt Validierung und Konfliktvokabular aus `lib/knowledge.my`, prüft aber nur den übergebenen Schnappschuss, niemals globales Wissen.

`advise-all-world` wendet denselben Vertrag atomar auf einen nichtleeren Clause-Stapel an. Vorgeschlagene Fakten und Regeln werden gemeinsam geprüft und dürfen einander stützen; Erfolg erzeugt genau eine Kindwelt, während eine ungültige Clause oder ein interner/abgeleiteter Konflikt die ursprüngliche Welt ohne Teilpräfix bewahrt.

`make-world-knowledge-package` exportiert ein Modul aus einem gewählten Schnappschuss in der bestehenden versionierten Datenhülle `my-lisp-knowledge`. `import-knowledge-package-world` prüft sie und delegiert an `advise-all-world`: gültige Daten erzeugen einen unabhängigen Kindzweig; ungültige, nicht unterstützte oder widersprüchliche Daten geben die exakte Zielwelt zurück. Paketdaten werden niemals evaluiert.

Geschichte ist ohne Uhr navigierbar: `world-depth` zählt Übergänge ab der Wurzel, `world-at-depth` findet einen exakten Vorgängerschnappschuss und `world-diff from to` liefert chronologische Ereignisse, wenn `from` Vorfahr von `to` ist. Geschwisterzweige liefern `World-not-ancestor`; Merge-Semantik wird nicht erraten.

`world-common-ancestor` gleicht unterschiedliche Tiefen aus und findet den semantischen Verzweigungspunkt. `world-branch-diff` liefert `(base ...)`, `(left ereignisse)` und `(right ereignisse)` als Belege für eine spätere Merge-Policy, ohne heute Konflikt- oder Rücknahmevorrang zu erfinden. Unabhängig rekonstruierte, strukturell gleiche Welten haben korrekt kein Delta.

Inhaltsidentität ist kanonisch, bevor sie kryptographisch ist. `knowledge-content-address` ist exakter rücklesbarer Text; `world-content-address` umfasst Journal und Metadaten ohne rekursive Eltern-Duplikation. Gleiche rekonstruierte Geschichten teilen eine Adresse, gleiche aktuelle Clauses aus verschiedenen Geschichten nicht. Ein späterer Digest darf die Adresse hashen, ohne Identität neu zu definieren.

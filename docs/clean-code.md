# Clean Code in my-lisp · Clean Code у my-lisp · Clean Code in my-lisp

## English

`my-lisp` should not enforce one personal coding style. It should make readable,
local, explicit, composable, testable, and explainable code the easiest code to
write. Clean Code is therefore a language-and-tooling direction, not a set of
mandatory line-count rules.

The semantic foundation comes first:

- values are immutable by default;
- state is passed explicitly through `World` in the foundational API;
- global APIs remain convenient compatibility wrappers, not a second semantics;
- effects stay at the capability boundary and should be recognizable by name;
- one S-expression model represents programs, data, knowledge, and proofs;
- macros evaluate ordinary arguments exactly once unless documented otherwise;
- laws such as `read(write(x)) = x` matter more than isolated examples.

Names should communicate intent. Predicates end in `?`; directional conversions
use `->`; namespaces may add context; vague names such as `process`, `handle`, or
`data` should produce educational hints rather than errors. Public definitions
should be distinguishable from implementation helpers.

Composition should be easier to read than deep nesting. A future threading macro
may express transformation pipelines, while a conservative formatter preserves
program structure. A linter should report size, nesting, complexity, hidden
global dependencies, effects, duplication, naming, and missing documentation as
configurable hints or warnings—not as a punitive quality score.

Documentation belongs beside code. First-class docstrings, `doc`, `source`, and
`macroexpand` should make the REPL a teacher. Important functions should carry
executable examples, explicit preconditions, and lightweight contracts. Errors
should identify kind, span, function, expected and received values, and useful
context without inventing a heavy static type system first.

Tooling follows the semantic foundation: formatter and linter, then contracts
and property/law tests, then IDE views such as call hierarchy, AST/data flow, and
purity/effect hints. Structural duplicate detection and `explain-code` come only
after those foundations. AI assistance is optional and last; idiomatic my-lisp
must encourage good code without AI.

This document deliberately does not mandate automatic World merge, mutation,
hard complexity limits, mandatory namespaces, or a large feature checklist.

## Українська

`my-lisp` не повинна нав’язувати один персональний стиль. Вона має зробити
читабельний, локальний, явний, композиційний, тестований і пояснюваний код
найпростішим способом писати програму. Clean Code тут — напрям семантики й
інструментів, а не набір обов’язкових лімітів рядків.

Спочатку семантичний фундамент:

- значення immutable за замовчуванням;
- фундаментальний API передає стан явно через `World`;
- global API лишається convenience/compatibility layer, не другою семантикою;
- effects живуть на capability-межі й мають бути впізнаваними з імені;
- одна модель S-expression представляє програми, дані, знання й докази;
- макроси обчислюють звичайні аргументи рівно раз, якщо явно не сказано інше;
- закони на кшталт `read(write(x)) = x` важливіші за окремі приклади.

Імена мають передавати намір. Предикати завершуються на `?`, напрямлені
перетворення використовують `->`, namespaces можуть додавати контекст, а нечіткі
`process`/`handle`/`data` мають давати навчальну підказку, не помилку. Public API
слід відрізнити від implementation helpers.

Композиція має читатися легше за глибоке вкладення. Майбутній threading macro
може показувати pipeline перетворень, а консервативний formatter — зберігати
структуру програми. Linter має повідомляти про розмір, nesting, complexity,
приховані globals, effects, duplication, naming і документацію як налаштовувані
hints/warnings, а не як каральний quality score.

Документація живе біля коду. First-class docstrings, `doc`, `source` і
`macroexpand` мають зробити REPL учителем. Важливі функції отримують executable
examples, явні preconditions і легкі contracts. Помилки мають називати kind,
span, function, expected/received та контекст, не вимагаючи спочатку важкої
static type system.

Tooling іде після фундаменту: formatter і linter, потім contracts та property/law
tests, тоді IDE call hierarchy, AST/data-flow і purity/effect hints. Structural
duplicate detection та `explain-code` — лише після цього. AI-помічник останній і
необов’язковий: idiomatic my-lisp має вести до хорошого коду без AI.

Документ навмисно не вимагає автоматичного World merge, мутації, жорстких
complexity limits, обов’язкових namespaces чи великого checklist можливостей.

## Deutsch

`my-lisp` soll keinen persönlichen Stil erzwingen. Lesbarer, lokaler,
expliziter, komponierbarer, testbarer und erklärbarer Code soll vielmehr der
einfachste Weg sein. Clean Code ist hier eine Richtung für Semantik und
Werkzeuge, kein Satz zwingender Zeilenlimits.

Das semantische Fundament kommt zuerst:

- Werte sind standardmäßig unveränderlich;
- die grundlegende API führt Zustand ausdrücklich als `World`;
- globale APIs bleiben Komfort-/Kompatibilitätshüllen statt zweiter Semantik;
- Effekte bleiben an der Capability-Grenze und sollen am Namen erkennbar sein;
- S-Ausdrücke stellen Programme, Daten, Wissen und Beweise gemeinsam dar;
- Makros werten normale Argumente genau einmal aus, sofern nicht anders erklärt;
- Gesetze wie `read(write(x)) = x` sind wichtiger als einzelne Beispiele.

Namen vermitteln Absicht: Prädikate enden in `?`, gerichtete Umwandlungen nutzen
`->`, Namespaces dürfen Kontext ergänzen. Vage Namen sollen lehrreiche Hinweise,
keine Fehler erzeugen. Öffentliche Definitionen sind von Hilfsdefinitionen zu
unterscheiden.

Komposition soll leichter lesbar sein als tiefe Verschachtelung. Ein späteres
Threading-Makro kann Pipelines ausdrücken; ein konservativer Formatter bewahrt
die Programmstruktur. Ein Linter meldet Größe, Verschachtelung, Komplexität,
versteckte globale Abhängigkeiten, Effekte, Duplikation, Benennung und fehlende
Dokumentation als konfigurierbare Hinweise statt als Strafwertung.

Dokumentation gehört zum Code. First-Class-Docstrings, `doc`, `source` und
`macroexpand` machen den REPL zum Lehrer. Wichtige Funktionen erhalten
ausführbare Beispiele, Vorbedingungen und leichte Verträge. Diagnosen nennen
Art, Span, Funktion, erwarteten/erhaltenen Wert und Kontext, ohne zuerst ein
schweres statisches Typsystem einzuführen.

Werkzeuge folgen dem Fundament: Formatter und Linter, danach Verträge und
Property-/Gesetzestests, anschließend IDE-Ansichten. Strukturelle Duplikatsuche,
`explain-code` und optionale KI-Hilfe kommen zuletzt. Automatischer World-Merge,
Mutation, harte Komplexitätslimits, Pflicht-Namespaces und Feature-Checklisten
sind ausdrücklich kein Mandat.

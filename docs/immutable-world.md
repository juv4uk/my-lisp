# Immutable World · Незмінний світ · Unveränderliche Welt

## English

`lib/world.my` is the first executable slice of the architecture `Expression × World → Value × World`. A world is ordinary S-expression data:

```lisp
(world parent knowledge-journal metadata)
```

`empty-world` creates a root. `world-tell` and `world-retract` return a new world whose `world-parent` is the previous value. They never alter the input world. The newest journal is one `cons` cell whose tail is the complete previous journal, so history uses structural sharing rather than copying every event. `world-clauses` projects a module at any retained version, including a version whose facts were later retracted.

This layer deliberately contains no Rust primitive and does not replace `lib/knowledge.my` yet. Its event representation is byte-for-byte compatible with the existing `(tell module clause)` / `(retract module clause)` journal. The next migration can therefore make knowledge operations accept and return an explicit world without changing packages, rules, or proofs.

`reason-in-world` and `forward-in-world` are the first consumers of that explicit state. Backward and forward reasoning now operate on a selected snapshot or branch without reading the global `*knowledge-journal*`; the same query may therefore have a different, reproducible answer in two worlds.

## Українська

`lib/world.my` — перший виконуваний зріз архітектури `Expression × World → Value × World`. Світ є звичайними S-expression-даними:

```lisp
(world батько журнал-знань метадані)
```

`empty-world` створює корінь. `world-tell` і `world-retract` повертають новий світ, де `world-parent` — попереднє значення, і ніколи не змінюють вхідний світ. Новий журнал додає лише одну cons-комірку, хвостом якої є весь попередній журнал: історія використовує structural sharing, а не копіювання всіх подій. `world-clauses` проєктує модуль у будь-якій збереженій версії, включно з версією, факти якої пізніше відкликали.

Шар навмисно не додає Rust-примітивів і поки не замінює `lib/knowledge.my`. Формат подій точно сумісний із чинним журналом `(tell модуль clause)` / `(retract модуль clause)`. Тому наступний перенос зможе зробити knowledge-операції явними функціями «світ на вході → світ на виході», не змінюючи пакети, правила чи докази.

`reason-in-world` і `forward-in-world` — перші споживачі цього явного стану. Backward- і forward-reasoning тепер працюють з обраним snapshot чи гілкою, не читаючи глобальний `*knowledge-journal*`; тому одна ціль може мати різну, відтворювану відповідь у двох світах.

## Deutsch

`lib/world.my` ist der erste ausführbare Ausschnitt der Architektur `Expression × World → Value × World`. Eine Welt besteht aus gewöhnlichen S-Expression-Daten:

```lisp
(world vorgänger wissenjournal metadaten)
```

`empty-world` erzeugt die Wurzel. `world-tell` und `world-retract` liefern eine neue Welt, deren `world-parent` der vorige Wert ist, und verändern niemals ihre Eingabe. Das neue Journal ergänzt nur eine Cons-Zelle, deren Ende das vollständige vorige Journal ist: Geschichte nutzt strukturelle Teilung statt alle Ereignisse zu kopieren. `world-clauses` projiziert ein Modul in jeder erhaltenen Version, auch wenn seine Fakten später zurückgenommen wurden.

Diese Schicht fügt bewusst kein Rust-Primitiv hinzu und ersetzt `lib/knowledge.my` noch nicht. Ihr Ereignisformat ist mit dem bestehenden Journal `(tell modul clause)` / `(retract modul clause)` vollständig kompatibel. Der nächste Migrationsschritt kann Wissensoperationen daher zu expliziten Funktionen „Welt hinein → Welt hinaus“ machen, ohne Pakete, Regeln oder Beweise zu ändern.

`reason-in-world` und `forward-in-world` sind die ersten Verbraucher dieses expliziten Zustands. Rückwärts- und Vorwärtsschluss arbeiten nun auf einem gewählten Schnappschuss oder Zweig, ohne das globale `*knowledge-journal*` zu lesen; dieselbe Anfrage kann deshalb in zwei Welten eine unterschiedliche, reproduzierbare Antwort haben.

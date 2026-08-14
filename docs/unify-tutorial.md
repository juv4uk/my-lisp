# Unification: matching with variables · Унікація: зіставлення зі змінними · Unifikation: Abgleich mit Variablen

## English

McCarthy's 1958 "Advice Taker" proposal imagined a system that reasons over symbolic facts — matching a question against what it knows, discovering what has to be true for the match to work. [`lib/unify.my`](../lib/unify.my) is that matching engine, [`unification`](https://en.wikipedia.org/wiki/Unification_(computer_science)), written entirely in my-lisp itself — the same building block backward-chaining/Prolog-style inference is built from.

### The pieces

- **`(logic-var 'x)`** creates a logic variable — internally `(var x)`, a two-element list, not a bare symbol (`eq` only accepts atoms, and my-lisp has no `symbol?` to test a naming convention like a leading `?`).
- **`(unify a b subst)`** tries to make `a` and `b` structurally identical by extending the substitution `subst` (an alist of variable-bindings) with new bindings, or returns the atom `fail` if no such bindings can exist. Start with `subst` as `'()` — the empty substitution.
- **`(apply-subst term subst)`** walks `term`, replacing every bound variable with what it resolved to — including through chains (`?x` bound to `?y` bound to `alice` resolves all the way to `alice`) — so you get a readable answer instead of raw substitution internals.
- **`(failed-subst? subst)`** — `subst` is either `fail` or a real substitution; this tells you which, without risking `eq` on a non-atom (see the caveat below).

### A worked example: family facts

```lisp
(def facts
  (list
    (list 'parent 'alice 'bob)
    (list 'parent 'bob 'carol)
    (list 'parent 'alice 'diana)))
```

Three facts, no rules yet — just data. Now ask "who are Alice's children?" by unifying a *pattern with a variable in it* against each fact, keeping only the ones that match:

```lisp
(def matches
  (filter
    (lambda (f) (not (failed-subst? (unify (list 'parent 'alice (logic-var 'x)) f '()))))
    facts))

(map (lambda (f) (apply-subst (logic-var 'x) (unify (list 'parent 'alice (logic-var 'x)) f '()))) matches)
```
Prints `(bob diana)`. Nothing in `unify.my` knows anything about family relationships — `unify` only knows how to make two trees of symbols line up, and `?x` was free to become whatever made that possible. The "reasoning" is entirely in how the pattern was shaped.

### One caveat, stated plainly

There's no *occurs-check*: unifying `?x` with a term that contains `?x` (like `(f ?x)`) produces a substitution that can't be fully resolved — an infinite structure. This is the standard simplification every small/teaching unifier makes; a real occurs-check just means walking the term for the variable before binding it, left out here to keep this genuinely small. See [`lib/unify.my`](../lib/unify.my)'s own header comment for the full account, including the two `eq`-on-non-atom bugs this design caught during hand-testing.

## Українська

Пропозиція "Advice Taker" Маккарті 1958 року уявляла систему, що міркує над символьними фактами — зіставляє питання з тим, що знає, і виявляє, що має бути істинним, щоб зіставлення спрацювало. [`lib/unify.my`](../lib/unify.my) — саме цей механізм зіставлення, [унікація (unification)](https://en.wikipedia.org/wiki/Unification_(computer_science)), написана повністю самою my-lisp — та сама цеглинка, з якої будується backward-chaining/Prolog-подібне висновування.

### Складові

- **`(logic-var 'x)`** створює логічну змінну — внутрішньо `(var x)`, список із двох елементів, не голий символ (`eq` приймає лише атоми, а в my-lisp немає `symbol?`, щоб перевірити угоду іменування на кшталт провідного `?`).
- **`(unify a b subst)`** намагається зробити `a` і `b` структурно ідентичними, розширюючи підстановку `subst` (asoc-список зв'язків змінних) новими зв'язками, або повертає атом `fail`, якщо таких зв'язків існувати не може. Починати з `subst` як `'()` — порожньою підстановкою.
- **`(apply-subst term subst)`** обходить `term`, замінюючи кожну зв'язану змінну на те, у що вона розв'язалась — включно через ланцюжки (`?x`, зв'язаний з `?y`, зв'язаним з `alice`, розв'язується аж до `alice`) — тож ти отримуєш читабельну відповідь, а не сирі внутрішні деталі підстановки.
- **`(failed-subst? subst)`** — `subst` є або `fail`, або справжньою підстановкою; це каже, яке саме, без ризику `eq` на не-атомі (див. застереження нижче).

### Робочий приклад: сімейні факти

```lisp
(def facts
  (list
    (list 'parent 'alice 'bob)
    (list 'parent 'bob 'carol)
    (list 'parent 'alice 'diana)))
```

Три факти, поки без правил — просто дані. Тепер спитай "хто діти Аліси?", унікуючи *шаблон зі змінною* проти кожного факту, лишаючи тільки ті, що збіглись:

```lisp
(def matches
  (filter
    (lambda (f) (not (failed-subst? (unify (list 'parent 'alice (logic-var 'x)) f '()))))
    facts))

(map (lambda (f) (apply-subst (logic-var 'x) (unify (list 'parent 'alice (logic-var 'x)) f '()))) matches)
```
Друкує `(bob diana)`. Нічого в `unify.my` не знає про сімейні відносини — `unify` вміє лише вирівнювати два дерева символів, а `?x` був вільний стати чим завгодно, що робило це можливим. "Міркування" повністю в тому, як був сформований шаблон.

### Одне застереження, прямо

Немає *occurs-check*: унікація `?x` з термом, що містить `?x` (як `(f ?x)`), дає підстановку, яку неможливо повністю розв'язати — нескінченну структуру. Це стандартне спрощення, яке робить кожен маленький/навчальний unifier; справжній occurs-check означає лише обхід терма на предмет змінної перед зв'язуванням, тут пропущений, щоб лишатись справді маленьким. Повний опис, включно з двома `eq`-на-не-атомі багами, які цей дизайн зловив під час ручного тестування — у власному header-коментарі [`lib/unify.my`](../lib/unify.my).

## Deutsch

McCarthys "Advice Taker"-Vorschlag von 1958 stellte sich ein System vor, das über symbolische Fakten schließt — eine Frage mit dem abgleicht, was es weiß, und entdeckt, was wahr sein muss, damit der Abgleich funktioniert. [`lib/unify.my`](../lib/unify.my) ist genau diese Abgleich-Engine, [Unifikation](https://en.wikipedia.org/wiki/Unification_(computer_science)), vollständig in my-lisp selbst geschrieben — derselbe Baustein, aus dem Backward-Chaining-/Prolog-artige Inferenz aufgebaut wird.

### Die Bausteine

- **`(logic-var 'x)`** erzeugt eine Logikvariable — intern `(var x)`, eine zweielementige Liste, kein bloßes Symbol (`eq` akzeptiert nur Atome, und my-lisp hat kein `symbol?`, um eine Namenskonvention wie ein führendes `?` zu prüfen).
- **`(unify a b subst)`** versucht, `a` und `b` strukturell identisch zu machen, indem die Substitution `subst` (eine Alist von Variablenbindungen) um neue Bindungen erweitert wird, oder gibt das Atom `fail` zurück, wenn solche Bindungen nicht existieren können. Start mit `subst` als `'()` — der leeren Substitution.
- **`(apply-subst term subst)`** durchläuft `term` und ersetzt jede gebundene Variable durch das, wozu sie aufgelöst wurde — auch über Ketten hinweg (`?x`, gebunden an `?y`, gebunden an `alice`, löst bis zu `alice` auf) — sodass eine lesbare Antwort entsteht statt roher Substitutions-Interna.
- **`(failed-subst? subst)`** — `subst` ist entweder `fail` oder eine echte Substitution; dies sagt, welches von beiden, ohne `eq` auf einem Nicht-Atom zu riskieren (siehe die Einschränkung unten).

### Ein durchgerechnetes Beispiel: Familienfakten

```lisp
(def facts
  (list
    (list 'parent 'alice 'bob)
    (list 'parent 'bob 'carol)
    (list 'parent 'alice 'diana)))
```

Drei Fakten, noch keine Regeln — nur Daten. Nun die Frage "wer sind Alices Kinder?", indem ein *Muster mit einer Variable darin* gegen jeden Fakt unifiziert wird, wobei nur die Treffer behalten werden:

```lisp
(def matches
  (filter
    (lambda (f) (not (failed-subst? (unify (list 'parent 'alice (logic-var 'x)) f '()))))
    facts))

(map (lambda (f) (apply-subst (logic-var 'x) (unify (list 'parent 'alice (logic-var 'x)) f '()))) matches)
```
Gibt `(bob diana)` aus. Nichts in `unify.my` weiß etwas über Familienbeziehungen — `unify` weiß nur, wie man zwei Symbolbäume zur Deckung bringt, und `?x` war frei, zu werden, was auch immer das ermöglichte. Das "Schließen" liegt vollständig darin, wie das Muster geformt wurde.

### Eine Einschränkung, klar benannt

Es gibt keinen *Occurs-Check*: Das Unifizieren von `?x` mit einem Term, der `?x` enthält (wie `(f ?x)`), erzeugt eine Substitution, die sich nicht vollständig auflösen lässt — eine unendliche Struktur. Dies ist die Standardvereinfachung, die jeder kleine/lehrende Unifier macht; ein echter Occurs-Check bedeutet nur, den Term vor dem Binden nach der Variable zu durchsuchen, hier weggelassen, um wirklich klein zu bleiben. Die vollständige Darstellung, einschließlich der zwei `eq`-auf-Nicht-Atom-Bugs, die dieses Design beim manuellen Testen fing, steht im eigenen Header-Kommentar von [`lib/unify.my`](../lib/unify.my).

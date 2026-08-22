# Guarded advice ingestion · Захищене приймання порад · Geschützte Wissensaufnahme

## English

`advise` is the data-only write boundary between an untrusted translator and
the symbolic knowledge journal. It accepts exactly one `lib/reason.my` clause,
validates the complete structure (including canonical `(var name)` variables),
checks for an explicitly known opposite, and mutates the journal only on
acceptance. It never treats failure to prove a statement as proof of its
negation.

```lisp
(advise astronomy (understand '(earth is a planet)))
(advise astronomy (understand '(all planet have mass)))

(def goal '(has earth mass))
(def proof (second (car (reason-in 'astronomy goal))))
(narrate-answer goal proof)
; => (earth has mass because earth is a planet)
```

Results are stable data shapes rather than printed messages:

- `(accepted (module name) (knowledge clause))`
- `(rejected (reason invalid-module|invalid-clause) (input value))`
- `(conflict (new clause) (existing opposite) (proof result))`

Explicit negative knowledge uses a head such as `((not (planet pluto)))`.
This is distinct from `(not goal)` inside a rule body, where the reasoning
engine implements negation as failure.

For translator output containing several clauses, use `advise-all`. It
validates the complete non-empty batch against both the current module and
the proposed clauses, including contradictions derived through proposed
rules. The journal receives every clause in one update or receives none:

```lisp
(advise-all astronomy
  '(((planet earth))
    ((has-mass (var x)) (planet (var x)))))
```

## Українська

`advise` — data-only межа запису між недовіреним перекладачем і символьним
журналом знань. Вона приймає рівно один clause у форматі `lib/reason.my`,
перевіряє всю структуру (включно з канонічними змінними `(var name)`), шукає
явно відому протилежність і змінює журнал лише після прийняття. Неможливість
довести твердження ніколи не вважається доказом його заперечення.

Наскрізний приклад вище проходить шлях `understand → advise → reason-in →
narrate-answer` і повертає пояснену відповідь. Результати `accepted`,
`rejected` та `conflict` є стабільними структурами даних, а не текстом,
прив'язаним до інтерфейсу.

Явне негативне знання має голову на кшталт `((not (planet pluto)))`. Це не
те саме, що `(not goal)` у тілі правила, де reasoning-рушій використовує
negation as failure.

Для результату перекладача з кількох clause слід використовувати
`advise-all`. Вона атомарно перевіряє весь непорожній пакет разом із чинним
модулем, включно із суперечностями, виведеними запропонованими правилами:
журнал отримує або всі clause одним оновленням, або жодної.

## Deutsch

`advise` ist die reine Datengrenze zwischen einem nicht vertrauenswürdigen
Übersetzer und dem symbolischen Wissensjournal. Sie nimmt genau eine Clause
im Format von `lib/reason.my` entgegen, prüft die gesamte Struktur
(einschließlich kanonischer `(var name)`-Variablen), sucht einen explizit
bekannten Gegensatz und verändert das Journal nur nach Annahme. Ein nicht
beweisbarer Satz gilt niemals als Beweis seiner Verneinung.

Das durchgängige Beispiel oben führt über `understand → advise → reason-in →
narrate-answer` zu einer erklärten Antwort. Die Ergebnisse `accepted`,
`rejected` und `conflict` sind stabile Datenstrukturen, keine an eine
Oberfläche gebundenen Meldungen.

Explizit negatives Wissen verwendet einen Kopf wie `((not (planet pluto)))`.
Das unterscheidet sich von `(not goal)` in einem Regelrumpf, wo die
Inferenz-Engine Negation als Fehlschlag verwendet.

Für Übersetzerausgaben mit mehreren Clauses dient `advise-all`. Sie prüft das
gesamte nichtleere Paket atomar zusammen mit dem vorhandenen Modul,
einschließlich durch vorgeschlagene Regeln abgeleiteter Widersprüche. Das
Journal erhält entweder alle Clauses in einer Aktualisierung oder keine.

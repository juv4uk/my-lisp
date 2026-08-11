# Knowledge package 0.1 · Пакет знань 0.1 · Wissenspaket 0.1

## English

The cross-project interchange format is one data-only S-expression:

```lisp
((format . my-lisp-knowledge)
 (version 0 1)
 (module . astronomy)
 (clauses . (((planet earth))
             ((has-mass (var x)) (planet (var x))))))
```

`format`, `version`, `module`, and `clauses` are required. Version `0.1` means
the envelope may still change incompatibly before `1.0`; consumers must reject
unknown versions rather than guess. Clauses use the existing `lib/reason.my`
shape and canonical `(var name)` variables. The file is read as data and must
never be loaded as executable source.

`import-knowledge-package` and `import-knowledge-file` validate the envelope,
then delegate to the same atomic acceptance policy as `advise-all`. Either all
clauses enter the journal or none do. `make-knowledge-package` constructs the
canonical in-memory shape for adapters that produce data directly.
`write-knowledge-package` performs the other direction: it validates a module
and clause batch, serializes the canonical envelope, and writes one expression.

For process-to-process transport, `send-knowledge-package` and
`receive-knowledge-package` use one package per TCP connection, with EOF as the
unambiguous frame boundary. Receivers drain all chunks before `read`; received
data is validated and imported, never evaluated.

## Українська

Міжпроєктний формат — один data-only S-вираз із обов'язковими полями `format`,
`version`, `module` і `clauses`. Версія `0.1` означає, що до `1.0` оболонка ще
може несумісно змінюватися; невідому версію треба відхиляти, а не вгадувати.
Clause використовують чинний формат `lib/reason.my` і канонічні змінні
`(var name)`. Файл читається як дані й ніколи не завантажується як код.

`import-knowledge-package` та `import-knowledge-file` перевіряють оболонку і
передають її тій самій атомарній політиці, що й `advise-all`: журнал отримує
або всі clause, або жодної. `make-knowledge-package` створює канонічну форму в
пам'яті для адаптерів інших проєктів.
`write-knowledge-package` виконує зворотний напрям: перевіряє пакет і записує
канонічну оболонку одним S-виразом.
Для обміну між процесами `send-knowledge-package`/`receive-knowledge-package`
передають один пакет на TCP-з'єднання з EOF як однозначною межею. Дані
читаються, перевіряються та імпортуються, але ніколи не виконуються.

## Deutsch

Das projektübergreifende Format ist ein einziger reiner Daten-S-Ausdruck mit
den Pflichtfeldern `format`, `version`, `module` und `clauses`. Version `0.1`
bedeutet, dass sich die Hülle vor `1.0` noch inkompatibel ändern kann;
unbekannte Versionen müssen abgelehnt statt erraten werden. Clauses verwenden
die bestehende `lib/reason.my`-Form und kanonische `(var name)`-Variablen. Die
Datei wird als Daten gelesen und niemals als ausführbarer Quelltext geladen.

`import-knowledge-package` und `import-knowledge-file` prüfen die Hülle und
verwenden danach dieselbe atomare Annahmepolitik wie `advise-all`: entweder
gelangen alle Clauses ins Journal oder keine. `make-knowledge-package` erzeugt
die kanonische Speicherform für Adapter anderer Projekte.
`write-knowledge-package` übernimmt die Gegenrichtung: prüfen und die
kanonische Hülle als einen S-Ausdruck schreiben.
Für Prozesse übertragen `send-knowledge-package`/`receive-knowledge-package`
genau ein Paket pro TCP-Verbindung mit EOF als eindeutiger Grenze. Empfangene
Daten werden gelesen, geprüft und importiert, niemals ausgeführt.

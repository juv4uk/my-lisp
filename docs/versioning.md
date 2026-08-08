# Versioning and inherited history · Версіонування та успадкована історія · Versionierung und übernommene Historie

## English

This repository was extracted from [`my-idea`](https://github.com/juv4uk/my-idea) via `git filter-repo`, keeping only the files that belong to the my-lisp language (see [`PLAN.md`](../PLAN.md) for the extraction plan). The extraction preserves per-file commit history for everything that stayed, so the log still shows how the language evolved inside the IDE project before the split — including commits and tags whose messages describe IDE-only work. They are historical provenance, not a statement that IDE code is part of this repository; the actual IDE files are gone.

Release tags from before the split are immutable and were not deleted; do not reuse an occupied version number. `crates/my-lisp` and its sibling crates version independently starting at `0.1.0`, regardless of what tag numbers exist in the inherited history.

Release tags in this repository use the prefix `l` (for example `l0.9.2`), not the inherited `v` prefix — a deliberate choice, so a new tag can never collide with one of the pre-split `v0.1.0`–`v0.9.0` tags still sitting in the inherited history. `.github/workflows/release.yml` triggers on `l*`, not `v*`.

## Українська

Цей репозиторій виділено з [`my-idea`](https://github.com/juv4uk/my-idea) через `git filter-repo`, залишивши лише файли, що належать мові my-lisp (план виділення — [`PLAN.md`](../PLAN.md)). Виділення зберігає покомітну історію для всього, що лишилось, тож лог і далі показує, як мова розвивалась усередині IDE-проєкту до розділення — включно з комітами й тегами, чиї повідомлення описують суто IDE-роботу. Це історичне походження, а не твердження, що код IDE є частиною цього репозиторію; фактичні файли IDE відсутні.

Релізні теги з часів до розділення незмінні й не видалялись; не використовувати повторно зайнятий номер версії. `crates/my-lisp` та сусідні крейти версіонуються незалежно, починаючи з `0.1.0`, незалежно від того, які номери тегів є в успадкованій історії.

Релізні теги в цьому репозиторії використовують префікс `l` (наприклад `l0.9.2`), а не успадкований префікс `v` — свідомий вибір, щоб новий тег ніколи не колізив зі старими тегами `v0.1.0`–`v0.9.0`, що досі є в успадкованій історії. `.github/workflows/release.yml` тригериться на `l*`, не на `v*`.

## Deutsch

Dieses Repository wurde per `git filter-repo` aus [`my-idea`](https://github.com/juv4uk/my-idea) extrahiert, wobei nur die zur my-lisp-Sprache gehörenden Dateien erhalten blieben (Extraktionsplan: [`PLAN.md`](../PLAN.md)). Die Extraktion bewahrt die dateibezogene Commit-Historie für alles, was blieb; das Log zeigt daher weiterhin, wie sich die Sprache innerhalb des IDE-Projekts vor der Trennung entwickelte — einschließlich Commits und Tags, deren Nachrichten reine IDE-Arbeit beschreiben. Dies ist historische Herkunft, keine Aussage, dass IDE-Code Teil dieses Repositories ist; die tatsächlichen IDE-Dateien sind nicht mehr vorhanden.

Release-Tags von vor der Trennung sind unveränderlich und wurden nicht gelöscht; eine bereits belegte Versionsnummer wird nicht wiederverwendet. `crates/my-lisp` und die Schwester-Crates versionieren unabhängig, beginnend bei `0.1.0`, unabhängig davon, welche Tag-Nummern in der übernommenen Historie existieren.

Release-Tags in diesem Repository verwenden das Präfix `l` (zum Beispiel `l0.9.2`), nicht das übernommene Präfix `v` — eine bewusste Wahl, damit ein neues Tag niemals mit einem der `v0.1.0`–`v0.9.0`-Tags aus der Zeit vor der Trennung kollidiert, die weiterhin in der übernommenen Historie stehen. `.github/workflows/release.yml` löst bei `l*` aus, nicht bei `v*`.

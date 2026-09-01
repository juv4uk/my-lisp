# DOC-SURVEY-2026-09-01 — огляд нових доків my-lisp

**Виконавець:** wsl-nidana-1 (ecosystem-координаційна сесія)
**Метод:** `git log --since=2026-08-28 --name-only --diff-filter=A -- docs/`
+ `git log --since=2026-08-28 --stat -- docs/*.md` для суттєво змінених
файлів. Виключено вже раніше розібране: lsp-m0.md, wsm-source-vs-protocol-
extension-policy.md, PROPOSAL-INVIOLABLE-PRIMITIVES.md (CANON),
mccarthy-1960-eval-apply-walkthrough, DIALECT-COMPARISON.md,
guard-reference curation.

## Відкрите архітектурне рішення (ще не ratified)

- **`docs/host-capability-scoping-adr-2026-08-27.md`** — PROPOSED: розширює
  наявний `process_allowlist`-патерн на `fs_read_roots`/`fs_write_roots`/
  `tcp_connect_policy`/`tcp_listen_policy` у `Environment::Limits`. Зараз —
  all-or-nothing доступ до файлів/мережі, щойно встановлено host-шар.
  Backward-compatible дизайн (`Option<...>`, `None` = поточна необмежена
  поведінка). Три відкриті питання власника всередині документа (окремі
  read/write root-списки? symlink-escape policy? явний "без обмежень" прапор?).

## Вже прийняте рішення (звужує scope вище)

- Коміт **2026-08-29** (`feat(runtime): enable native Lisp-machine process
  execution`, торкається `docs/capabilities.md`): власник ратифікував
  **необмежений `process-run`** для довіреного "native Lisp-machine profile"
  (`Environment::root()` після `my-lisp-host::install()`) — локальні програми
  більше не потребують per-executable allowlist. TCP/oracle-сесії без
  автентифікації й далі deny-all без явного `--allow-process=...`.

## Нові доки

- **`docs/DATE-TIME-AND-SYNC-ARCHITECTURE-2026-08-31.md`** — мапує
  часові інтерфейси (`mono-ms`/`mono-ns`/`utc-now`/`internet-time-sync`/
  `timezone-detect`/`timezone-config`), фіксує 5 явних не-еквівалентностей
  (mono-ns≠дата, wall-clock≠логічна ревізія, назва TZ≠фіксований офсет,
  NTP≠автентифікована істина, timestamp≠content identity). WSM-FS ніколи не
  використовує wall-clock для ідентичності/порядку — лише root/journal докази.

- **`docs/guard-oracle-node-plan.md`** (найбільш churned, 6 ревізій) —
  уніфікує Oracle/Guard/swarm-node: `Oracle = очікувана семантична істина`,
  `Observer/Rust = спостережені факти`, `Guard/WSM = пояснює розрив`,
  `swarm-node = лише координація, не семантичний авторитет`. Додає
  `oracle-eval` (versioned `oracle-result/1`) і агентський `oracle-check`
  (parse-only, structured error з byte-span/line-col/suggested-edit, без
  silent auto-apply). Формалізує канонічний swarm-node startup і розділяє
  `join` (membership) від `agent-send register` (transport) — не плутати.

- **`docs/wsm-backronym-proposals.md`** — 7 кандидатів на розшифровку "WSM"
  (включно з санскритським "Vyākaraṇa Symbolic Model"). Брейнштормінг, не
  рішення — ratified-варіанту не знайдено.

## Не деталізовано (implementation-companion, той самий клас)

`docs/UTC-DATETIME-NANOSECOND-CONTRACT-2026-08-31.md`,
`docs/reason-scale-profile-2026-08-29.md`, `knowledge/guard-runtime-policy.wsm`.

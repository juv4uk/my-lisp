## Дисципліна співпраці з агентами — основний документ (2026-09-03)

**Статус: основний (primary) для всіх активних репозиторіїв екосистеми.** Цей розділ визначає, як агенти працюють із власником над кодом — і йде першим, перед будь-яким іншим вмістом цього файлу.

### Головний зсув: не "агент пише за мене", а "агент будує експеримент, а я розбираю, як ідея стала кодом"

```text
ідея
  ↓
агент пропонує реалізацію
  ↓
власник читає код
  ↓
власник пояснює його своїми словами
  ↓
дивиться ту саму ідею в іншій мові/субстраті (де це доречно)
  ↓
порівнює представлення
  ↓
тільки потім наступний крок
```

Агенти в цій екосистемі — не "програмісти замість власника". Вони:

```text
дослідник
+
лаборант
+
співстудент
+
рецензент
```

Власник лишається тим, хто формує концепцію й поступово вчиться читати її фізичне втілення.

### Не приховувати складність за готовим кодом

Якщо агент пише функцію чи будь-який нетривіальний фрагмент, він має розкласти рішення до рівня причин, не лише показати результат:

```text
тип
↓
параметри
↓
calling convention / представлення в пам'яті
↓
allocation
↓
memory layout
↓
returned value
```

Наприклад, не просто:

```c
typedef uintptr_t Value;
```

і далі — а з поясненням: чому саме цей тип, чому не альтернатива, скільки це байтів на цільовій архітектурі, що гарантує відповідний заголовок/стандарт, як це виглядає на рівні регістра. Власник сам вирішує, наскільки глибоко копати сьогодні — але агент завжди пропонує цей рівень деталізації, не ховає його.

### Після кожного невеликого фрагмента коду — 3-5 питань на розуміння САМЕ цього коду

Не абстрактний тест із мови загалом, а конкретні питання про щойно написаний фрагмент. Приклад формату:

```text
Чому тут саме цей тип, а не інша очевидна альтернатива?

Що саме зберігається в цій змінній — значення чи адреса?

Що означає ця конкретна операція/маска/умова?

Яка інструкція процесора приблизно відповідає цьому коду?

Яка частина цього рішення належить мові/предметній області, а яка — конкретній реалізації/субстрату?
```

### Крос-субстратне порівняння (де застосовно — переважно `my-lisp` і суміжні репозиторії мови)

Коли та сама ідея існує в кількох реалізаціях (наприклад, `my-lisp`: Rust, C, x86 asm, Guile, FPGA), корисний формат порівняння:

```text
1. LANGUAGE FACT       — що стверджує сама мова?
2. RUST REPRESENTATION — як це представлено зараз?
3. C REPRESENTATION    — як це можна представити в C?
4. ASM VIEW            — у що це реально перетворюється на цільовій архітектурі?
5. GUILE VIEW          — як та сама ідея виглядає на високому символьному рівні?
6. HARDWARE VIEW       — що з цього реально існує як біти, адреси, операції?
7. WHAT IS ESSENTIAL   — що належить мові, а що належить субстрату?
```

Мета — щоб після знайомства з однією ідеєю (наприклад, `cons`/pair) власник бачив не лише "що це працює", а що саме лишається незмінним у самій ідеї, а що є лише способом її представити на конкретному фізичному чи мовному субстраті. Це не обов'язковий ритуал для кожного репозиторію — застосовується там, де справді є кілька субстратів/реалізацій тієї самої ідеї для порівняння.

### Резюме принципу

Мета — не "вивчити мову X", а малими вертикальними зрізами повністю зрозуміти, як одна конкретна ідея проходить від задуму до фізичного втілення (біта в регістрі, гейта на кремнії, вузла в дереві коду). Генерувати можна багато — засвоювати варто малими, повністю зрозумілими кроками.

---
# AGENTS.md — my-lisp

See also `docs/agent-doctrine.md` — cross-repo rules (prose vs. contract
precedence, evidence discipline, subagent/specialist-model usage) that
apply to every sibling in this swarm, not just this repo.

## Guard як довідкове бюро / Guard as a reference bureau

Перед пошуком навмання або створенням нового workflow завантажте
`lib/guard.wsm` і `knowledge/guard-reference.wsm`. Запитайте
`(guard-reference topic)`, `(guard-authority topic)`,
`(guard-how-to topic)` або `(guard-verify topic)`. Каталог указує, де лежить
авторитетна інформація; він не копіює і не замінює її. Невідома тема повертає
`UNKNOWN/UNRESOLVED`, після чого потрібен перевірений новий запис, а не здогад.

Before searching blindly or inventing a workflow, load `lib/guard.wsm` and
`knowledge/guard-reference.wsm`. Ask `(guard-reference topic)`,
`(guard-authority topic)`, `(guard-how-to topic)`, or `(guard-verify topic)`.
The directory points to authoritative information; it does not copy or
replace it. An unknown topic returns `UNKNOWN/UNRESOLVED`, requiring a
reviewed new entry rather than a guess. `UNKNOWN` then exposes three explicit
routes: ask a responsible agent for ecosystem-local knowledge, ask the owner
for authority/intent/license/scope decisions, or research authoritative web
sources for external or time-sensitive facts.

## Session start — join the swarm

**Coordination protocol (superseded 2026-08-12, drift found and fixed
2026-08-18 — verify this section against `docs/swarm-mesh-v2.md` before
trusting it, prose is not authoritative):** coordination moved off the
single `:9999` server onto `swarm-node`, a P2P journal/claim mesh — one
process per agent, TCP `:910x`, no single point of failure, no
restart-wipes-everything problem the old model had. See
`docs/swarm-mesh-v2.md` for the full design, wire protocol, onboarding
checklist, and remote-deployment playbook. In short:

1. Start (or connect to an already-running) `swarm-node --port 910x
   --node-id <your-id> --project my-lisp --data-dir ~/.swarm-node/<your-id>
   --connect <a-known-peer>:9101` — see the doc's "Onboarding checklist".
2. `(join (capabilities (...)))` to register, `(list-members)` to see
   who else is live, `(next-best-action (capabilities (...)))` to see
   what's actionable.
3. `(claim-task (task ...))` → do the work → `(complete-task (task ...)
   (generation N))` → `(emit (type ...) (payload ...))` for anything
   worth other agents knowing durably (not just a chat message).

The `my-lisp --tcp=9999 --protocol=sexpr` process is a **separate,
still-live thing**: the semantic oracle (`eval`/`parse`/`diagnose`
against this repo's interpreter), unrelated to coordination now. Use it
if you need to evaluate my-lisp code remotely; don't confuse it with
the swarm-node coordination plane above — see `docs/swarm-mesh-v2.md`'s
"Two planes" section for exactly this distinction.

**Restart etiquette:** a `swarm-node` restart only affects that one
agent's presence/in-flight claims (durable journal state survives via
`--data-dir` and anti-entropy sync from peers on reconnect) — this is
the whole reason for the 2026-08-12 migration away from the
single-server model, where every restart wiped `claim`/`presence`/the
task registry for everyone at once.

## Role

Semantic source of truth for the four-repository ecosystem (`my-lisp`,
`fpga-lisp`, `cml`, `my-idea`). Defines what a my-lisp program means; every
other repository must match this, not the reverse.

A fifth and sixth sibling, `my-lisp-panini` and `shiva-sutras`, research Pāṇinian Sanskrit grammar as
a formal system feeding this repo's `SANSKRIT-P*` semantic-atom migration
(`docs/sanskrit-semantic-migration.md`) — but they are not part of the
match-the-contract relationship above; they do not touch `my-lisp` at all
until their own machine-model gate reviews pass (see their `AGENTS.md` files).

## Authoritative files

- `language-contract.my` — the versioned semantic contract. **Read its
  `(major . N) (minor . N)` cons directly** rather than trusting a
  number written in prose anywhere (including this file) — prose drifts,
  the file is the contract.
- `docs/language-core-axioms.md` — the G1–G8/S1–S3 axioms the contract
  covers, with the reasoning behind each.
- `tests/fixtures/conformance.my` — the fixture set every claim of
  conformance (from any repo) is checked against, tagged by axiom.
- `ecosystem-status.my` — a curated snapshot pointer across all four repos,
  not itself authoritative for any one repo's details (see `evidence/`).

## How to run tests

```
cargo +stable-x86_64-pc-windows-msvc test --workspace
```
(GNU toolchain is flaky on this machine when the shared rustup default
toolchain changes — use the MSVC toolchain explicitly.)

## What not to change without a contract bump

- Any axiom in `docs/language-core-axioms.md`, or `language-contract.my`'s
  version number, without deliberate discussion — other repos pin against
  this version.
- `tests/fixtures/conformance.my` entries are append-only historical
  facts; don't edit an existing fixture's `expr`/`expected`, add a new one.

## How to create evidence

See `evidence/README.md` for the format. One file per
`(requirement-id, implementation, commit)` at
`evidence/<id>/<implementation>/<short-sha>.my`. A durable claim ("X now
passes/fails") gets an evidence file or a contract edit — not a status
message.

**The `notify`/`poll`/`claim`/`presence`/`define-task`/`capability-request`
mailbox described below runs on `:9999` alongside the semantic oracle —
it predates the swarm-node migration above and its live status hasn't
been re-verified since. Treat `swarm-node` (`docs/swarm-mesh-v2.md`) as
authoritative for claims/tasks/presence; if you use anything below,
confirm it still responds before depending on it.**

## How to check neighboring repositories

Read `fpga-lisp/isa-contract.my`, `cml/compatibility.my`, and each
neighbor's own `evidence/` directory directly rather than asking. Use
`my-lisp --tcp=9999 --protocol=sexpr` (loopback-only, one thread per
connection) for three distinct things:

- `eval`/`parse`/`diagnose`/`contract-version` — the semantic oracle,
  each connection its own isolated `Environment` (a `def` on one
  connection is invisible to every other, and now also physically a
  separate thread, not just a separate value).
- `notify`/`poll` — a lightweight, poll-based cross-agent mailbox
  (owner decision, 2026-08-12), one server-wide in-memory list, capped
  at 500 entries (oldest-first drain), gone on server restart. `notify`
  takes `from`, optional `to` (omit for broadcast), `message`; `poll`
  takes `for` and optional `since` (a mailbox entry id, default 0),
  returns every entry addressed to `for` or broadcast with `id` greater
  than `since`. Use this for "check when convenient." **This is a
  first-class pattern, not a fallback** — an agent that's one tool call
  per turn with no memory between calls (no background process to hold
  a `subscribe` socket open) genuinely cannot use push; `notify`/`poll`
  plus calling `next-best-action`/`presence` on demand each turn is the
  complete, correct way for such an agent to participate. Don't design
  toward `subscribe` as the only channel.
- `subscribe`/`publish` — genuine push, not polling, for agents that
  *do* have a long-lived process to hold the connection (owner decision,
  2026-08-12). `subscribe` takes `topics` (a list; empty or omitted
  means every topic) and optional `since` (an event id, default 0) —
  replays every matching event logged after `since` before switching
  to live delivery, so a reconnecting agent that remembers the last
  event id it saw (each `(event (id N) ...)` carries one) doesn't miss
  what happened while its connection was down. Then permanently turns
  the connection into a receiver: it blocks and writes each matching
  `(event (id ..) (from ..) (topic ..) (message ..))` line the instant
  a `publish` happens elsewhere — open a second connection if you also
  need to `eval`/`notify`. The event log itself is capped at 500 (same
  as the mailbox) and, like everything else here, gone on server
  restart — `since` covers a subscriber's own reconnect, not the
  server going down. `publish` takes `from`, `topic`, `message`,
  responds with how many subscribers actually received it. Use this
  for "wake me up the moment X happens" (a handoff landing, an evidence
  file appearing, a peer getting blocked) instead of a `poll` loop.
  `claim`/`release`/`hello`/`define-task` (below) auto-`publish` on
  `claim-taken`/`claim-released`/`agent-joined`/`task-created` when
  they cause one — subscribe to those instead of polling `list-claims`/
  `presence` if you want to react the instant they change. Topics with
  no corresponding op (`evidence-created`, `handoff-created`,
  `contract-changed`, `dependency-satisfied`, `need-published`,
  `offer-published`) are convention only — `publish` them yourself at
  the moment they become true in your own repo's files.

- `claim`/`release`/`list-claims` — atomic task claiming (owner
  decision, 2026-08-12), for `next-best-action`-style self-organization:
  two agents racing for the same task can never both win. `claim` takes
  `task` and `from`; succeeds (`value` = `t`) if `task` is unclaimed or
  already held by `from`, otherwise returns the current holder's name so
  the loser knows who to wait on — unless the holder has gone quiet: if
  its `presence` heartbeat is older than 300s, the new `claim` succeeds
  as a reclaim instead (`claim-stale-reclaimed` published), so one agent
  going silent doesn't lock a task forever. A holder with no `presence`
  entry at all is *not* treated as stale (can't tell, don't steal).
  `release` takes the same fields; only
  the holder can release (others get the holder's name back, unchanged).
  `list-claims` takes no fields, returns every currently-held
  `((task . ..) (agent . ..))` pair. In-memory, non-persistent — a
  coordination hint about who's working on what *right now*, not the
  durable record of what got done (that's still `evidence/`).

- `hello`/`heartbeat`/`presence` — agent registry (owner decision,
  2026-08-12). `hello` takes `from`, optional `project`, optional
  `capabilities` (a list) — registers/refreshes the agent and returns
  the current peer list (excluding yourself). `heartbeat` takes `from`
  and optional `task` — refreshes liveness and current task, same
  peer-list response; no ordering requirement between `hello` and
  `heartbeat`, an agent that only ever heartbeats still shows up.
  `presence` (no fields) returns every registered agent's `project`,
  `capabilities`, `task`, and `seconds-since-heartbeat` — no automatic
  eviction, judge staleness yourself. In-memory, non-persistent.

- `define-task`/`complete-task`/`next-best-action` — self-organizing
  task scoring (owner decision, 2026-08-12). `define-task` takes `task`,
  optional `priority` (default 1.0), `capabilities`, `depends-on` (a
  list of other task ids), and optional `description` (prose — what the
  task actually is; preserved across a redefinition that omits it, same
  as `done`). `complete-task` takes `task`, marks it done and drops its
  claim — not restricted to the current holder. `next-best-action` takes
  `from` and optional `capabilities` (falls back to `presence`'s record
  of `from`'s last `hello` if omitted), returns every actionable task
  ranked by `priority × (1 + unblock-impact)` descending, each entry
  carrying its `description` (or `()` if none was ever set) so you don't
  have to cross-reference a file just to know what a ranked task id is
  — a task naming a capability the caller lacks, with an unsatisfied
  `depends-on`, already done, or already claimed by someone else is
  excluded outright, not merely down-ranked. `unblock-impact` is how
  many other not-yet-done tasks list this one in `depends-on`.
  `list-claims` also carries each claim's `description` the same way.
  In-memory, non-persistent.

- `sync-tasks`/`sync-milestone` — bridge durable files into the
  in-memory task registry, so `next-best-action` has something to score
  without every repo's own `define-task` calls re-typing what a
  git-tracked file already says. `sync-tasks` takes `file`, expects a
  `((tasks . (("id" . ((priority . N) (capabilities . (...))
  (depends-on . (...)) (done . t-or-nil))) ...)))` shape — upserts each
  listed task, preserving `done` unless the file overrides it, leaves
  tasks *not* listed alone. `sync-milestone` takes `file`, reads
  `ecosystem-status.my`'s own `next-milestone.per-repo` alist directly
  (no new file format) and defines one `MILESTONE:<name>:<repo>` task
  per entry at priority 5.0 with `capabilities (repo)` — the convention
  this creates is including your own repo name in `hello`'s
  `capabilities` so this surfaces specifically to you. Neither op
  reads a description back through `next-best-action` (that only
  returns task ids + scores) — the task-created event's `message`
  carries the prose once, at creation; otherwise read the source file.

- `capability-request` — temporary coalition formation (owner decision,
  2026-08-12). Takes `from`, optional `task`, `needs` (a capability
  name), optional `context`. Finds every `presence`-registered agent
  whose `capabilities` include `needs`, delivers the request to them
  both ways (`publish`ed on the `capability-request` topic for anyone
  `subscribe`d, and left in their `notify` mailbox regardless so a
  non-subscribed agent still sees it on the next `poll`), and
  auto-`define-task`s `HELP:<needs>:<task-or-from>` at priority 10.0
  requiring exactly `needs` — surfaces at the top of that agent's own
  `next-best-action` without a separate matching engine. Response
  reports `matching-agents` found and the `elevated-task` id.

**Every op above resets to empty on server restart** — restarting
after a deploy wipes `notify`'s mailbox, active `subscribe`s,
`claim`/`presence`/task state all at once. Don't treat any of it as a
place to relay durable content (a full proposal, a design doc): write
that to a file (`NOTE-*.md`, `docs/`) first, then send only a short
pointer through `notify`/`publish` — the pointer surviving a restart
costs nothing; the content wouldn't have.

- **`server-generation`** — every response (`ok` and `error` alike)
  carries `(server-generation N)`, the server process's start-time Unix
  timestamp (owner decision, 2026-08-12). For a stateless agent (one
  tool call per turn, no long-lived process to hold a `subscribe`
  connection or notice a dropped socket) this is the only way to
  self-detect "the server I last talked to is gone, this is a new
  one" — compare it against what you saw on your last call; a changed
  value means `claim`/`presence`/the task registry all reset, and
  it's on you to `sync-tasks`/`hello` again, not on someone else to
  tell you.

- `validate-tasks` — dry-run of `sync-tasks` (takes the same `file`
  field), never touches the task registry. A top-level parse error in
  the file reports 1-indexed `(line, column)`; a well-formed file still
  reports `would-define` + the same per-entry `warnings` `sync-tasks`
  would give, so a `tasks.my` mistake is visible without ever writing
  to shared state.

- **`file` must be absolute** on `sync-tasks`/`sync-milestone`/
  `validate-tasks` (owner decision, 2026-08-12, after a real incident):
  a relative path resolves against *this server process's* working
  directory, not the caller's, and used to silently read (and sync)
  whatever unrelated file happened to exist at that relative path on
  the server's side — no error, just a quiet wrong-file sync. Now
  rejected outright with an explanation.

- `list-tasks` — full, unfiltered dump of every `define-task`d task:
  `priority`, `capabilities`, `depends-on`, `done`, current
  `claimed-by` (or `()`), `description`. The debugging counterpart to
  `next-best-action`, which hides anything excluded by a capability
  mismatch, an unmet dependency, an existing claim, or `done` — without
  `list-tasks` there was no way to tell "this task doesn't exist" from
  "it exists but got filtered for a reason."

**On `done` tasks surviving a restart:** a task's `done: t` in your own
`tasks.my` is restored the instant you `sync-tasks` after a restart —
`next-best-action` already excludes anything `done`, so there's no need
to re-run `hello`/`claim`/`complete-task` for work that's already
proven; `sync-tasks` alone is sufficient. If that theater still felt
necessary, either the file's `done` field wasn't actually being set to
`t`, or it's worth filing as a real bug rather than working around by
habit — a `done` task never needing a claim in the first place is by
design.

**A limitation this protocol cannot fix, recorded from direct
feedback:** an agent whose harness gives it one tool call per turn with
no memory or background process between calls (as opposed to a
long-lived process that can hold a `subscribe` socket open) is
architecturally poll-only — no `publish` will ever wake such an agent's
session on its own; it only reacts when the owner or another process
prompts it to check `notify`/`poll`/`next-best-action`. If you need an
instant reaction *from* such an agent, don't rely on `subscribe`/
`publish` reaching it — `notify` it and expect to wait, the same way
you'd wait for any human-mediated step. This isn't a protocol gap to
close, just a real constraint some peers have that others don't.

All eleven ops classes share one process, but nothing `Rc`-based
(the language's own `Value`) ever crosses a thread boundary — only
plain `String`s move between connection threads, so this doesn't touch
`Value`'s single-threaded reference counting.

## Environment: WSL2 + Guix

Work in this repo from inside WSL2, under the Linux user named after this
repo (`my-lisp`), not directly from Windows. Enter the declared environment
before running anything:

```
wsl -u my-lisp
cd /mnt/c/GitHub/my-lisp
guix shell -m manifest.scm
```

`manifest.scm` pins the toolchain versions this repo expects; don't rely on
whatever happens to be on `$PATH` outside the shell.

## Live coordination context

A separate, parallel coordination effort (Codex as primary agent, OpenCode
as reviewer) runs through `C:\Users\user\Documents\GitHub\docs` — read
`docs/AGENT_MEMORY.md` there before assuming an area is untouched.

## Agent Guard (M0 — PROPOSED, 2026-08-22)

План executable-constitution guardrails для агентських сесій:
`/home/agents/ecosystem/plans/AGENT-GUARD-M0.md`

Машинні гачки на C1/C7/C9/C11 (ox-alpha constitution v1.2):
tool wrapper + evidence ledger + claim gate. Статус: план,
реалізація не почата. Агенти, що заходять у репо: прочитайте
план перед write-heavy роботою; зауваження — у plans/ або
власнику напряму.


## NLP / Embeddings tooling (2026-08-22)

Для NLP-задач (ембедінги, семантична класифікація, BGE-M3): системний
python3 НЕ має torch. Використовуй
`/home/agents/GitHub/FlagEmbedding/.venv/bin/python`.
Конфіг і готові індекси: `/home/agents/GitHub/vault-semantic-mcp/`
(корпусні ембедінги вже в `data/sanskrit_embeddings.jsonl` — перевикористовуй).
GPU лише 4GB — батчі ≤4, fp16, не перераховувати зайве.
Повний рецепт: `/home/agents/ecosystem/memory/nlp-tooling-setup.md`.

### NLP consumer (2026-08-23)
Результати семантичної класифікації корпусу вже готові — дивись без GPU:
`python3 /home/agents/GitHub/vault-semantic-mcp/lookup_concept.py anumāna`
(режими: концепт / --file / --search). Епістеміка: semantic-suggest =
гіпотеза, не authority. Повний рецепт: ecosystem/memory/nlp-tooling-setup.md.

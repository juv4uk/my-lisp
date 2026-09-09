# Аудит інваріантів swarm — 2026-09-09

Issue: #21  
База аудиту: `ccc91fd7821b899aa503831c0112914b285db87e`  
Робочий PR: #34

Цей аудит відповідає не на питання про кількість LOC чи `#[test]`, а на сильніше питання:

> Що саме має піти не так, щоб два агенти розійшлися у стані, втратили тривалі coordination-evidence або прийняли застаріле володіння — і який виконуваний доказ це ловить?

`crates/swarm-node/tests/integration.rs` — великий real-process TCP suite. Тому один файл містить багато різних distributed proofs; співвідношення «файли/тести» саме по собі нічого не доводить.

## Словник сили доказу

- **confirmed** — виконуваний тест прямо відтворює failure mode або проходить через потрібну process/network boundary і зараз проходить.
- **partial** — механізм і суміжні тести є, але конкретний failure mode не перевірений прямо end-to-end.
- **broken** — виконуваний експеримент або прямий аналіз durable-state path показує, що інваріант зараз хибний.
- **unknown** — достатнього виконуваного доказу не знайдено.

Flaky proof не підвищується до `confirmed` лише тому, що повторний запуск випадково пройшов.

## Матриця інваріантів

| Інваріант | Failure mode | Власник реалізації | Виконуваний доказ | Статус | Чого бракує / дія |
|---|---|---|---|---|---|
| Володіння node identity / заборона спільного data-dir | Два живі процеси змінюють один identity/journal або інший node-id повторно використовує наявний стан | `journal.rs`, startup/data-dir lock у `main.rs` | `startup_rejects_data_dir_owned_by_another_identity`; `shared_data_dir_rejects_second_live_process_on_another_port` | **confirmed** | Зберегти startup validation до будь-якої mutation. |
| Безпека duplicate port startup | Невдалий другий bind змінює durable identity або заважає живому власнику | startup path у `main.rs` | `duplicate_port_fails_before_mutating_identity` | **confirmed** | Прогалин не знайдено. |
| Durable journal append / звичайне restart recovery | ACK/state видно до durable append або restart скидає namespace/sequence | `journal.rs::append`, startup replay | `restart_preserves_incarnation_epoch_increments_seq_continues`; append викликає `sync_data()` до in-memory commit/broadcast | **confirmed** для звичайного restart | Power-loss semantics файлової системи поза межами loopback CI. |
| Пошкоджений/частковий authoritative `node.my` | Parse failure мовчки стає новим identity і переписує durable state | `journal.rs::load_or_init_identity` | PR #34: `corrupt_identity_is_rejected_without_rewriting_it`; `malformed_identity_fields_are_rejected_without_rewriting_them`; контрприклад legacy upgrade | **confirmed після fix у PR #34** | На базовому main було **broken** через `parse(...).unwrap_or(empty)`. |
| Пошкоджений/частковий authoritative `events.log` | Невалідний рядок мовчки пропускається, coordination evidence зникає при replay | `journal.rs::Journal::open` | PR #34: `journal_open_rejects_corrupt_line_without_truncating_history`; `journal_open_rejects_structurally_invalid_event_without_skipping_it` | **confirmed після fix у PR #34** | На базовому main було **broken**: невалідні рядки мовчки ігнорувались. |
| Anti-entropy convergence | Пізній peer назавжди не отримує вже наявні facts | sync hello/events у `main.rs`, `journal.rs` | `anti_entropy_sync_and_live_push_event`; `reincarnation_does_not_collide_and_anti_entropy_converges` | **confirmed** | Для звичайного reconnect/late join прогалин не знайдено. |
| Live event propagation | Під'єднаний peer збігається лише після явного resync | push-event/broadcast у `main.rs` | `anti_entropy_sync_and_live_push_event` | **confirmed** | Delivery failure перевіряється retry-тестами окремо. |
| Reconnect convergence | Перезапущений/reconnected peer не відновлює поточний replicated view | peer reconnect + anti-entropy | `failed_delivery_is_redelivered_after_peer_reconnects`; `p2p_presence_work_visibility_help_request_offer_and_reconnect` | **confirmed** | Справжній network partition→heal нижче — окрема властивість. |
| Peer discovery / mesh convergence | Вузол із одним bootstrap не дізнається/не під'єднується до решти mesh | peer gossip у `main.rs` | `gossip_peer_discovery_reaches_full_mesh` | **confirmed** | Для loopback mesh прогалин не знайдено. |
| Quorum claim fencing — послідовний сценарій | Після видимого commit можливий duplicate claim, wrong-generation completion або reclaim completed task | claim proposal/vote/state reducers | `quorum_claim_fencing_and_stale_rejection` | **confirmed** лише для sequential/post-propagation path | Не можна переносити цей висновок на simultaneous proposals. |
| Duplicate claim rejection | Другий claimant успішний після того, як commit першого вже видимий | `state::task_state`, `handle_claim_task` | `quorum_claim_fencing_and_stale_rejection` | **confirmed** | Одночасний сценарій broken нижче. |
| Stale generation rejection | Агент завершує/release ownership із неправильною generation | task-state fencing | `quorum_claim_fencing_and_stale_rejection` | **confirmed** | Прогалин не знайдено. |
| Completed-task fencing | Завершене завдання можна claim-нути знову | task-state reducer + claim path | `quorum_claim_fencing_and_stale_rejection` | **confirmed** | Прогалин не знайдено. |
| Network partition → heal | Обидві сторони живі, але роз'єднані; після heal не сходяться або псують identity/state | heartbeat stale-close, redial, anti-entropy | Reconnect/dead-peer tests перевіряють суміжні механізми; історичні docs описують SIGSTOP experiments | **partial** | Немає focused current executable partition→heal witness. Краще deterministic transport fault injection, а не privileged firewall CI. |
| Restart / incarnation semantics | Звичайний restart створює нове lifetime або втрачений data-dir повторно використовує старий event namespace | identity/journal + anti-entropy | `restart_preserves_incarnation_epoch_increments_seq_continues`; `reincarnation_does_not_collide_and_anti_entropy_converges` | **confirmed** | Прогалин не знайдено. |
| Унікальність event ID через restart/reincarnation | Однаковий `node-id:seq` двох lifetime дедуплікує різні facts | `Event::id`, incarnation-aware journal | `reincarnation_does_not_collide_and_anti_entropy_converges`; `journal.rs::has_distinguishes_incarnations` | **confirmed** | Legacy pre-incarnation events залишаються compatibility namespace навмисно. |
| Simultaneous/racing claims | Два proposers обидва отримують quorum і commit-ять той самий task/generation | `handle_claim_proposal`, `handle_claim_task`, voter promises | `claim_race.rs::simultaneous_claims_commit_exactly_one_owner_and_converge` **спростував інваріант** на head `0438477ea2d5fffbd68f351cc93cd1b3e4be80aa`: A і B обидва повернули `(ok ... generation 1 votes 2/3)` | **broken** | #35. Local self-vote обходить `node.promised`; потрібне одне atomic promise-acquisition rule. Witness лишається ignored до repair, але не послаблюється. |
| Delivery timeout / retry / reconnect redelivery | Kernel write успішний, але peer не ACK-нув; event зникає замість retry | `pending_acks`, `retry.rs`, heartbeat sweep | `silent_peer_delivery_never_silently_counts_as_acked`; `failed_delivery_is_redelivered_after_peer_reconnects`; unit tests persistence retry queue | **confirmed** для push-event ACK path | Корупція `retry.my` навмисно деградує до empty, бо retry queue не authoritative; це може втратити можливість redelivery, але не journal authority. |
| Task-state durability | Restart того самого node не відновлює task definitions/claims/completions зі свого journal | journal replay + `state.rs` reducers | Reincarnation/anti-entropy tests показують збереження replicated task facts через інший node і reconnect; normal restart доводить journal namespace/seq continuity | **partial** | Потрібен focused same-data-dir restart test для task-defined → claimed/completed, якщо це стане release gate. |
| Crash/restart під час state mutation | Crash посеред mutation знищує попередній повний durable state | `Journal::append`, `Journal::replace_all`, identity write | Append має append-first + fsync; fault injection для compaction відсутній | **broken** загалом | #36: `replace_all` truncates живий `events.log` до завершення replacement; потрібні temp→fsync→atomic rename та fault-injection proof. |
| Malformed/unknown wire messages | Поганий input crash-ить process, змінює state або приймається як valid op | `sexpr.rs`, connection dispatcher у `main.rs` | Parser unit tests є; connection handler логгує parse failure і продовжує; startup відкидає unknown CLI args | **partial** | Focused real-process malformed/unknown wire E2E witness не знайдений. Ризик нижчий за ownership/durability. |

## Конструктивні поломки, знайдені аудитом

### 1. Корупція authoritative durable state була fail-open

На базі аудиту malformed `node.my` перетворювався на empty S-expression і міг бути переписаний, а malformed/invalid records у `events.log` мовчки пропускалися. PR #34 переводить обидва authoritative recovery paths у fail-closed режим і додає контрприклади зі збереженням bytes. Валідний pre-M1.1a identity без `incarnation` залишається явним compatibility migration, а не помилково класифікується як corruption.

### 2. M0.6 simultaneous claims реально дають split-brain

Focused three-process barrier test був доданий саме тому, що наявний sequential quorum test прямо відділяє racing case як інше твердження. Exact-head CI не завис і не отримав «zero winner»: обидва незалежні clients commit-нули `RACE-0` на generation 1 із `2/3` votes.

```text
A=(ok (task RACE-0) (generation 1) (votes 2/3))
B=(ok (task RACE-0) (generation 1) (votes 2/3))
```

Причина: remote YES vote захоплює `node.promised`; automatic self-vote local proposer-а цього не робить. Тому A і B можуть кожен проголосувати за себе і водночас YES за конкурента. Див. #35.

### 3. Compaction не crash-atomic

`Journal::replace_all` зараз truncates живий `events.log` до завершення replacement. Коментар говорить про safe swap, але фактичного swap немає. Crash/ENOSPC/write failure після truncate може знищити останню повну історію. Див. #36.

## Ранжування прогалин за ризиком

Score = impact × likelihood × low-observability, кожен фактор 1–5. Це пріоритизація, а не оцінка ймовірності.

| Rank | Gap | I | L | O | Score | Чому |
|---:|---|---:|---:|---:|---:|---|
| 1 | #35 simultaneous-claim split brain | 5 | 4 | 4 | **80** | Два агенти можуть одночасно вважати один task своїм; executable reproduction вже є. |
| 2 | #36 truncate-in-place compaction | 5 | 3 | 4 | **60** | Може знищити authoritative history при crash/write failure; проявитися здатне лише після restart. |
| 3 | True network partition→heal proof | 5 | 3 | 3 | **45** | Reconnect mechanisms тестуються, але both-sides-alive partition може відкрити інші races. |
| 4 | Same-node task-state restart witness | 4 | 2 | 3 | **24** | Механізми виглядають сильними, але exact lifecycle proof відсутній. |
| 5 | Malformed/unknown wire E2E | 3 | 2 | 3 | **18** | Parser/dispatcher mechanisms існують; network-level rejection прямо не pressure-tested. |
| 6 | Test harness port/log observability | 3 | 3 | 2 | **18** | Може перетворити runner collision/slowness на оманливий protocol failure; сам state не псує. |

## Аналіз flaky surface

### Сильні сторони наявних тестів

- `eventually(...)` polling перевіряє semantic condition замість fixed sleep перед assert.
- `Node::Drop` робить `kill + wait`, зменшуючи ризик orphan-process interference.
- Частина liveness tests зменшує production timeout через явні test-only environment variables, а не глобально роздуває constants.
- Failed-delivery tests розрізняють local write success і peer ACK та мають durable retry path.

### Слабкі місця

1. **Fixed 3-second startup/read windows.** `wait_for_port`, `wait_for_file`, connect retry і client read timeout мають 3 s. Це bounds, не unconditional sleeps, але saturated hosted runner може перетворити startup latency на false protocol failure.
2. **Port allocation process-local, а не OS-reserved.** Legacy integration file збільшує `AtomicU16` від 15001. Це усуває collision лише між тестами одного процесу, але не резервує порт від зовнішнього процесу. Focused `claim_race` witness використовує ephemeral OS-assigned ports.
3. **Logs у legacy helper opt-in.** Без `SWARM_TEST_LOGS` child stdout/stderr ідуть у `/dev/null`; тоді handshake failure важко відрізнити від runner slowness. Focused race helper завжди пише per-node logs і включає startup log у diagnostics.
4. **Історичний handshake failure пройшов після rerun.** Тому startup/handshake evidence не слід вважати безумовно deterministic, доки failure не залишає достатньо diagnostics для розрізнення runner delay і product race.

### Політика щодо flakes

Не виправляти blanket timeout increase. Порядок дій:

- чекати protocol/state condition замість elapsed sleep;
- де практично — резервувати ports через ОС;
- автоматично зберігати child logs при failure;
- test-configurable timeout використовувати, коли тест доводить саме timeout mechanism;
- якщо тест intermittently падає, invariant залишається `partial`, доки причина не зрозуміла.

## Що реально змінив аудит

- Не додавав тести заради більшої цифри coverage.
- Перетворив два silent durable-state failure modes на fail-closed executable invariants.
- Додав targeted real-process simultaneous-claim witness і дозволив йому впасти. Цей failure відкрив #35, а не був прихований rerun-ом.
- Відкрив #36 для durability defect у mutation publication path.
- Partition/heal, same-node task-state restart і malformed-wire E2E лишені чесно `partial`, а не підняті до `confirmed` із документації.

## Команди відтворення / review

Звичайні audit gates:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Відомий broken M0.6 witness — очікувано падає до repair #35:

```sh
cargo test -p swarm-node --test claim_race -- --ignored --nocapture
```

Після repair #35 треба прибрати `#[ignore]` і зробити цей witness частиною звичайного workspace CI.

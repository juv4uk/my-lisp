# Swarm invariant audit — 2026-09-09

Issue: #21  
Audit base: `ccc91fd7821b899aa503831c0112914b285db87e`  
Working PR: #34

This audit answers a different question from LOC or raw `#[test]` counts:

> What would have to go wrong for two agents to diverge, lose durable coordination evidence, or accept stale ownership — and which executable proof catches it?

`crates/swarm-node/tests/integration.rs` is a large real-process TCP suite. One file therefore represents many distributed proofs; file/test-count ratios are not useful evidence by themselves.

## Evidence vocabulary

- **confirmed** — an executable test directly exercises the failure mode or the invariant's process/network boundary and currently passes.
- **partial** — relevant mechanism/tests exist, but the stated failure mode is not directly exercised end to end.
- **broken** — executable evidence or direct durable-state analysis shows the invariant is currently false.
- **unknown** — no adequate executable evidence was found.

A flaky proof is never promoted to `confirmed` merely because a retry passed.

## Invariant matrix

| Invariant | Failure mode | Implementation owner | Executable evidence | Status | Missing proof / action |
|---|---|---|---|---|---|
| Node identity ownership / shared data-dir exclusion | Two live processes mutate one identity/journal, or a different node-id reuses existing state | `journal.rs`, startup/data-dir lock in `main.rs` | `startup_rejects_data_dir_owned_by_another_identity`; `shared_data_dir_rejects_second_live_process_on_another_port` | **confirmed** | Keep startup validation before mutation. |
| Duplicate port startup safety | Failed second bind mutates durable identity or disturbs live owner | startup path in `main.rs` | `duplicate_port_fails_before_mutating_identity` | **confirmed** | None found. |
| Durable journal append / normal restart recovery | ACK/state is exposed before durable append, or restart resets event namespace/sequence | `journal.rs::append`, startup replay | `restart_preserves_incarnation_epoch_increments_seq_continues`; append uses `sync_data()` before in-memory commit/broadcast | **confirmed** for normal restart | Power-loss filesystem semantics remain outside loopback CI scope. |
| Corrupt/partial authoritative `node.my` | Parse failure silently becomes a new identity and rewrites durable state | `journal.rs::load_or_init_identity` | PR #34: `corrupt_identity_is_rejected_without_rewriting_it`; `malformed_identity_fields_are_rejected_without_rewriting_them`; legacy-upgrade counterexample | **confirmed after PR #34 fix** | Main at audit base was **broken** (`parse(...).unwrap_or(empty)`). |
| Corrupt/partial authoritative `events.log` | Invalid line is silently skipped and coordination evidence disappears on replay | `journal.rs::Journal::open` | PR #34: `journal_open_rejects_corrupt_line_without_truncating_history`; `journal_open_rejects_structurally_invalid_event_without_skipping_it` | **confirmed after PR #34 fix** | Main at audit base was **broken** (invalid lines silently skipped). |
| Anti-entropy convergence | Late peer permanently misses pre-existing facts | sync hello/events handlers in `main.rs`, `journal.rs` | `anti_entropy_sync_and_live_push_event`; `reincarnation_does_not_collide_and_anti_entropy_converges` | **confirmed** | None for ordinary reconnect/late join. |
| Live event propagation | Connected peer only converges after explicit resync | push-event/broadcast path in `main.rs` | `anti_entropy_sync_and_live_push_event` | **confirmed** | Delivery failure is covered separately by retry tests. |
| Reconnect convergence | A restarted/reconnected peer never recovers current replicated view | peer reconnect + anti-entropy | `failed_delivery_is_redelivered_after_peer_reconnects`; `p2p_presence_work_visibility_help_request_offer_and_reconnect` | **confirmed** | True network partition/heal is separate below. |
| Peer discovery / mesh convergence | A node connected to one bootstrap never learns/dials the rest | peer gossip in `main.rs` | `gossip_peer_discovery_reaches_full_mesh` | **confirmed** | None found for loopback mesh. |
| Quorum claim fencing — sequential | A propagated claim can be duplicated, wrong generation completed, or completed task reclaimed | claim proposal/vote/state reducers | `quorum_claim_fencing_and_stale_rejection` | **confirmed** only for sequential/post-propagation path | Must not be generalized to simultaneous proposals. |
| Duplicate claim rejection | Second claimant succeeds after first commit is visible | `state::task_state`, `handle_claim_task` | `quorum_claim_fencing_and_stale_rejection` | **confirmed** | Simultaneous case is broken below. |
| Stale generation rejection | Actor completes/releases ownership with wrong generation | task-state fencing | `quorum_claim_fencing_and_stale_rejection` | **confirmed** | None found. |
| Completed-task fencing | Completed work can be claimed again | task-state reducer + claim path | `quorum_claim_fencing_and_stale_rejection` | **confirmed** | None found. |
| Network partition → heal | Both sides remain alive but disconnected, then reconnect and converge without identity/state corruption | heartbeat stale-close, redial, anti-entropy | Reconnect/dead-peer tests exercise adjacent mechanisms; historical docs describe SIGSTOP experiments | **partial** | No focused current executable partition→heal witness. Prefer deterministic transport fault injection; do not require privileged firewall CI. |
| Node restart / incarnation semantics | Ordinary restart creates new lifetime, or lost data-dir reuses old event namespace | identity/journal + anti-entropy | `restart_preserves_incarnation_epoch_increments_seq_continues`; `reincarnation_does_not_collide_and_anti_entropy_converges` | **confirmed** | None found. |
| Event-ID uniqueness across restart / reincarnation | Same `node-id:seq` from two lifetimes deduplicates distinct facts | `Event::id`, incarnation-aware journal | `reincarnation_does_not_collide_and_anti_entropy_converges`; `journal.rs::has_distinguishes_incarnations` | **confirmed** | Legacy pre-incarnation events remain compatibility namespace by design. |
| Simultaneous/racing claims | Two proposers each obtain quorum and both commit same task/generation | `handle_claim_proposal`, `handle_claim_task`, voter promises | `claim_race.rs::simultaneous_claims_commit_exactly_one_owner_and_converge` **falsified** head `0438477ea2d5fffbd68f351cc93cd1b3e4be80aa`: A and B both returned `(ok ... generation 1 votes 2/3)` | **broken** | #35. Local self-vote bypasses `node.promised`; share one atomic promise-acquisition rule. Witness stays ignored until repaired, not weakened. |
| Delivery timeout / retry / reconnect redelivery | Kernel write succeeds but peer never ACKs; event vanishes instead of retrying | `pending_acks`, `retry.rs`, heartbeat sweep | `silent_peer_delivery_never_silently_counts_as_acked`; `failed_delivery_is_redelivered_after_peer_reconnects`; retry queue persistence unit tests | **confirmed** for push-event ACK path | `retry.my` corruption intentionally degrades to empty because retry queue is non-authoritative; this can delay/loss redelivery opportunity but not erase journal authority. |
| Task-state durability | Restart of the same node preserves all task definitions/claims/completions from its own journal | journal replay + `state.rs` reducers | Reincarnation/anti-entropy tests prove replicated task facts survive another node and reconnect; normal restart test proves journal namespace/seq continuity | **partial** | Add a focused same-data-dir restart test covering task-defined → claimed/completed state if this becomes a release gate. |
| Crash/restart around state mutation | Crash during mutation destroys previous complete durable state | `Journal::append`, `Journal::replace_all`, identity write | Append is append-first + fsync; no fault injection around compaction | **broken** overall | #36: `replace_all` truncates live `events.log` before writing replacement; require temp→fsync→atomic rename and fault-injection proof. |
| Malformed/unknown wire messages | Bad input crashes process, mutates state, or is accepted as a valid operation | `sexpr.rs`, connection dispatcher in `main.rs` | Parser unit tests exist; connection handler logs parse failure and continues; startup rejects unknown CLI args | **partial** | No focused real-process malformed/unknown wire protocol rejection witness found. Lower risk than ownership/durability gaps. |

## Constructive failures found by this audit

### 1. Authoritative durable-state corruption failed open

At the audit base, malformed `node.my` was converted to an empty S-expression and could be rewritten, while malformed/invalid `events.log` records were silently skipped. PR #34 changes both authoritative recovery paths to fail closed and adds byte-preservation counterexamples. A valid pre-M1.1a identity without `incarnation` remains an explicit compatibility migration, not corruption.

### 2. M0.6 simultaneous claims can split brain

The focused three-process barrier test was intentionally added because the existing sequential quorum test says the racing case is a different assertion. Exact-head CI did not merely time out: both independent clients committed `RACE-0` at generation 1 with `2/3` votes.

```text
A=(ok (task RACE-0) (generation 1) (votes 2/3))
B=(ok (task RACE-0) (generation 1) (votes 2/3))
```

Root cause: remote YES votes acquire `node.promised`; the local proposer's automatic self-vote does not. Therefore A and B can each self-vote and also vote YES for the other. See #35.

### 3. Compaction is not crash-atomic

`Journal::replace_all` currently truncates the live `events.log` before the replacement is complete. The code comment describes a safe swap, but there is no swap. Crash/ENOSPC/write failure after truncate can destroy the last complete history. See #36.

## Risk-ranked gaps

Score = impact × likelihood × low-observability, each 1–5. This is prioritization, not a probability claim.

| Rank | Gap | I | L | O | Score | Why |
|---:|---|---:|---:|---:|---:|---|
| 1 | #35 simultaneous-claim split brain | 5 | 4 | 4 | **80** | Two agents can both believe they own the same task; executable reproduction already exists. |
| 2 | #36 truncate-in-place compaction | 5 | 3 | 4 | **60** | Can destroy authoritative history on crash/write failure; failure may only surface on restart. |
| 3 | True network partition→heal proof | 5 | 3 | 3 | **45** | Reconnect mechanisms are tested, but a both-sides-alive partition can expose different races. |
| 4 | Same-node task-state restart witness | 4 | 2 | 3 | **24** | Mechanisms strongly suggest recovery, but exact lifecycle proof is missing. |
| 5 | Malformed/unknown wire E2E | 3 | 2 | 3 | **18** | Parser/dispatcher mechanisms exist; network-level rejection is not directly pressure-tested. |
| 6 | Test harness port/log observability | 3 | 3 | 2 | **18** | Can turn runner collisions/slowness into misleading protocol failures; does not itself corrupt state. |

## Flake analysis

### Good existing practices

- `eventually(...)` polls a semantic condition instead of sleeping a fixed amount before asserting.
- `Node::Drop` kills and waits for child processes, reducing orphan-process interference.
- Several liveness tests shrink production timeouts through explicit test-only environment variables instead of globally inflating constants.
- Failed-delivery tests distinguish local write success from peer ACK and preserve a durable retry path.

### Weak points

1. **Fixed 3-second startup/read windows.** `wait_for_port`, `wait_for_file`, connect retry, and client read timeout all use 3 s. These are bounds, not unconditional sleeps, but a saturated hosted runner can turn startup latency into a false protocol failure.
2. **Port allocation is process-local, not OS-reserved.** The legacy integration file increments an `AtomicU16` from 15001. It prevents collisions among tests in that one process but cannot reserve against unrelated processes/jobs. The focused `claim_race` witness uses ephemeral OS-assigned ports instead.
3. **Logs are opt-in in the legacy helper.** Unless `SWARM_TEST_LOGS` is set, child stdout/stderr go to `/dev/null`; a failed handshake can therefore be indistinguishable from runner slowness. The focused race helper always records per-node logs and includes startup logs in failure diagnostics.
4. **A historical handshake failure passed on rerun.** Therefore handshake/startup timing evidence should remain suspect until failures preserve enough diagnostics to classify runner delay vs product race.

### Flake policy

Do **not** fix these by blanket timeout increases. Prefer, in order:

- wait for protocol/state conditions rather than elapsed sleeps;
- reserve ports via the OS where practical;
- preserve child logs automatically on failure;
- make timeout values test-configurable only when a test is proving the timeout mechanism itself;
- classify a proof as `partial` if it intermittently fails until the failure mode is understood.

## What the audit changed

- It did **not** add tests to improve a count.
- It converted two silent durable-state failure modes into fail-closed executable invariants.
- It added a targeted real-process simultaneous-claim witness and allowed it to fail. That failure opened #35 rather than being hidden by retrying CI.
- It opened #36 for a durability defect discovered by reading the mutation publication path.
- It leaves partition/heal, same-node task-state restart, and malformed-wire E2E explicitly `partial` instead of promoting them from documentation.

## Reproduction / review commands

Normal audit gates:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Known-broken M0.6 witness (expected to fail until #35 is fixed):

```sh
cargo test -p swarm-node --test claim_race -- --ignored --nocapture
```

Once #35 is repaired, remove the `#[ignore]` and make that command part of ordinary workspace CI.

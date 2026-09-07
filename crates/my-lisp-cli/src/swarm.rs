//! swarm.rs - the legacy semantic TCP/oracle layer still owned by the CLI:
//! the sexpr request/response protocol, broker/mailbox/claims/presence/task
//! tables, and the `--connect` P2P client. Moved out of main.rs verbatim
//! (2026-08-22 mechanical CLI split, no behavior changes).

use my_lisp::{
    eval_parsed_expressions, eval_parsed_expressions_incremental, parse, Environment, ErrorKind,
    Exactness, Session, Value,
};
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::process;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

/// Walks a `Value` list (the `Pair`-chain shape `Value::list` builds) into
/// a `Vec`, stopping at the first non-`Pair` tail. Used only for reading
/// the machine-protocol's own request/response envelopes — the language
/// itself has `car`/`cdr` for this, but the CLI here is reading a `Value`
/// that was never `def`d into a running `Session`.
fn list_items(mut value: &Value) -> Vec<Value> {
    let mut items = Vec::new();
    while let Value::Pair(head, tail) = value {
        items.push((**head).clone());
        value = tail;
    }
    items
}

enum RequestFrame {
    Eof,
    Line(Vec<u8>),
    TooLarge,
}

/// Read one newline-delimited frame without ever allocating more than `limit`
/// bytes for it. Oversized frames are drained through the newline and then
/// reported, so the connection cannot be desynchronised by the rejected data.
fn read_request_frame(reader: &mut BufReader<TcpStream>, limit: usize) -> io::Result<RequestFrame> {
    let mut bytes = Vec::with_capacity(limit.min(4096));
    let mut too_large = false;
    loop {
        let chunk = reader.fill_buf()?;
        if chunk.is_empty() {
            return Ok(if too_large {
                RequestFrame::TooLarge
            } else if bytes.is_empty() {
                RequestFrame::Eof
            } else {
                RequestFrame::Line(bytes)
            });
        }
        if let Some(newline) = chunk.iter().position(|byte| *byte == b'\n') {
            let frame_len = newline + 1;
            if too_large || bytes.len() + frame_len > limit {
                reader.consume(frame_len);
                return Ok(RequestFrame::TooLarge);
            }
            bytes.extend_from_slice(&chunk[..frame_len]);
            reader.consume(frame_len);
            return Ok(RequestFrame::Line(bytes));
        }
        if too_large || bytes.len() + chunk.len() > limit {
            too_large = true;
            let consume_len = chunk.len();
            reader.consume(consume_len);
        } else {
            let copied = chunk.to_vec();
            let consume_len = copied.len();
            bytes.extend_from_slice(&copied);
            reader.consume(consume_len);
        }
    }
}

/// Looks up `(key . value)` in a dotted-pair alist like
/// `language-contract.my`'s `((major . 1) (minor . 0) ...)` — distinct
/// from `list_items`' 2-element-list reading of the request/response
/// envelope, since a dotted pair's cdr is the value directly, not a
/// nested one-element list.
pub(crate) fn dotted_alist_lookup(alist: &Value, key: &str) -> Option<Value> {
    list_items(alist).into_iter().find_map(|item| match &item {
        Value::Pair(k, v) => match &**k {
            Value::Symbol(name) if &**name == key => Some((**v).clone()),
            _ => None,
        },
        _ => None,
    })
}

/// A bare atom usable as an id or capability name — a `Symbol` or a
/// `String`, both of which the data files (`tasks.my`, `ecosystem-status.my`)
/// legitimately use for identifiers.
fn atom_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.to_string()),
        Value::Symbol(s) => Some(s.to_string()),
        _ => None,
    }
}

/// `capabilities`/`depends-on` from a data file are plain lists of
/// symbols or strings; a malformed entry in the middle is skipped rather
/// than failing the whole file, and the caller reports it as a warning.
fn list_of_atoms(value: &Value) -> Vec<String> {
    list_items(value)
        .into_iter()
        .filter_map(|item| atom_string(&item))
        .collect()
}

/// The `done` status in `tasks.my` — accepts the same spellings a data
/// file might use for a boolean, `None` only if the field is present but
/// unrecognized (the caller then keeps the existing in-memory status
/// rather than guessing).
fn bool_from_value(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(b) => Some(*b),
        Value::Nil => Some(false),
        Value::Symbol(s) | Value::String(s) => match &**s {
            "t" | "true" | "yes" => Some(true),
            "nil" | "false" | "no" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

/// Set once, at this process's first response, to the wall-clock second
/// it started — not persisted, not synced, just a number that's
/// guaranteed different after any restart. Exists for exactly one
/// reason: a *stateless* client (one tool call per turn, no long-lived
/// process to hold a `subscribe` connection or notice a dropped socket)
/// has no other way to detect "the server I last talked to is gone and
/// this is a new one" — everything else in this protocol (`claim`,
/// `presence`, the task registry) resets silently on restart, and
/// polling alone can't distinguish "nothing changed" from "everything
/// I knew was wiped." Carried on every response so any single call
/// reveals it, without a dedicated op to ask.
static SERVER_GENERATION: std::sync::OnceLock<u64> = std::sync::OnceLock::new();

/// 1-indexed (line, column) for a byte offset into `source` — turns a
/// `LanguageError`'s `span.start` (a raw byte position) into something
/// a human or agent can actually jump to in an editor, for
/// `validate-tasks`'s error reporting.
fn line_col_at(source: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, ch) in source.char_indices() {
        if i >= byte_offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// A relative `file` path (`sync-tasks`/`sync-milestone`/`validate-tasks`)
/// resolves against *this server process's* working directory, not the
/// caller's — a trap a caller can walk straight into with no error:
/// the read succeeds, just against whatever file happens to exist at
/// that relative path on the server's side, silently syncing the wrong
/// data. Rejecting anything non-absolute turns that into a clear error
/// instead of a quiet mismatch.
fn require_absolute_path(path: &str) -> Result<(), String> {
    if std::path::Path::new(path).is_absolute() {
        Ok(())
    } else {
        Err(format!(
            "`file` must be an absolute path — `{path}` would resolve against this server's own working directory, not the caller's, and silently read the wrong file"
        ))
    }
}

fn server_generation() -> u64 {
    *SERVER_GENERATION.get_or_init(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    })
}

/// `output` carries every `print`/`println`-style side-effect line the
/// evaluated expression produced, in order — dropping it (the first cut
/// of this protocol did) silently discards real program output, which is
/// exactly the "optimistic" half-truth this protocol exists to prevent.
fn ok_response(id: &Value, value: Value, output: &[String], contract_version: &Value) -> Value {
    Value::list([
        Value::Symbol("response".into()),
        Value::list([Value::Symbol("id".into()), id.clone()]),
        Value::list([Value::Symbol("status".into()), Value::Symbol("ok".into())]),
        Value::list([Value::Symbol("value".into()), value]),
        Value::list([
            Value::Symbol("output".into()),
            Value::list(
                output
                    .iter()
                    .map(|line| Value::String(line.as_str().into())),
            ),
        ]),
        Value::list([
            Value::Symbol("contract-version".into()),
            contract_version.clone(),
        ]),
        Value::list([
            Value::Symbol("server-generation".into()),
            Value::Number(server_generation() as f64, Exactness::Exact),
        ]),
    ])
}

fn error_response(id: &Value, kind: &str, message: &str, contract_version: &Value) -> Value {
    Value::list([
        Value::Symbol("response".into()),
        Value::list([Value::Symbol("id".into()), id.clone()]),
        Value::list([
            Value::Symbol("status".into()),
            Value::Symbol("error".into()),
        ]),
        Value::list([Value::Symbol("kind".into()), Value::Symbol(kind.into())]),
        Value::list([
            Value::Symbol("message".into()),
            Value::String(message.into()),
        ]),
        Value::list([
            Value::Symbol("contract-version".into()),
            contract_version.clone(),
        ]),
        Value::list([
            Value::Symbol("server-generation".into()),
            Value::Number(server_generation() as f64, Exactness::Exact),
        ]),
    ])
}

/// Same wire shape as `error_response`, plus one additive `(classification
/// ...)` field for a genuine `ErrorKind`-derived failure (evaluator/parser
/// errors only — request-shape validation like "missing field" has no
/// `ErrorKind` to classify and keeps using `error_response`). `status`
/// stays `error` and `kind` stays exactly as before, so an existing
/// consumer that only reads those two fields (e.g. `conformance-check.py`,
/// `mccarthy.rs`) is unaffected — this is Stage 1 of a two-stage design
/// (`ecosystem/decisions/`), not a protocol-version change.
fn error_response_classified(
    id: &Value,
    kind: &str,
    classification: my_lisp::Classification,
    message: &str,
    contract_version: &Value,
) -> Value {
    Value::list([
        Value::Symbol("response".into()),
        Value::list([Value::Symbol("id".into()), id.clone()]),
        Value::list([
            Value::Symbol("status".into()),
            Value::Symbol("error".into()),
        ]),
        Value::list([Value::Symbol("kind".into()), Value::Symbol(kind.into())]),
        Value::list([
            Value::Symbol("classification".into()),
            Value::Symbol(classification.as_str().into()),
        ]),
        Value::list([
            Value::Symbol("message".into()),
            Value::String(message.into()),
        ]),
        Value::list([
            Value::Symbol("contract-version".into()),
            contract_version.clone(),
        ]),
        Value::list([
            Value::Symbol("server-generation".into()),
            Value::Number(server_generation() as f64, Exactness::Exact),
        ]),
    ])
}

fn protocol_error(kind: &str, message: &str, contract_version: &Value) -> Value {
    Value::list([
        Value::Symbol("protocol-error".into()),
        Value::list([Value::Symbol("kind".into()), Value::Symbol(kind.into())]),
        Value::list([
            Value::Symbol("message".into()),
            Value::String(message.into()),
        ]),
        Value::list([
            Value::Symbol("contract-version".into()),
            contract_version.clone(),
        ]),
        Value::list([
            Value::Symbol("server-generation".into()),
            Value::Number(server_generation() as f64, Exactness::Exact),
        ]),
    ])
}

fn source_digest(source: &str) -> String {
    let digest = my_lisp::sha256_source(source.as_bytes());
    let mut hex = String::with_capacity(7 + digest.len() * 2);
    hex.push_str("sha256:");
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

fn exact_usize(value: usize) -> Value {
    Value::Number(value as f64, Exactness::Exact)
}

/// Turn the canonical `LanguageError` into an agent-oriented, machine-readable
/// diagnostic.  The parser remains the only syntax authority; this merely
/// preserves its span and adds a conservative suggested edit for the two most
/// common parenthesis failures.  The edit is advice, never applied silently.
fn oracle_error_payload(source: &str, error: &my_lisp::LanguageError) -> Value {
    let (line, column) = error.line_col(source);
    let (code, guidance, repair) = if error.message.starts_with("unclosed list") {
        (
            "unclosed-list",
            "Додайте одну ')' в кінці незакритої форми · Add one ')' at the end of the unclosed form",
            Value::list([
                Value::list([Value::Symbol("action".into()), Value::Symbol("insert".into())]),
                Value::list([Value::Symbol("text".into()), Value::String(")".into())]),
                Value::list([Value::Symbol("offset".into()), exact_usize(source.len())]),
            ]),
        )
    } else if error.message.starts_with("unexpected closing parenthesis") {
        (
            "unexpected-closing-parenthesis",
            "Приберіть зайву ')' у вказаному діапазоні · Remove the extra ')' at the reported span",
            Value::list([
                Value::list([
                    Value::Symbol("action".into()),
                    Value::Symbol("delete".into()),
                ]),
                Value::list([Value::Symbol("start".into()), exact_usize(error.span.start)]),
                Value::list([Value::Symbol("end".into()), exact_usize(error.span.end)]),
            ]),
        )
    } else {
        (
            error_kind_symbol(&error.kind),
            "Перевірте вказаний діапазон і повторіть oracle-check · Inspect the reported span and run oracle-check again",
            Value::list([Value::list([
                Value::Symbol("action".into()),
                Value::Symbol("inspect".into()),
            ])]),
        )
    };

    Value::list([
        Value::Symbol("error".into()),
        Value::list([
            Value::list([
                Value::Symbol("kind".into()),
                Value::Symbol(error_kind_symbol(&error.kind).into()),
            ]),
            Value::list([Value::Symbol("code".into()), Value::Symbol(code.into())]),
            Value::list([
                Value::Symbol("message".into()),
                Value::String(error.message.clone().into()),
            ]),
            Value::list([
                Value::Symbol("span".into()),
                Value::list([
                    Value::list([Value::Symbol("start".into()), exact_usize(error.span.start)]),
                    Value::list([Value::Symbol("end".into()), exact_usize(error.span.end)]),
                ]),
            ]),
            Value::list([
                Value::Symbol("location".into()),
                Value::list([
                    Value::list([Value::Symbol("line".into()), exact_usize(line)]),
                    Value::list([Value::Symbol("column".into()), exact_usize(column)]),
                ]),
            ]),
            Value::list([
                Value::Symbol("rendered".into()),
                Value::String(error.render(source).into()),
            ]),
            Value::list([
                Value::Symbol("guidance".into()),
                Value::String(guidance.into()),
            ]),
            Value::list([Value::Symbol("suggested-edit".into()), repair]),
        ]),
    ])
}

/// Versioned semantic result carried as the value of `oracle-eval`'s normal
/// transport response.  Transport success and language outcome are separate:
/// an evaluator error is a successfully delivered `(outcome error)` record,
/// not a TCP/protocol failure. Existing `eval` clients remain byte-compatible.
fn oracle_result_value(
    source: &str,
    contract_version: &Value,
    outcome: Value,
    payload: Value,
    output: &[String],
    evidence_class: &str,
) -> Value {
    Value::list([
        Value::Symbol("oracle-result".into()),
        Value::list([
            Value::Symbol("protocol".into()),
            Value::Symbol("oracle-result/1".into()),
        ]),
        Value::list([
            Value::Symbol("contract-revision".into()),
            contract_version.clone(),
        ]),
        Value::list([
            Value::Symbol("source-digest".into()),
            Value::String(source_digest(source).into()),
        ]),
        Value::list([
            Value::Symbol("implementation-revision".into()),
            Value::String(format!("my-lisp-cli/{}", env!("CARGO_PKG_VERSION")).into()),
        ]),
        Value::list([Value::Symbol("outcome".into()), outcome]),
        payload,
        Value::list([
            Value::Symbol("output".into()),
            Value::list(
                output
                    .iter()
                    .map(|line| Value::String(line.as_str().into())),
            ),
        ]),
        Value::list([
            Value::Symbol("evidence-class".into()),
            Value::Symbol(evidence_class.into()),
        ]),
        Value::list([
            Value::Symbol("relation".into()),
            Value::Symbol("reference-implementation".into()),
        ]),
        Value::list([
            Value::Symbol("provenance".into()),
            Value::list([
                Value::list([
                    Value::Symbol("repository".into()),
                    Value::String("my-lisp".into()),
                ]),
                Value::list([
                    Value::Symbol("runner".into()),
                    Value::String("my-lisp-cli".into()),
                ]),
            ]),
        ]),
    ])
}

/// Side-effect-free agent preflight shared by the local CLI and TCP Oracle.
/// `true` means the complete source parsed; no evaluation has taken place.
pub(crate) fn oracle_check(source: &str, contract_version: &Value) -> (Value, bool) {
    match parse(source) {
        Ok(ast) => (
            oracle_result_value(
                source,
                contract_version,
                Value::Symbol("valid".into()),
                Value::list([
                    Value::Symbol("syntax".into()),
                    Value::list([
                        Value::list([
                            Value::Symbol("status".into()),
                            Value::Symbol("valid".into()),
                        ]),
                        Value::list([Value::Symbol("forms".into()), exact_usize(ast.len())]),
                    ]),
                ]),
                &[],
                "oracle-syntax-check",
            ),
            true,
        ),
        Err(error) => (
            oracle_result_value(
                source,
                contract_version,
                Value::Symbol("error".into()),
                oracle_error_payload(source, &error),
                &[],
                "oracle-syntax-check",
            ),
            false,
        ),
    }
}

/// Query the WSM-owned reference directory without duplicating its entries in
/// Rust. The session already contains core.my; Guard is the one shared
/// embed in `wsm-guard-core` (same library every Guard consumer uses), but
/// its reference directory is read fresh from disk on every call, exactly
/// like the LSP's `guard_knowledge.rs` already does — adding or editing a
/// topic in `knowledge/guard-reference.wsm` takes effect immediately, no
/// rebuild. `None` lists the curated frequently-used tool names. A named
/// query goes through `guard-ask`, which searches the tool directory and
/// the reference-topic directory together and tags the result `type
/// tool` / `type reference-topic` / `ambiguous` (found in both) / a
/// generic unknown-routed `not-found` — a cold agent asks one thing and
/// never needs to know which of the two directories actually held the
/// answer.
pub(crate) fn oracle_help(session: &mut Session, topic: Option<&str>) -> Result<Value, String> {
    let reference = std::fs::read_to_string("knowledge/guard-reference.wsm").map_err(|error| {
        format!(
            "cannot read knowledge/guard-reference.wsm (run from the my-lisp repo root): {error}"
        )
    })?;
    for (name, source) in [
        ("lib/guard.wsm", wsm_guard_core::GUARD),
        ("knowledge/guard-reference.wsm", reference.as_str()),
    ] {
        let ast = parse(source)
            .map_err(|error| format!("cannot parse {name}: {}", error.render(source)))?;
        eval_parsed_expressions_incremental(&ast, session)
            .map_err(|error| format!("cannot load {name}: {}", error.render(source)))?;
    }

    let query = match topic {
        None => "(guard-scripts *guard-script-directory*)".to_string(),
        Some(topic)
            if !topic.is_empty()
                && topic
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '-') =>
        {
            format!("(guard-ask (quote {topic}))")
        }
        Some(_) => {
            return Err("oracle-help topic must contain only ASCII letters, digits, or '-'".into())
        }
    };
    let ast = parse(&query).map_err(|error| error.render(&query))?;
    eval_parsed_expressions_incremental(&ast, session)
        .map(|result| result.value)
        .map_err(|error| error.render(&query))
}

/// A single `notify`d message, kept in `run_tcp_repl_sexpr`'s in-memory
/// mailbox — deliberately separate from any `Session`/`Environment`, so
/// agent coordination never touches the isolated eval-oracle state each
/// connection gets (see this function's own doc comment). `to: None`
/// means broadcast to every `poll`er.
struct MailboxEntry {
    id: u64,
    from: String,
    to: Option<String>,
    message: String,
}

/// `notify`/`poll`'s shared state, now behind a `Mutex` since
/// `run_tcp_repl_sexpr` handles connections concurrently (one OS thread
/// per connection) rather than one at a time — see that function's own
/// doc comment for why threading was safe to add despite `Value`'s `Rc`
/// (never shared across threads; only plain, `Send`-safe `String`s cross
/// the thread boundary, via this struct and `Subscriber::sender`).
#[derive(Default)]
struct MailboxState {
    entries: Vec<MailboxEntry>,
    next_id: u64,
}

/// One `subscribe`d connection's live channel — `publish` looks up every
/// `Subscriber` whose `topics` is empty (subscribed to everything) or
/// contains the published topic, and sends the already-rendered event
/// text (a plain `String`, not a `Value`) down `sender`. The connection's
/// own thread blocks on the matching `Receiver`, writing each event to
/// its socket as it arrives — genuine push, not polling.
struct Subscriber {
    id: u64,
    topics: Vec<String>,
    sender: mpsc::Sender<String>,
}

/// One published event, kept in `Broker::event_log` alongside live
/// delivery so a `subscribe` with `since` can replay everything a
/// reconnecting agent missed while its connection was down — the gap
/// `AGENTS.md`'s durability warning describes ("every op resets on
/// restart") doesn't have to mean a *subscriber* restart also loses
/// history, only that the log itself doesn't survive the *server*
/// restarting. Capped the same way the mailbox is (oldest-first drain).
struct StoredEvent {
    id: u64,
    from: String,
    topic: String,
    message: String,
}

#[derive(Default)]
struct Broker {
    subscribers: Vec<Subscriber>,
    next_subscriber_id: u64,
    event_log: Vec<StoredEvent>,
    next_event_id: u64,
}

fn stored_event_to_text(event: &StoredEvent) -> String {
    Value::list([
        Value::Symbol("event".into()),
        Value::list([
            Value::Symbol("id".into()),
            Value::Number(event.id as f64, Exactness::Exact),
        ]),
        Value::list([
            Value::Symbol("from".into()),
            Value::String(event.from.as_str().into()),
        ]),
        Value::list([
            Value::Symbol("topic".into()),
            Value::String(event.topic.as_str().into()),
        ]),
        Value::list([
            Value::Symbol("message".into()),
            Value::String(event.message.as_str().into()),
        ]),
    ])
    .to_string()
}

/// `claim`/`release`'s shared state — a task id maps to the agent name
/// holding it, or is absent if unclaimed. `claim` is compare-and-swap in
/// spirit: it only succeeds if the task has no holder yet (or the caller
/// already holds it, so a re-`claim` is idempotent), all under one lock
/// acquisition, so two agents racing for the same task can never both
/// win. Same rendered-`String`-only rule as the mailbox/broker — this
/// only ever stores plain task-id/agent-name strings, never a `Value`.
/// In-memory, non-persistent, gone on server restart — same as the
/// mailbox and broker, and for the same reason: this is a coordination
/// hint (who's working on what *right now*), not the durable record of
/// what got done. A completed task's actual evidence still belongs in
/// `evidence/`, not here.
#[derive(Default)]
struct ClaimTable {
    holders: std::collections::HashMap<String, String>,
}

/// One agent's registered presence — `hello` inserts/overwrites it,
/// `heartbeat` refreshes `last_seen`/`task`, `presence` reads the whole
/// table. `last_seen` is a plain `Instant` (`Copy`, `Send`, no `Rc`), so
/// this crosses thread boundaries the same safe way everything else in
/// this file does. No automatic eviction — a stale entry just reports a
/// large `seconds-since-heartbeat` in `presence`, so callers decide their
/// own liveness threshold rather than the server silently deciding one.
struct PresenceEntry {
    project: Option<String>,
    capabilities: Vec<String>,
    task: Option<String>,
    last_seen: std::time::Instant,
}

#[derive(Default)]
struct PresenceTable {
    agents: std::collections::HashMap<String, PresenceEntry>,
}

/// A `define-task`d unit of work — the machine-readable task list
/// `docs/swarm-coordination.md`'s `next-best-action` scoring needs.
/// `depends_on` names other task ids that must be `complete-task`d
/// first; a task with any unsatisfied dependency is excluded from
/// `next-best-action`'s ranking entirely (claiming it would just block
/// immediately). `capabilities` is what an agent needs to even be
/// considered for it — empty means anyone qualifies.
struct TaskDef {
    priority: f64,
    capabilities: Vec<String>,
    depends_on: Vec<String>,
    done: bool,
    /// Optional prose — what the task actually is, so `next-best-action`/
    /// `list-claims` results are self-contained instead of forcing a
    /// second lookup (a durable file, an old `task-created` event) just
    /// to find out what a ranked task id means.
    description: Option<String>,
}

#[derive(Default)]
struct TaskTable {
    tasks: std::collections::HashMap<String, TaskDef>,
}

/// Shared by `publish`, `capability-request`, and every op that
/// auto-publishes a lifecycle event (`claim`/`release`/`hello`/
/// `define-task`, below) — one delivery path so `subscribe`rs never see
/// a different envelope shape depending on who triggered the event.
/// Returns how many subscribers actually received it.
fn broadcast_event(broker: &Arc<Mutex<Broker>>, from: &str, topic: &str, message: &str) -> u32 {
    let mut delivered = 0u32;
    let mut broker_state = broker.lock().unwrap_or_else(|e| e.into_inner());
    // Log first, deliver second, both under this one lock acquisition —
    // that's what makes `subscribe`'s replay-then-live-register race-free
    // (see its own comment): no event can land between a subscriber
    // taking its replay snapshot and registering for live delivery,
    // because both of those also happen under this same `broker` lock.
    broker_state.next_event_id += 1;
    let event_id = broker_state.next_event_id;
    broker_state.event_log.push(StoredEvent {
        id: event_id,
        from: from.to_string(),
        topic: topic.to_string(),
        message: message.to_string(),
    });
    const EVENT_LOG_CAPACITY: usize = 500;
    if broker_state.event_log.len() > EVENT_LOG_CAPACITY {
        let excess = broker_state.event_log.len() - EVENT_LOG_CAPACITY;
        broker_state.event_log.drain(0..excess);
    }
    let event_text = stored_event_to_text(broker_state.event_log.last().expect("just pushed"));
    broker_state.subscribers.retain(|subscriber| {
        if !subscriber.topics.is_empty() && !subscriber.topics.iter().any(|t| t == topic) {
            return true;
        }
        match subscriber.sender.send(event_text.clone()) {
            Ok(()) => {
                delivered += 1;
                true
            }
            // The subscriber's connection thread is gone (client
            // disconnected) — drop it here too, rather than leaking a
            // dead entry forever.
            Err(_) => false,
        }
    });
    delivered
}

fn presence_entry_to_value(agent: &str, entry: &PresenceEntry) -> Value {
    Value::list([
        Value::list([Value::Symbol("agent".into()), Value::String(agent.into())]),
        Value::list([
            Value::Symbol("project".into()),
            match &entry.project {
                Some(project) => Value::String(project.as_str().into()),
                None => Value::Nil,
            },
        ]),
        Value::list([
            Value::Symbol("capabilities".into()),
            Value::list(
                entry
                    .capabilities
                    .iter()
                    .map(|c| Value::Symbol(c.as_str().into())),
            ),
        ]),
        Value::list([
            Value::Symbol("task".into()),
            match &entry.task {
                Some(task) => Value::String(task.as_str().into()),
                None => Value::Nil,
            },
        ]),
        Value::list([
            Value::Symbol("seconds-since-heartbeat".into()),
            Value::Number(entry.last_seen.elapsed().as_secs_f64(), Exactness::Inexact),
        ]),
    ])
}

fn mailbox_entry_to_value(entry: &MailboxEntry) -> Value {
    Value::list([
        Value::list([
            Value::Symbol("id".into()),
            Value::Number(entry.id as f64, Exactness::Exact),
        ]),
        Value::list([
            Value::Symbol("from".into()),
            Value::String(entry.from.as_str().into()),
        ]),
        Value::list([
            Value::Symbol("to".into()),
            match &entry.to {
                Some(to) => Value::String(to.as_str().into()),
                None => Value::Nil,
            },
        ]),
        Value::list([
            Value::Symbol("message".into()),
            Value::String(entry.message.as_str().into()),
        ]),
    ])
}

fn error_kind_symbol(kind: &ErrorKind) -> &'static str {
    match kind {
        ErrorKind::Parse => "parse-error",
        ErrorKind::UnknownSymbol => "unknown-symbol",
        ErrorKind::Arity => "arity-error",
        ErrorKind::Type => "type-error",
        ErrorKind::InvalidForm => "invalid-form",
        ErrorKind::OutOfMemory => "out-of-memory",
        ErrorKind::NumericOverflow => "numeric-overflow",
        ErrorKind::DivisionByZero => "division-by-zero",
    }
}

/// `--tcp=PORT --protocol=sexpr` — the same live oracle as `run_tcp_repl`,
/// but for machines instead of humans: no banner, no prompt, one strict
/// `(request (id ..) (op ..) (source ..))` in, one `(response (id ..)
/// (status ..) ..)` out, every time, so `cml`/`fpga-lisp`/`my-idea` can
/// parse a response without guessing whether a given line is a value, an
/// error, or REPL chrome. Op set: `eval`, `parse`, `diagnose`,
/// `contract-version` for semantic-oracle use. `oracle-eval` returns the
/// same semantics in a versioned, provenance-bearing result value while
/// leaving the legacy `eval` response unchanged. `notify`/`poll` are for
/// short-lived, poll-based agent mailbox; `subscribe`/`publish` for
/// genuine push (owner decision, 2026-08-12) — a `subscribe`d connection
/// blocks and receives `(event ...)` lines the instant a matching
/// `publish` happens on any other connection, not on the next poll.
///
/// One OS thread per accepted connection (changed from strictly
/// sequential handling to make `subscribe` possible — a subscriber has
/// to block waiting for events while other connections keep working).
/// Each thread builds its own `Session`/`Environment` locally and never
/// shares it — `Value`'s `Rc`-based sharing (non-atomic refcounts) would
/// be unsound across threads, so nothing `Rc`-based ever crosses a
/// thread boundary here: `contract_version` is rebuilt fresh per
/// connection from two plain `f64`s, and the mailbox/broker only ever
/// pass already-`to_string()`-rendered `String`s between threads, never
/// a live `Value`. The isolation guarantee (`eval`/`parse`/`diagnose`
/// state invisible across connections) is now also physical (separate
/// threads), not just logical (separate `Environment`s in one thread).
pub(crate) fn run_tcp_repl_sexpr(
    port: u16,
    core_lib: &'static str,
    allowed: Vec<String>,
    contract_major: f64,
    contract_minor: f64,
) {
    let listener = match TcpListener::bind((Ipv4Addr::LOCALHOST, port)) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("Error: could not bind TCP REPL to 127.0.0.1:{port}: {err}");
            process::exit(1);
        }
    };
    let actual_port = listener.local_addr().map(|a| a.port()).unwrap_or(port);
    eprintln!(
        "my-lisp TCP REPL v{} (sexpr protocol) listening on 127.0.0.1:{actual_port}",
        env!("CARGO_PKG_VERSION")
    );

    let allowed = Arc::new(allowed);
    let mailbox: Arc<Mutex<MailboxState>> = Arc::new(Mutex::new(MailboxState::default()));
    let broker: Arc<Mutex<Broker>> = Arc::new(Mutex::new(Broker::default()));
    let claims: Arc<Mutex<ClaimTable>> = Arc::new(Mutex::new(ClaimTable::default()));
    let presence: Arc<Mutex<PresenceTable>> = Arc::new(Mutex::new(PresenceTable::default()));
    let tasks: Arc<Mutex<TaskTable>> = Arc::new(Mutex::new(TaskTable::default()));

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(_) => continue,
        };
        let allowed = Arc::clone(&allowed);
        let mailbox = Arc::clone(&mailbox);
        let broker = Arc::clone(&broker);
        let claims = Arc::clone(&claims);
        let presence = Arc::clone(&presence);
        let tasks = Arc::clone(&tasks);
        // Deep my-lisp recursion (e.g. (fact 1000)) consumes native stack
        // through the tree-walking evaluator; the default 2 MiB thread
        // stack overflowed and ABORTED the whole shared oracle process
        // (found 2026-08-25 by wsl-nidana-1/wsl-ganaka-1). Give every
        // connection its own generous stack instead.
        thread::Builder::new()
            .stack_size(256 * 1024 * 1024)
            .spawn(move || {
                handle_sexpr_connection(
                    stream,
                    core_lib,
                    &allowed,
                    contract_major,
                    contract_minor,
                    &mailbox,
                    &broker,
                    &claims,
                    &presence,
                    &tasks,
                );
            })
            .expect("failed to spawn TCP connection thread");
    }
}

#[allow(clippy::too_many_arguments)] // wire-protocol handler: every table is a distinct input
fn handle_sexpr_connection(
    mut stream: TcpStream,
    core_lib: &str,
    allowed: &[String],
    contract_major: f64,
    contract_minor: f64,
    mailbox: &Arc<Mutex<MailboxState>>,
    broker: &Arc<Mutex<Broker>>,
    claims: &Arc<Mutex<ClaimTable>>,
    presence: &Arc<Mutex<PresenceTable>>,
    tasks: &Arc<Mutex<TaskTable>>,
) {
    const MAX_REQUEST_LINE_BYTES: usize = 256 * 1024;
    let contract_version = Value::list([
        Value::Number(contract_major, Exactness::Exact),
        Value::Number(contract_minor, Exactness::Exact),
    ]);

    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "?".into());
    eprintln!("TCP REPL: connection from {peer}");

    // The oracle protocol is an unauthenticated network boundary. Keep its
    // process policy explicit even though trusted native root sessions are
    // unrestricted by default.
    let environment = Environment::root().with_process_allowlist(allowed.to_vec());
    let mut session = Session { environment };
    if let Ok(core_ast) = parse(core_lib) {
        let _ = eval_parsed_expressions(&core_ast, &mut session);
    }

    let mut reader = BufReader::new(stream.try_clone().expect("clone TCP stream"));
    loop {
        let frame = match read_request_frame(&mut reader, MAX_REQUEST_LINE_BYTES) {
            Ok(frame) => frame,
            Err(error) => {
                let resp = error_response(
                    &Value::Nil,
                    "invalid-encoding",
                    &format!("request is not valid UTF-8: {error}"),
                    &contract_version,
                );
                let _ = writeln!(stream, "{resp}");
                break;
            }
        };
        let line = match frame {
            RequestFrame::Eof => break,
            RequestFrame::TooLarge => {
                let resp = error_response(
                    &Value::Nil,
                    "request-too-large",
                    &format!("request line exceeds {MAX_REQUEST_LINE_BYTES} bytes"),
                    &contract_version,
                );
                let _ = writeln!(stream, "{resp}");
                break;
            }
            RequestFrame::Line(bytes) => match String::from_utf8(bytes) {
                Ok(line) => line,
                Err(error) => {
                    let resp = error_response(
                        &Value::Nil,
                        "invalid-encoding",
                        &format!("request is not valid UTF-8: {error}"),
                        &contract_version,
                    );
                    let _ = writeln!(stream, "{resp}");
                    break;
                }
            },
        };
        let trimmed = line.trim();
        {
            if trimmed.is_empty() {
                continue;
            }
            eprintln!("TCP REPL: {peer} > request-bytes={}", line.len());

            // The request envelope itself is read as literal data
            // (`quote`), never evaluated — `(op eval)` deciding to
            // evaluate `source` is the only place code ever runs.
            let quoted = format!("(quote {trimmed})");
            let request = match parse(&quoted).ok().and_then(|ast| {
                eval_parsed_expressions_incremental(&ast, &mut session)
                    .ok()
                    .map(|r| r.value)
            }) {
                Some(value) => value,
                None => {
                    let resp = error_response(
                        &Value::Nil,
                        "parse-error",
                        "request envelope is not a valid s-expression",
                        &contract_version,
                    );
                    let _ = writeln!(stream, "{resp}");
                    continue;
                }
            };

            let fields = list_items(&request);
            // fields[0] is the `request` tag symbol itself.
            let mut id = Value::Nil;
            let mut id_seen = false;
            let mut op: Option<String> = None;
            let mut source: Option<String> = None;
            let mut from: Option<String> = None;
            let mut to: Option<String> = None;
            let mut message: Option<String> = None;
            let mut for_agent: Option<String> = None;
            let mut since: u64 = 0;
            let mut topic: Option<String> = None;
            let mut topics: Vec<String> = Vec::new();
            let mut task: Option<String> = None;
            let mut project: Option<String> = None;
            let mut capabilities: Vec<String> = Vec::new();
            let mut priority: Option<f64> = None;
            let mut depends_on: Vec<String> = Vec::new();
            let mut needs: Option<String> = None;
            let mut context: Option<String> = None;
            let mut description: Option<String> = None;
            let mut file: Option<String> = None;
            for field in fields.iter().skip(1) {
                let kv = list_items(field);
                let (Some(key), Some(val)) = (kv.first(), kv.get(1)) else {
                    continue;
                };
                if let Value::Symbol(name) = key {
                    match &**name {
                        "id" => {
                            id_seen = true;
                            id = val.clone();
                        }
                        "op" => {
                            if let Value::Symbol(s) = val {
                                op = Some(s.to_string());
                            }
                        }
                        "source" => {
                            if let Value::String(s) = val {
                                source = Some(s.to_string());
                            }
                        }
                        "from" => {
                            if let Value::String(s) = val {
                                from = Some(s.to_string());
                            }
                        }
                        "to" => {
                            if let Value::String(s) = val {
                                to = Some(s.to_string());
                            }
                        }
                        "message" => {
                            if let Value::String(s) = val {
                                message = Some(s.to_string());
                            }
                        }
                        "for" => {
                            if let Value::String(s) = val {
                                for_agent = Some(s.to_string());
                            }
                        }
                        "since" => {
                            if let Value::Number(n, _) = val {
                                since = *n as u64;
                            }
                        }
                        "topic" => {
                            if let Value::String(s) = val {
                                topic = Some(s.to_string());
                            }
                        }
                        "topics" => {
                            topics = list_items(val)
                                .into_iter()
                                .filter_map(|item| match &item {
                                    Value::String(s) => Some(s.to_string()),
                                    Value::Symbol(s) => Some(s.to_string()),
                                    _ => None,
                                })
                                .collect();
                        }
                        "task" => {
                            if let Value::String(s) = val {
                                task = Some(s.to_string());
                            }
                        }
                        "project" => {
                            if let Value::String(s) = val {
                                project = Some(s.to_string());
                            }
                        }
                        "capabilities" => {
                            capabilities = list_items(val)
                                .into_iter()
                                .filter_map(|item| match &item {
                                    Value::String(s) => Some(s.to_string()),
                                    Value::Symbol(s) => Some(s.to_string()),
                                    _ => None,
                                })
                                .collect();
                        }
                        "priority" => {
                            if let Value::Number(n, _) = val {
                                priority = Some(*n);
                            }
                        }
                        "depends-on" => {
                            depends_on = list_items(val)
                                .into_iter()
                                .filter_map(|item| match &item {
                                    Value::String(s) => Some(s.to_string()),
                                    Value::Symbol(s) => Some(s.to_string()),
                                    _ => None,
                                })
                                .collect();
                        }
                        "needs" => {
                            needs = match val {
                                Value::String(s) => Some(s.to_string()),
                                Value::Symbol(s) => Some(s.to_string()),
                                _ => None,
                            };
                        }
                        "context" => {
                            if let Value::String(s) = val {
                                context = Some(s.to_string());
                            }
                        }
                        "description" => {
                            if let Value::String(s) = val {
                                description = Some(s.to_string());
                            }
                        }
                        "file" => {
                            if let Value::String(s) = val {
                                file = Some(s.to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }

            if !id_seen || matches!(id, Value::Nil) {
                let response = protocol_error(
                    "missing-id",
                    "request requires a non-nil id",
                    &contract_version,
                );
                let _ = writeln!(stream, "{response}");
                continue;
            }

            let response = match op.as_deref() {
                    Some("contract-version") => ok_response(&id, contract_version.clone(), &[], &contract_version),
                    // `parse` renders the canonical structure via the same
                    // `quote`-and-print path the request envelope itself
                    // uses, not Rust's `{:?}` — the caller gets my-lisp
                    // syntax back, not this CLI's internal AST debug
                    // format. Limited to a single top-level form, the
                    // same arity `quote` itself has.
                    Some("parse") => match &source {
                        None => error_response(&id, "invalid-form", "op `parse` requires a `source` field", &contract_version),
                        Some(src) => match parse(src) {
                            Ok(ast) if ast.len() == 1 => {
                                let quoted_src = format!("(quote {src})");
                                match parse(&quoted_src).ok().and_then(|q| {
                                    eval_parsed_expressions_incremental(&q, &mut session).ok().map(|r| r.value)
                                }) {
                                    Some(structure) => ok_response(&id, structure, &[], &contract_version),
                                    None => error_response(&id, "parse-error", "source parsed but could not be rendered as data", &contract_version),
                                }
                            }
                            Ok(_) => error_response(&id, "invalid-form", "op `parse` accepts exactly one top-level form", &contract_version),
                            Err(e) => error_response_classified(&id, error_kind_symbol(&e.kind), e.kind.classification(), &e.message, &contract_version),
                        },
                    },
                    // Agent-facing syntax preflight. Unlike `oracle-eval`,
                    // this never evaluates the submitted program or invokes
                    // capabilities. It preserves canonical parser spans and
                    // gives a conservative suggested edit for mismatched
                    // parentheses.
                    Some("oracle-check") => match &source {
                        None => error_response(&id, "invalid-form", "op `oracle-check` requires a `source` field", &contract_version),
                        Some(src) => ok_response(&id, oracle_check(src, &contract_version).0, &[], &contract_version),
                    },
                    Some("oracle-help") => match oracle_help(&mut session, topic.as_deref()) {
                        Ok(reference) => ok_response(&id, reference, &[], &contract_version),
                        Err(message) => error_response(&id, "invalid-form", &message, &contract_version),
                    },
                    Some("oracle-eval") => match &source {
                        None => error_response(&id, "invalid-form", "op `oracle-eval` requires a `source` field", &contract_version),
                        Some(src) => {
                            let result = match parse(src) {
                                Ok(ast) => match eval_parsed_expressions(&ast, &mut session) {
                                    Ok(result) => oracle_result_value(
                                        src,
                                        &contract_version,
                                        Value::Symbol("value".into()),
                                        Value::list([Value::Symbol("value".into()), result.value]),
                                        &result.output,
                                        "oracle-evaluation",
                                    ),
                                    Err(error) => oracle_result_value(
                                        src,
                                        &contract_version,
                                        Value::Symbol("error".into()),
                                        oracle_error_payload(src, &error),
                                        &[],
                                        "oracle-evaluation",
                                    ),
                                },
                                Err(error) => oracle_result_value(
                                    src,
                                    &contract_version,
                                    Value::Symbol("error".into()),
                                    oracle_error_payload(src, &error),
                                    &[],
                                    "oracle-evaluation",
                                ),
                            };
                            ok_response(&id, result, &[], &contract_version)
                        }
                    },
                    Some(op_name @ ("eval" | "diagnose")) => match &source {
                        None => error_response(&id, "invalid-form", &format!("op `{op_name}` requires a `source` field"), &contract_version),
                        Some(src) => match parse(src) {
                            Ok(ast) => match eval_parsed_expressions(&ast, &mut session) {
                                Ok(result) => ok_response(&id, result.value, &result.output, &contract_version),
                                Err(e) => error_response_classified(&id, error_kind_symbol(&e.kind), e.kind.classification(), &e.message, &contract_version),
                            },
                            Err(e) => error_response_classified(&id, error_kind_symbol(&e.kind), e.kind.classification(), &e.message, &contract_version),
                        },
                    },
                    Some("notify") => match (&from, &message) {
                        (None, _) => error_response(&id, "invalid-form", "op `notify` requires a `from` field", &contract_version),
                        (_, None) => error_response(&id, "invalid-form", "op `notify` requires a `message` field", &contract_version),
                        (Some(from), Some(message)) => {
                            let mut state = mailbox.lock().unwrap_or_else(|e| e.into_inner());
                            state.next_id += 1;
                            let entry_id = state.next_id;
                            state.entries.push(MailboxEntry {
                                id: entry_id,
                                from: from.clone(),
                                to: to.clone(),
                                message: message.clone(),
                            });
                            // Bounded so a long-lived server (or a
                            // runaway notifier) can't grow this
                            // in-memory, non-persistent mailbox
                            // without limit — oldest entries are
                            // dropped first; a `poll` with `since`
                            // older than what's left just gets
                            // whatever's still here.
                            const MAILBOX_CAPACITY: usize = 500;
                            if state.entries.len() > MAILBOX_CAPACITY {
                                let excess = state.entries.len() - MAILBOX_CAPACITY;
                                state.entries.drain(0..excess);
                            }
                            ok_response(&id, Value::Number(entry_id as f64, Exactness::Exact), &[], &contract_version)
                        }
                    },
                    Some("poll") => match &for_agent {
                        None => error_response(&id, "invalid-form", "op `poll` requires a `for` field", &contract_version),
                        Some(for_agent) => {
                            let state = mailbox.lock().unwrap_or_else(|e| e.into_inner());
                            let matches: Vec<Value> = state
                                .entries
                                .iter()
                                .filter(|entry| entry.id > since)
                                .filter(|entry| entry.to.as_deref().is_none_or(|to| to == for_agent))
                                .map(mailbox_entry_to_value)
                                .collect();
                            ok_response(&id, Value::list(matches), &[], &contract_version)
                        }
                    },
                    // `publish` delivers to every `subscribe`d connection
                    // whose `topics` is empty (subscribed to everything)
                    // or contains this `topic`, then responds with how
                    // many actually received it — visibility into
                    // whether anyone was listening, not just an ack.
                    Some("publish") => match (&from, &topic, &message) {
                        (None, _, _) => error_response(&id, "invalid-form", "op `publish` requires a `from` field", &contract_version),
                        (_, None, _) => error_response(&id, "invalid-form", "op `publish` requires a `topic` field", &contract_version),
                        (_, _, None) => error_response(&id, "invalid-form", "op `publish` requires a `message` field", &contract_version),
                        (Some(from), Some(topic), Some(message)) => {
                            let delivered = broadcast_event(broker, from, topic, message);
                            ok_response(&id, Value::Number(delivered as f64, Exactness::Exact), &[], &contract_version)
                        }
                    },
                    // `subscribe` permanently turns this connection into a
                    // push receiver: after the ack below, it stops reading
                    // further requests and instead blocks on its channel,
                    // writing each matching `publish` as an `(event ...)`
                    // line the instant it arrives. One connection, one
                    // purpose — a client that also wants to `eval`/`notify`
                    // opens a second connection for that, the same way a
                    // real pub/sub client library keeps publish and
                    // subscribe on separate sockets.
                    //
                    // `since` (an event id, default 0) replays everything
                    // matching `topics` from `Broker::event_log` before
                    // switching to live delivery — a reconnecting agent
                    // that remembers the last event id it saw doesn't lose
                    // whatever happened while its connection was down.
                    // The replay snapshot and the live-subscriber
                    // registration happen under the same lock acquisition
                    // (below), so there's no gap an event could fall
                    // through: anything logged before this point is in
                    // the replay list, anything logged after is delivered
                    // live, and `broadcast_event` itself only ever logs
                    // and delivers under that identical lock.
                    Some("subscribe") => {
                        let (sender, receiver) = mpsc::channel::<String>();
                        let (subscriber_id, replay) = {
                            let mut broker_state = broker.lock().unwrap_or_else(|e| e.into_inner());
                            let replay: Vec<String> = broker_state
                                .event_log
                                .iter()
                                .filter(|event| event.id > since)
                                .filter(|event| topics.is_empty() || topics.iter().any(|t| t == &event.topic))
                                .map(stored_event_to_text)
                                .collect();
                            broker_state.next_subscriber_id += 1;
                            let subscriber_id = broker_state.next_subscriber_id;
                            broker_state.subscribers.push(Subscriber {
                                id: subscriber_id,
                                topics: topics.clone(),
                                sender,
                            });
                            (subscriber_id, replay)
                        };
                        let ack = ok_response(&id, Value::Symbol("subscribed".into()), &[], &contract_version);
                        if writeln!(stream, "{ack}").is_err() {
                            let mut broker_state = broker.lock().unwrap_or_else(|e| e.into_inner());
                            broker_state.subscribers.retain(|s| s.id != subscriber_id);
                            break;
                        }
                        let mut replay_failed = false;
                        for event_text in &replay {
                            if writeln!(stream, "{event_text}").is_err() {
                                replay_failed = true;
                                break;
                            }
                        }
                        if replay_failed {
                            let mut broker_state = broker.lock().unwrap_or_else(|e| e.into_inner());
                            broker_state.subscribers.retain(|s| s.id != subscriber_id);
                            break;
                        }
                        eprintln!("TCP REPL: {peer} subscribed to {topics:?} (replayed {} missed events since {since}), switching to push mode", replay.len());
                        for event_text in receiver.iter() {
                            if writeln!(stream, "{event_text}").is_err() {
                                break;
                            }
                        }
                        let mut broker_state = broker.lock().unwrap_or_else(|e| e.into_inner());
                        broker_state.subscribers.retain(|s| s.id != subscriber_id);
                        break;
                    }
                    // Compare-and-swap under one lock acquisition: a
                    // `claim` only succeeds if `task` has no holder yet,
                    // or `from` already holds it (idempotent re-claim), or
                    // the current holder's `presence` heartbeat is stale
                    // past `STALE_CLAIM_SECS` — a claim held by an agent
                    // that's gone quiet shouldn't block the task forever.
                    // `None` in `presence` (holder never called `hello`)
                    // is deliberately treated as "can't tell, don't steal"
                    // rather than "stale" — reclaim is an opt-in safety
                    // net for agents that heartbeat, not a way to bypass
                    // the claim guarantee for agents that never registered.
                    // Two agents racing for the same task can still never
                    // both see success. `value` is `t` on success, or the
                    // current (still-live) holder's name if someone else
                    // already has it, so the loser knows who to wait on or
                    // `publish` a `need` at.
                    Some("claim") => match (&task, &from) {
                        (None, _) => error_response(&id, "invalid-form", "op `claim` requires a `task` field", &contract_version),
                        (_, None) => error_response(&id, "invalid-form", "op `claim` requires a `from` field", &contract_version),
                        (Some(task), Some(from)) => {
                            const STALE_CLAIM_SECS: f64 = 300.0;
                            let mut claim_state = claims.lock().unwrap_or_else(|e| e.into_inner());
                            match claim_state.holders.get(task).cloned() {
                                Some(holder) if &holder == from => ok_response(&id, Value::Bool(true), &[], &contract_version),
                                Some(holder) => {
                                    let is_stale = {
                                        let presence_state = presence.lock().unwrap_or_else(|e| e.into_inner());
                                        presence_state
                                            .agents
                                            .get(&holder)
                                            .map(|entry| entry.last_seen.elapsed().as_secs_f64() > STALE_CLAIM_SECS)
                                            .unwrap_or(false)
                                    };
                                    if is_stale {
                                        claim_state.holders.insert(task.clone(), from.clone());
                                        drop(claim_state);
                                        broadcast_event(
                                            broker,
                                            from,
                                            "claim-stale-reclaimed",
                                            &format!("{from} reclaimed {task} from stale holder {holder} (no heartbeat for over {STALE_CLAIM_SECS}s)"),
                                        );
                                        ok_response(&id, Value::Bool(true), &[], &contract_version)
                                    } else {
                                        ok_response(&id, Value::String(holder.as_str().into()), &[], &contract_version)
                                    }
                                }
                                None => {
                                    claim_state.holders.insert(task.clone(), from.clone());
                                    drop(claim_state);
                                    broadcast_event(broker, from, "claim-taken", &format!("{from} claimed {task}"));
                                    ok_response(&id, Value::Bool(true), &[], &contract_version)
                                }
                            }
                        }
                    },
                    // Only the current holder can release; releasing an
                    // unclaimed or already-your-own-released task is a
                    // no-op success (idempotent), same spirit as `claim`.
                    Some("release") => match (&task, &from) {
                        (None, _) => error_response(&id, "invalid-form", "op `release` requires a `task` field", &contract_version),
                        (_, None) => error_response(&id, "invalid-form", "op `release` requires a `from` field", &contract_version),
                        (Some(task), Some(from)) => {
                            let mut claim_state = claims.lock().unwrap_or_else(|e| e.into_inner());
                            match claim_state.holders.get(task) {
                                Some(holder) if holder != from => ok_response(&id, Value::String(holder.as_str().into()), &[], &contract_version),
                                _ => {
                                    let was_held = claim_state.holders.remove(task).is_some();
                                    drop(claim_state);
                                    if was_held {
                                        broadcast_event(broker, from, "claim-released", &format!("{from} released {task}"));
                                    }
                                    ok_response(&id, Value::Bool(true), &[], &contract_version)
                                }
                            }
                        }
                    },
                    // Read-only: every currently-held claim, so an agent
                    // computing its own next-best-action can see what's
                    // already spoken for before claiming.
                    Some("list-claims") => {
                        let claim_state = claims.lock().unwrap_or_else(|e| e.into_inner());
                        let task_state = tasks.lock().unwrap_or_else(|e| e.into_inner());
                        let entries: Vec<Value> = claim_state
                            .holders
                            .iter()
                            .map(|(task, holder)| {
                                let description = task_state.tasks.get(task).and_then(|def| def.description.clone());
                                Value::list([
                                    Value::list([Value::Symbol("task".into()), Value::String(task.as_str().into())]),
                                    Value::list([Value::Symbol("agent".into()), Value::String(holder.as_str().into())]),
                                    Value::list([
                                        Value::Symbol("description".into()),
                                        match description {
                                            Some(d) => Value::String(d.as_str().into()),
                                            None => Value::Nil,
                                        },
                                    ]),
                                ])
                            })
                            .collect();
                        ok_response(&id, Value::list(entries), &[], &contract_version)
                    }
                    // Full, unfiltered dump of every `define-task`d task —
                    // the debugging counterpart to `next-best-action`,
                    // which deliberately hides anything excluded (wrong
                    // capability, unmet dependency, already claimed by
                    // someone else, already done). Without this there's
                    // no way to tell "the task doesn't exist" from "it
                    // exists but next-best-action filtered it for a
                    // reason" — direct feedback this ambiguity cost real
                    // debugging time. Includes `done` and, for each still
                    // held, the current holder — cross-referencing
                    // `list-claims` shouldn't be necessary just to see
                    // the whole board.
                    Some("list-tasks") => {
                        let task_state = tasks.lock().unwrap_or_else(|e| e.into_inner());
                        let claim_state = claims.lock().unwrap_or_else(|e| e.into_inner());
                        let entries: Vec<Value> = task_state
                            .tasks
                            .iter()
                            .map(|(task_id, def)| {
                                Value::list([
                                    Value::list([Value::Symbol("task".into()), Value::String(task_id.as_str().into())]),
                                    Value::list([Value::Symbol("priority".into()), Value::Number(def.priority, Exactness::Inexact)]),
                                    Value::list([
                                        Value::Symbol("capabilities".into()),
                                        Value::list(def.capabilities.iter().map(|c| Value::Symbol(c.as_str().into()))),
                                    ]),
                                    Value::list([
                                        Value::Symbol("depends-on".into()),
                                        Value::list(def.depends_on.iter().map(|d| Value::String(d.as_str().into()))),
                                    ]),
                                    Value::list([Value::Symbol("done".into()), Value::Bool(def.done)]),
                                    Value::list([
                                        Value::Symbol("claimed-by".into()),
                                        match claim_state.holders.get(task_id) {
                                            Some(holder) => Value::String(holder.as_str().into()),
                                            None => Value::Nil,
                                        },
                                    ]),
                                    Value::list([
                                        Value::Symbol("description".into()),
                                        match &def.description {
                                            Some(d) => Value::String(d.as_str().into()),
                                            None => Value::Nil,
                                        },
                                    ]),
                                ])
                            })
                            .collect();
                        ok_response(&id, Value::list(entries), &[], &contract_version)
                    }
                    // `hello`/`heartbeat` both write the same table —
                    // `hello` is just the first heartbeat, with an
                    // optional `project`/`capabilities` attached. Neither
                    // requires the other to have been called first,
                    // deliberately: an agent that only ever calls
                    // `heartbeat` still shows up in `presence`, just
                    // without capability info until it sends a `hello`.
                    Some(op_name @ ("hello" | "heartbeat")) => match &from {
                        None => error_response(&id, "invalid-form", &format!("op `{op_name}` requires a `from` field"), &contract_version),
                        Some(from) => {
                            let mut presence_state = presence.lock().unwrap_or_else(|e| e.into_inner());
                            let is_new_agent = !presence_state.agents.contains_key(from);
                            let entry = presence_state.agents.entry(from.clone()).or_insert_with(|| PresenceEntry {
                                project: None,
                                capabilities: Vec::new(),
                                task: None,
                                last_seen: std::time::Instant::now(),
                            });
                            entry.last_seen = std::time::Instant::now();
                            if let Some(task) = &task {
                                entry.task = Some(task.clone());
                            }
                            if op_name == "hello" {
                                if project.is_some() {
                                    entry.project = project.clone();
                                }
                                if !capabilities.is_empty() {
                                    entry.capabilities = capabilities.clone();
                                }
                            }
                            let peers: Vec<Value> = presence_state
                                .agents
                                .iter()
                                .filter(|(agent, _)| *agent != from)
                                .map(|(agent, entry)| presence_entry_to_value(agent, entry))
                                .collect();
                            drop(presence_state);
                            if op_name == "hello" && is_new_agent {
                                broadcast_event(broker, from, "agent-joined", &format!("{from} joined"));
                            }
                            ok_response(&id, Value::list(peers), &[], &contract_version)
                        }
                    },
                    // Read-only snapshot of every registered agent,
                    // including staleness (`seconds-since-heartbeat`) so
                    // the caller judges liveness itself rather than the
                    // server silently evicting anyone.
                    Some("presence") => {
                        let presence_state = presence.lock().unwrap_or_else(|e| e.into_inner());
                        let entries: Vec<Value> = presence_state
                            .agents
                            .iter()
                            .map(|(agent, entry)| presence_entry_to_value(agent, entry))
                            .collect();
                        ok_response(&id, Value::list(entries), &[], &contract_version)
                    }
                    // Registers or redefines a task's scoring inputs.
                    // Redefining an existing task keeps its `done` status
                    // (changing priority/capabilities/deps shouldn't
                    // un-complete it) — only `complete-task` sets `done`.
                    Some("define-task") => match &task {
                        None => error_response(&id, "invalid-form", "op `define-task` requires a `task` field", &contract_version),
                        Some(task) => {
                            let mut task_state = tasks.lock().unwrap_or_else(|e| e.into_inner());
                            let is_new_task = !task_state.tasks.contains_key(task);
                            let done = task_state.tasks.get(task).map(|t| t.done).unwrap_or(false);
                            let existing_description = task_state.tasks.get(task).and_then(|t| t.description.clone());
                            task_state.tasks.insert(task.clone(), TaskDef {
                                priority: priority.unwrap_or(1.0),
                                capabilities: capabilities.clone(),
                                depends_on: depends_on.clone(),
                                done,
                                description: description.clone().or(existing_description),
                            });
                            drop(task_state);
                            if is_new_task {
                                let publisher = from.clone().unwrap_or_else(|| "unknown".to_string());
                                broadcast_event(broker, &publisher, "task-created", &format!("task {task} defined"));
                            }
                            ok_response(&id, Value::Bool(true), &[], &contract_version)
                        }
                    },
                    // Marks a task done and drops its claim (if any) —
                    // deliberately does NOT require the caller to be the
                    // current holder: a task can legitimately get
                    // completed by someone other than whoever claimed it
                    // (a handoff), and this registry is a coordination
                    // hint, not an access-control system. The durable
                    // "who actually did it" record still belongs in
                    // evidence/, same as everything else here.
                    Some("complete-task") => match &task {
                        None => error_response(&id, "invalid-form", "op `complete-task` requires a `task` field", &contract_version),
                        Some(task) => {
                            let mut task_state = tasks.lock().unwrap_or_else(|e| e.into_inner());
                            match task_state.tasks.get_mut(task) {
                                None => error_response(&id, "invalid-form", &format!("no such task `{task}` — define-task it first"), &contract_version),
                                Some(def) => {
                                    def.done = true;
                                    let mut claim_state = claims.lock().unwrap_or_else(|e| e.into_inner());
                                    claim_state.holders.remove(task);
                                    ok_response(&id, Value::Bool(true), &[], &contract_version)
                                }
                            }
                        }
                    },
                    // Dry-run of `sync-tasks` (below): same file, same
                    // parsing, same per-entry checks, but never touches
                    // the task registry — no lock acquisition on `tasks`,
                    // no `task-created` events. Written because a
                    // malformed `tasks.my` (an extra paren, a typo'd key)
                    // used to be found by trial and error against the
                    // live registry; this reports a top-level parse
                    // error's exact 1-indexed `(line . N) (column . N)`
                    // (from the same `LanguageError` `span` `sync-tasks`
                    // already gets, just not previously surfaced as a
                    // position) plus every per-entry warning `sync-tasks`
                    // would have produced, so a syntax mistake is visible
                    // before it ever reaches the shared server-wide state.
                    Some("validate-tasks") => match &file {
                        None => error_response(&id, "invalid-form", "op `validate-tasks` requires a `file` field", &contract_version),
                        Some(path) if require_absolute_path(path).is_err() => {
                            error_response(&id, "invalid-form", &require_absolute_path(path).unwrap_err(), &contract_version)
                        }
                        Some(path) => match fs::read_to_string(path) {
                            Err(e) => error_response(&id, "io-error", &format!("cannot read `{path}`: {e}"), &contract_version),
                            Ok(content) => match parse(&content) {
                                Err(e) => {
                                    let (line, column) = line_col_at(&content, e.span.start);
                                    error_response_classified(
                                        &id,
                                        error_kind_symbol(&e.kind),
                                        e.kind.classification(),
                                        &format!("{} (line {line}, column {column})", e.message),
                                        &contract_version,
                                    )
                                }
                                Ok(ast) if ast.len() != 1 => error_response(
                                    &id,
                                    "invalid-form",
                                    "op `validate-tasks` expects a single top-level form (one flat alist) in the file",
                                    &contract_version,
                                ),
                                Ok(_) => {
                                    let quoted_file = format!("(quote {content})");
                                    let rendered = parse(&quoted_file).ok().and_then(|q| {
                                        eval_parsed_expressions_incremental(&q, &mut session).ok().map(|r| r.value)
                                    });
                                    match rendered {
                                        None => error_response(
                                            &id,
                                            "parse-error",
                                            "file parsed but could not be rendered as data",
                                            &contract_version,
                                        ),
                                        Some(file_value) => match dotted_alist_lookup(&file_value, "tasks") {
                                            None => error_response(
                                                &id,
                                                "invalid-form",
                                                "validate-tasks file must contain a `(tasks . ...)` alist key",
                                                &contract_version,
                                            ),
                                            Some(tasks_value) => {
                                                let mut would_define: Vec<String> = Vec::new();
                                                let mut warnings: Vec<String> = Vec::new();
                                                for entry in list_items(&tasks_value) {
                                                    let Value::Pair(task_id_value, props_value) = &entry else {
                                                        warnings.push("a task entry is not a dotted pair (task-id . props)".to_string());
                                                        continue;
                                                    };
                                                    let Some(task_id) = atom_string(task_id_value) else {
                                                        warnings.push("a task id is neither a string nor a symbol".to_string());
                                                        continue;
                                                    };
                                                    if task_id.is_empty() {
                                                        warnings.push("a task id is empty".to_string());
                                                        continue;
                                                    }
                                                    if let Some(priority_value) = dotted_alist_lookup(props_value, "priority") {
                                                        if !matches!(priority_value, Value::Number(_, _)) {
                                                            warnings.push(format!("task `{task_id}`: `priority` is not a number"));
                                                        }
                                                    }
                                                    would_define.push(task_id);
                                                }
                                                let value = Value::list([
                                                    Value::list([
                                                        Value::Symbol("would-define".into()),
                                                        Value::list(would_define.into_iter().map(|task_id| Value::String(task_id.as_str().into()))),
                                                    ]),
                                                    Value::list([
                                                        Value::Symbol("warnings".into()),
                                                        Value::list(warnings.into_iter().map(|warning| Value::String(warning.as_str().into()))),
                                                    ]),
                                                ]);
                                                ok_response(&id, value, &[], &contract_version)
                                            }
                                        },
                                    }
                                }
                            },
                        },
                    },
                    // Imports the durable task plan from a `tasks.my`
                    // flat-alist file (same data convention as
                    // ecosystem-status.my): `((kind . tasks-my) (tasks .
                    // (("TASK-ID" . ((priority . 0.8) (capabilities .
                    // (compiler rust)) (depends-on . ("OTHER")) (done . ()))))
                    // ...)))`. Upsert — defines or redefines each listed
                    // task, preserving `done` unless the file says
                    // otherwise; tasks *not* listed are left alone (so
                    // re-syncing can't clobber auto-created `HELP:...`
                    // tasks or the in-memory claims). This is the bridge
                    // between the durable plan (git-tracked files) and the
                    // in-memory registry `next-best-action` scores, and the
                    // fix for the restart-loss the AGENTS.md durability
                    // rule warns about: after a server restart an agent
                    // re-runs `sync-tasks` and the plan is back. A file
                    // error fails the whole op; a malformed entry inside an
                    // otherwise valid file is skipped with a warning, so
                    // one typo doesn't silently drop the whole board.
                    Some("sync-tasks") => match &file {
                        None => error_response(&id, "invalid-form", "op `sync-tasks` requires a `file` field", &contract_version),
                        Some(path) if require_absolute_path(path).is_err() => {
                            error_response(&id, "invalid-form", &require_absolute_path(path).unwrap_err(), &contract_version)
                        }
                        Some(path) => match fs::read_to_string(path) {
                            Err(e) => error_response(&id, "io-error", &format!("cannot read `{path}`: {e}"), &contract_version),
                            Ok(content) => match parse(&content) {
                                Err(e) => error_response_classified(&id, error_kind_symbol(&e.kind), e.kind.classification(), &e.message, &contract_version),
                                Ok(ast) if ast.len() != 1 => error_response(
                                    &id,
                                    "invalid-form",
                                    "op `sync-tasks` expects a single top-level form (one flat alist) in the file",
                                    &contract_version,
                                ),
                                Ok(_) => {
                                    // `parse` yields `Expr`s, not `Value`s,
                                    // so the file's structure is turned into
                                    // data the same way the request envelope
                                    // and the `parse` op do: `quote` it and
                                    // evaluate — dotted pairs stay dotted.
                                    let quoted_file = format!("(quote {content})");
                                    let rendered = parse(&quoted_file).ok().and_then(|q| {
                                        eval_parsed_expressions_incremental(&q, &mut session).ok().map(|r| r.value)
                                    });
                                    match rendered {
                                        None => error_response(
                                            &id,
                                            "parse-error",
                                            "tasks file parsed but could not be rendered as data",
                                            &contract_version,
                                        ),
                                        Some(file_value) => match dotted_alist_lookup(&file_value, "tasks") {
                                            None => error_response(
                                                &id,
                                                "invalid-form",
                                                "sync-tasks file must contain a `(tasks . ...)` alist key",
                                                &contract_version,
                                            ),
                                            Some(tasks_value) => {
                                                let mut defined: Vec<String> = Vec::new();
                                                let mut warnings: Vec<String> = Vec::new();
                                                for entry in list_items(&tasks_value) {
                                                    let Value::Pair(task_id_value, props_value) = &entry else {
                                                        warnings.push("a task entry is not a dotted pair (task-id . props)".to_string());
                                                        continue;
                                                    };
                                                    let Some(task_id) = atom_string(task_id_value) else {
                                                        warnings.push("a task id is neither a string nor a symbol".to_string());
                                                        continue;
                                                    };
                                                    if task_id.is_empty() {
                                                        warnings.push("a task id is empty".to_string());
                                                        continue;
                                                    }
                                                    let mut priority = 1.0;
                                                    if let Some(priority_value) = dotted_alist_lookup(props_value, "priority") {
                                                        if let Value::Number(n, _) = &priority_value {
                                                            priority = *n;
                                                        } else {
                                                            warnings.push(format!("task `{task_id}`: `priority` is not a number"));
                                                        }
                                                    }
                                                    let capabilities = dotted_alist_lookup(props_value, "capabilities")
                                                        .map(|v| list_of_atoms(&v))
                                                        .unwrap_or_default();
                                                    let depends_on = dotted_alist_lookup(props_value, "depends-on")
                                                        .map(|v| list_of_atoms(&v))
                                                        .unwrap_or_default();
                                                    let file_done = dotted_alist_lookup(props_value, "done").and_then(|v| bool_from_value(&v));
                                                    let file_description = dotted_alist_lookup(props_value, "description").and_then(|v| atom_string(&v));
                                                    let (is_new_task, _existing_done) = {
                                                        let mut task_state = tasks.lock().unwrap_or_else(|e| e.into_inner());
                                                        let new = !task_state.tasks.contains_key(&task_id);
                                                        let existing_done = task_state.tasks.get(&task_id).map(|t| t.done).unwrap_or(false);
                                                        let existing_description = task_state.tasks.get(&task_id).and_then(|t| t.description.clone());
                                                        task_state.tasks.insert(task_id.clone(), TaskDef {
                                                            priority,
                                                            capabilities,
                                                            depends_on,
                                                            done: file_done.unwrap_or(existing_done),
                                                            description: file_description.or(existing_description),
                                                        });
                                                        (new, existing_done)
                                                    };
                                                    if is_new_task {
                                                        let publisher = from.clone().unwrap_or_else(|| "unknown".to_string());
                                                        broadcast_event(broker, &publisher, "task-created", &format!("task {task_id} defined via sync-tasks"));
                                                    }
                                                    defined.push(task_id);
                                                }
                                                let value = Value::list([
                                                    Value::list([
                                                        Value::Symbol("defined".into()),
                                                        Value::list(defined.into_iter().map(|task_id| Value::String(task_id.as_str().into()))),
                                                    ]),
                                                    Value::list([
                                                        Value::Symbol("warnings".into()),
                                                        Value::list(warnings.into_iter().map(|warning| Value::String(warning.as_str().into()))),
                                                    ]),
                                                ]);
                                                ok_response(&id, value, &[], &contract_version)
                                            }
                                        },
                                    }
                                }
                            },
                        },
                    },
                    // Auto-derives claimable tasks from
                    // `ecosystem-status.my`'s `next-milestone.per-repo`
                    // alist — the prose "what cml/fpga-lisp/my-idea
                    // should each do next" the file already carries by
                    // hand, turned into a `HELP:`-style task per repo
                    // without redundantly retyping it via `define-task`.
                    // One task per `per-repo` entry, id
                    // `MILESTONE:<name>:<repo>`, `capabilities` set to
                    // exactly `(repo)` — the convention this creates:
                    // include your own repo name in `hello`'s
                    // `capabilities` (e.g. `cml` declares `(compiler rust
                    // cml)`) so this task surfaces specifically to that
                    // repo's own agent, not everyone. Priority fixed at
                    // 5.0 — well above hand-defined tasks, since this is
                    // the ecosystem's one pinned current milestone, not
                    // routine work. The task registry only stores an id
                    // and scoring inputs, not the description itself
                    // (same shape `define-task` always had) — the
                    // `task-created` event's `message` carries the prose
                    // once, at creation, but `next-best-action` results
                    // are just ids; read `ecosystem-status.my` itself for
                    // the actual instructions, same as any other task
                    // requires reading its own definition somewhere.
                    Some("sync-milestone") => match &file {
                        None => error_response(&id, "invalid-form", "op `sync-milestone` requires a `file` field", &contract_version),
                        Some(path) if require_absolute_path(path).is_err() => {
                            error_response(&id, "invalid-form", &require_absolute_path(path).unwrap_err(), &contract_version)
                        }
                        Some(path) => match fs::read_to_string(path) {
                            Err(e) => error_response(&id, "io-error", &format!("cannot read `{path}`: {e}"), &contract_version),
                            Ok(content) => match parse(&content) {
                                Err(e) => error_response_classified(&id, error_kind_symbol(&e.kind), e.kind.classification(), &e.message, &contract_version),
                                Ok(ast) if ast.len() != 1 => error_response(
                                    &id,
                                    "invalid-form",
                                    "op `sync-milestone` expects a single top-level form (one flat alist) in the file",
                                    &contract_version,
                                ),
                                Ok(_) => {
                                    let quoted_file = format!("(quote {content})");
                                    let rendered = parse(&quoted_file).ok().and_then(|q| {
                                        eval_parsed_expressions_incremental(&q, &mut session).ok().map(|r| r.value)
                                    });
                                    match rendered {
                                        None => error_response(
                                            &id,
                                            "parse-error",
                                            "file parsed but could not be rendered as data",
                                            &contract_version,
                                        ),
                                        Some(file_value) => match dotted_alist_lookup(&file_value, "next-milestone") {
                                            None => error_response(
                                                &id,
                                                "invalid-form",
                                                "file must contain a `(next-milestone . ...)` alist key",
                                                &contract_version,
                                            ),
                                            Some(milestone_value) => {
                                                let milestone_name = dotted_alist_lookup(&milestone_value, "name")
                                                    .and_then(|v| atom_string(&v))
                                                    .unwrap_or_else(|| "milestone".to_string());
                                                let mut defined: Vec<String> = Vec::new();
                                                let mut warnings: Vec<String> = Vec::new();
                                                match dotted_alist_lookup(&milestone_value, "per-repo") {
                                                    None => warnings.push("next-milestone has no `per-repo` key".to_string()),
                                                    Some(per_repo_value) => {
                                                        for entry in list_items(&per_repo_value) {
                                                            let Value::Pair(repo_value, description_value) = &entry else {
                                                                warnings.push("a per-repo entry is not a dotted pair (repo . description)".to_string());
                                                                continue;
                                                            };
                                                            let Some(repo) = atom_string(repo_value) else {
                                                                warnings.push("a per-repo key is neither a string nor a symbol".to_string());
                                                                continue;
                                                            };
                                                            let description = description_value.to_string();
                                                            let task_id = format!("MILESTONE:{milestone_name}:{repo}");
                                                            let is_new_task = {
                                                                let mut task_state = tasks.lock().unwrap_or_else(|e| e.into_inner());
                                                                let new = !task_state.tasks.contains_key(&task_id);
                                                                let existing_done = task_state.tasks.get(&task_id).map(|t| t.done).unwrap_or(false);
                                                                task_state.tasks.insert(task_id.clone(), TaskDef {
                                                                    priority: 5.0,
                                                                    capabilities: vec![repo.clone()],
                                                                    depends_on: Vec::new(),
                                                                    done: existing_done,
                                                                    description: Some(description.clone()),
                                                                });
                                                                new
                                                            };
                                                            if is_new_task {
                                                                let publisher = from.clone().unwrap_or_else(|| "unknown".to_string());
                                                                broadcast_event(
                                                                    broker,
                                                                    &publisher,
                                                                    "task-created",
                                                                    &format!("task {task_id} defined via sync-milestone: {description}"),
                                                                );
                                                            }
                                                            defined.push(task_id);
                                                        }
                                                    }
                                                }
                                                let value = Value::list([
                                                    Value::list([Value::Symbol("milestone".into()), Value::String(milestone_name.as_str().into())]),
                                                    Value::list([
                                                        Value::Symbol("defined".into()),
                                                        Value::list(defined.into_iter().map(|t| Value::String(t.as_str().into()))),
                                                    ]),
                                                    Value::list([
                                                        Value::Symbol("warnings".into()),
                                                        Value::list(warnings.into_iter().map(|w| Value::String(w.as_str().into()))),
                                                    ]),
                                                ]);
                                                ok_response(&id, value, &[], &contract_version)
                                            }
                                        },
                                    }
                                }
                            },
                        },
                    },
                    // `score = priority × capability-match × (1 +
                    // unblock-impact)`, per docs/swarm-coordination.md.
                    // capability-match is a hard gate here, not a
                    // fraction: a task naming capabilities the caller
                    // doesn't have is excluded outright, not merely
                    // down-ranked — claiming work you can't actually do
                    // isn't a "lower-priority" outcome, it's not an
                    // option. unblock-impact counts how many *other*,
                    // not-yet-done tasks list this one in `depends-on` —
                    // finishing a task blocking 5 others outranks one
                    // blocking none, all else equal. A task with any
                    // unsatisfied dependency, already `done`, or already
                    // claimed by someone else is excluded entirely —
                    // it's not actionable yet or not available.
                    // `capabilities` may be passed explicitly; if not,
                    // falls back to whatever the caller's last `hello`
                    // registered in `presence`.
                    Some("next-best-action") => match &from {
                        None => error_response(&id, "invalid-form", "op `next-best-action` requires a `from` field", &contract_version),
                        Some(from) => {
                            let caller_capabilities: Vec<String> = if !capabilities.is_empty() {
                                capabilities.clone()
                            } else {
                                let presence_state = presence.lock().unwrap_or_else(|e| e.into_inner());
                                presence_state
                                    .agents
                                    .get(from)
                                    .map(|entry| entry.capabilities.clone())
                                    .unwrap_or_default()
                            };
                            let task_state = tasks.lock().unwrap_or_else(|e| e.into_inner());
                            let claim_state = claims.lock().unwrap_or_else(|e| e.into_inner());
                            let mut ranked: Vec<(String, f64, Option<String>)> = task_state
                                .tasks
                                .iter()
                                .filter(|(_, def)| !def.done)
                                .filter(|(task_id, _)| {
                                    claim_state.holders.get(*task_id).is_none_or(|holder| holder == from)
                                })
                                .filter(|(_, def)| {
                                    def.depends_on.iter().all(|dep| {
                                        task_state.tasks.get(dep).map(|d| d.done).unwrap_or(true)
                                    })
                                })
                                .filter(|(_, def)| {
                                    def.capabilities.is_empty()
                                        || def.capabilities.iter().all(|needed| caller_capabilities.contains(needed))
                                })
                                .map(|(task_id, def)| {
                                    let unblock_impact = task_state
                                        .tasks
                                        .values()
                                        .filter(|other| !other.done && other.depends_on.contains(task_id))
                                        .count() as f64;
                                    (task_id.clone(), def.priority * (1.0 + unblock_impact), def.description.clone())
                                })
                                .collect();
                            ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                            let value = Value::list(ranked.into_iter().map(|(task_id, score, description)| {
                                Value::list([
                                    Value::list([Value::Symbol("task".into()), Value::String(task_id.as_str().into())]),
                                    Value::list([Value::Symbol("score".into()), Value::Number(score, Exactness::Inexact)]),
                                    Value::list([
                                        Value::Symbol("description".into()),
                                        match description {
                                            Some(d) => Value::String(d.as_str().into()),
                                            None => Value::Nil,
                                        },
                                    ]),
                                ])
                            }));
                            ok_response(&id, value, &[], &contract_version)
                        }
                    },
                    // Forms a temporary coalition around a stuck agent's
                    // unmet need. Three things happen atomically from the
                    // caller's point of view: (1) every `presence`-
                    // registered agent whose `capabilities` include
                    // `needs` gets the request pushed instantly if
                    // they're `subscribe`d to the `capability-request`
                    // topic, and (2) gets it left in their `notify`
                    // mailbox regardless, so a non-subscribed agent
                    // still sees it on its next `poll`; (3) a task named
                    // `HELP:<needs>:<task-or-from>` is auto-`define-task`d
                    // at high priority requiring exactly `needs`, so it
                    // surfaces at the top of `next-best-action` for any
                    // agent with that capability — the "system sees fpga
                    // offers waveform-debug" step from the proposal,
                    // done via the scoring machinery already built
                    // rather than a separate matching engine.
                    Some("capability-request") => match (&from, &needs) {
                        (None, _) => error_response(&id, "invalid-form", "op `capability-request` requires a `from` field", &contract_version),
                        (_, None) => error_response(&id, "invalid-form", "op `capability-request` requires a `needs` field", &contract_version),
                        (Some(from), Some(needs)) => {
                            let matching: Vec<String> = {
                                let presence_state = presence.lock().unwrap_or_else(|e| e.into_inner());
                                presence_state
                                    .agents
                                    .iter()
                                    .filter(|(agent, entry)| *agent != from && entry.capabilities.iter().any(|c| c == needs))
                                    .map(|(agent, _)| agent.clone())
                                    .collect()
                            };

                            let task_ref = task.clone().unwrap_or_default();
                            let request_message = format!(
                                "capability-request from {from}: needs `{needs}`{}{}",
                                if task_ref.is_empty() { String::new() } else { format!(" for task {task_ref}") },
                                match &context {
                                    Some(c) if !c.is_empty() => format!(" — {c}"),
                                    _ => String::new(),
                                }
                            );

                            broadcast_event(broker, from, "capability-request", &request_message);

                            {
                                let mut mailbox_state = mailbox.lock().unwrap_or_else(|e| e.into_inner());
                                for agent in &matching {
                                    mailbox_state.next_id += 1;
                                    let entry_id = mailbox_state.next_id;
                                    mailbox_state.entries.push(MailboxEntry {
                                        id: entry_id,
                                        from: from.clone(),
                                        to: Some(agent.clone()),
                                        message: request_message.clone(),
                                    });
                                }
                                const MAILBOX_CAPACITY: usize = 500;
                                if mailbox_state.entries.len() > MAILBOX_CAPACITY {
                                    let excess = mailbox_state.entries.len() - MAILBOX_CAPACITY;
                                    mailbox_state.entries.drain(0..excess);
                                }
                            }

                            let elevated_task_id = format!(
                                "HELP:{needs}:{}",
                                if task_ref.is_empty() { from.clone() } else { task_ref.clone() }
                            );
                            {
                                let mut task_state = tasks.lock().unwrap_or_else(|e| e.into_inner());
                                let done = task_state.tasks.get(&elevated_task_id).map(|t| t.done).unwrap_or(false);
                                task_state.tasks.insert(elevated_task_id.clone(), TaskDef {
                                    priority: 10.0,
                                    capabilities: vec![needs.clone()],
                                    depends_on: Vec::new(),
                                    done,
                                    description: Some(request_message.clone()),
                                });
                            }

                            let value = Value::list([
                                Value::list([
                                    Value::Symbol("matching-agents".into()),
                                    Value::list(matching.iter().map(|a| Value::String(a.as_str().into()))),
                                ]),
                                Value::list([Value::Symbol("elevated-task".into()), Value::String(elevated_task_id.as_str().into())]),
                            ]);
                            ok_response(&id, value, &[], &contract_version)
                        }
                    },
                    Some(other) => error_response(&id, "invalid-form", &format!("unknown op `{other}`"), &contract_version),
                    None => error_response(&id, "invalid-form", "request is missing an `op` field", &contract_version),
                };

            if writeln!(stream, "{response}").is_err() {
                break;
            }
        }
    }
    eprintln!("TCP REPL: {peer} disconnected");
}

/// One-shot P2P client: connects to a peer's TCP REPL and forwards a
/// single sexpr request read from stdin, printing the response line to
/// stdout. This is the peer side of the mesh topology in
/// docs/swarm-autonomy.md — every agent runs its own server, and agents
/// talk to each other directly through this primitive, no shared hub:
/// `printf (quote %s)\n' '(request (op notify) (from "me") (to "you") (message "hi"))' |
///   my-lisp --connect=127.0.0.1:9992`.
pub(crate) fn run_client(address: &str) {
    let mut stream = match TcpStream::connect(address) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("my-lisp: cannot connect to {address}: {e}");
            process::exit(1);
        }
    };
    let mut line = String::new();
    match std::io::stdin().lock().read_line(&mut line) {
        Ok(0) => {
            eprintln!("my-lisp: --connect expects a request on stdin, none was found");
            process::exit(1);
        }
        Ok(_) => {
            let request = line.trim();
            if request.is_empty() {
                eprintln!("my-lisp: --connect expects a non-empty request on stdin");
                process::exit(1);
            }
            if writeln!(stream, "{request}").is_err() {
                eprintln!("my-lisp: write to {address} failed");
                process::exit(1);
            }
            let mut response = String::new();
            if BufReader::new(&stream).read_line(&mut response).is_ok() {
                print!("{response}");
            } else {
                eprintln!("my-lisp: read from {address} failed");
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("my-lisp: cannot read request from stdin: {e}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod oracle_result_tests {
    use super::*;

    fn contract() -> Value {
        Value::list([
            Value::Number(3.0, Exactness::Exact),
            Value::Number(0.0, Exactness::Exact),
        ])
    }

    #[test]
    fn oracle_result_is_versioned_and_provenance_bearing() {
        let result = oracle_result_value(
            "(+ 1 2)",
            &contract(),
            Value::Symbol("value".into()),
            Value::list([
                Value::Symbol("value".into()),
                Value::Number(3.0, Exactness::Exact),
            ]),
            &[],
            "oracle-evaluation",
        )
        .to_string();
        assert!(result.contains("(protocol oracle-result/1)"));
        assert!(result.contains("(contract-revision (3 0))"));
        assert!(result.contains("(source-digest \"sha256:"));
        assert!(result.contains("(outcome value)"));
        assert!(result.contains("(relation reference-implementation)"));
    }

    #[test]
    fn source_digest_is_stable_and_source_sensitive() {
        assert_eq!(source_digest("(+ 1 2)"), source_digest("(+ 1 2)"));
        assert_ne!(source_digest("(+ 1 2)"), source_digest("(+ 1 3)"));
    }

    #[test]
    fn unclosed_list_diagnostic_tells_agents_where_to_insert_one_paren() {
        let source = "(def answer\n  (+ 40 2)";
        let error = parse(source).unwrap_err();
        let diagnostic = oracle_error_payload(source, &error).to_string();
        assert!(diagnostic.contains("(code unclosed-list)"));
        assert!(diagnostic.contains("(location ((line 1) (column 1)))"));
        assert!(diagnostic.contains("(action insert)"));
        assert!(diagnostic.contains("(text \")\")"));
        assert!(diagnostic.contains(&format!("(offset {})", source.len())));
    }

    #[test]
    fn extra_closing_paren_diagnostic_tells_agents_exactly_what_to_delete() {
        let source = "(def answer 42)\n)";
        let error = parse(source).unwrap_err();
        let diagnostic = oracle_error_payload(source, &error).to_string();
        assert!(diagnostic.contains("(code unexpected-closing-parenthesis)"));
        assert!(diagnostic.contains("(location ((line 2) (column 1)))"));
        assert!(diagnostic.contains("(action delete)"));
        assert!(diagnostic.contains("(start 16)"));
        assert!(diagnostic.contains("(end 17)"));
    }
}

//! swarm-node — see docs/swarm-mesh-v2.md.
//!
//! A separate binary from `my-lisp`'s `:9999` semantic oracle: this is the
//! *coordination plane*, not the *semantic plane*. M0.1: persistent event
//! journal, node-id + epoch, peer handshake, sequence numbers, anti-entropy
//! sync, deterministic derived state from replayed events. M0.2: quorum
//! claim + fencing generation for exclusive task ownership. M0.2.1: gossip
//! peer discovery — a new node only needs one `--connect` to an existing
//! member and learns (and dials) the rest of the mesh on its own.
//!
//! To join an already-running swarm from a brand-new agent, run e.g.:
//!   swarm-node --port 9105 --node-id my-agent-1 --project my-project \
//!              --data-dir ~/.swarm-node/my-agent-1 --connect 127.0.0.1:9101
//! No other flags, no need to know every other member's address up front.

mod journal;
mod sexpr;
mod state;

use journal::{Event, Identity, Journal};
use sexpr::Sexp;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const VOTE_TIMEOUT: Duration = Duration::from_millis(1500);

struct Node {
    identity: Identity,
    project: String,
    listen_port: u16,
    journal: Mutex<Journal>,
    lamport: AtomicU64,
    peers: Mutex<HashMap<String, TcpStream>>,
    /// `node-id -> (ip, listen-port)` for every peer we've ever handshaken
    /// with or heard about via gossip — the address book that lets a
    /// freshly-joined node reach the rest of the mesh through just one
    /// `--connect`.
    peer_addrs: Mutex<HashMap<String, (String, u16)>>,
    /// `ip:port` strings currently being dialed, so gossip from several
    /// peers about the same newly-joined node doesn't race into duplicate
    /// outbound connection attempts.
    dialing: Mutex<HashSet<String>>,
    /// Votes for an in-flight `claim-task` proposal, keyed by `task:generation`.
    pending_votes: Mutex<HashMap<String, mpsc::Sender<bool>>>,
}

impl Node {
    fn tick_lamport(&self, received: u64) -> u64 {
        let mut cur = self.lamport.load(Ordering::SeqCst);
        loop {
            let next = cur.max(received) + 1;
            match self.lamport.compare_exchange(cur, next, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => return next,
                Err(actual) => cur = actual,
            }
        }
    }
}

struct Args {
    port: u16,
    node_id: String,
    project: String,
    data_dir: PathBuf,
    connect: Vec<String>,
}

fn parse_args() -> Args {
    let mut port = 9101u16;
    let mut node_id = "node-1".to_string();
    let mut project = "unknown".to_string();
    let mut data_dir = PathBuf::from(".swarm-node");
    let mut connect = Vec::new();

    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--port" => port = it.next().and_then(|v| v.parse().ok()).unwrap_or(port),
            "--node-id" => node_id = it.next().unwrap_or(node_id),
            "--project" => project = it.next().unwrap_or(project),
            "--data-dir" => data_dir = it.next().map(PathBuf::from).unwrap_or(data_dir),
            "--connect" => {
                if let Some(v) = it.next() {
                    connect.push(v);
                }
            }
            other => eprintln!("swarm-node: ignoring unknown argument `{other}`"),
        }
    }
    Args { port, node_id, project, data_dir, connect }
}

fn main() -> std::io::Result<()> {
    let args = parse_args();
    let identity = journal::load_or_init_identity(&args.data_dir, &args.node_id)?;
    let journal = Journal::open(&args.data_dir)?;
    let lamport_start = journal.max_lamport();
    eprintln!(
        "swarm-node: node={} epoch={} project={} journal={} events={} listening on 127.0.0.1:{}",
        identity.node_id,
        identity.epoch,
        args.project,
        journal.path().display(),
        journal.events.len(),
        args.port
    );

    let node = Arc::new(Node {
        identity,
        project: args.project,
        listen_port: args.port,
        journal: Mutex::new(journal),
        lamport: AtomicU64::new(lamport_start),
        peers: Mutex::new(HashMap::new()),
        peer_addrs: Mutex::new(HashMap::new()),
        dialing: Mutex::new(HashSet::new()),
        pending_votes: Mutex::new(HashMap::new()),
    });

    for addr in &args.connect {
        spawn_connect(&node, addr.clone());
    }

    let listener = TcpListener::bind(("127.0.0.1", args.port))?;
    for incoming in listener.incoming() {
        let stream = match incoming {
            Ok(s) => s,
            Err(e) => {
                eprintln!("swarm-node: accept error: {e}");
                continue;
            }
        };
        let node = node.clone();
        thread::spawn(move || handle_connection(node, stream, false));
    }
    Ok(())
}

/// Dials `addr` ("ip:port") on a background thread and runs the normal
/// handshake as initiator. De-duplicates against addresses already being
/// dialed so gossip about the same peer arriving from multiple directions
/// doesn't open redundant sockets.
fn spawn_connect(node: &Arc<Node>, addr: String) {
    {
        let mut dialing = node.dialing.lock().unwrap();
        if !dialing.insert(addr.clone()) {
            return;
        }
    }
    let node = node.clone();
    thread::spawn(move || {
        match TcpStream::connect(&addr) {
            Ok(stream) => handle_connection(node.clone(), stream, true),
            Err(e) => eprintln!("swarm-node: could not connect to {addr}: {e}"),
        }
        node.dialing.lock().unwrap().remove(&addr);
    });
}

fn send(stream: &mut TcpStream, msg: &Sexp) {
    let line = format!("{}\n", msg.to_text());
    let _ = stream.write_all(line.as_bytes());
}

fn handle_connection(node: Arc<Node>, mut stream: TcpStream, initiator: bool) {
    let peer_addr = stream.peer_addr().map(|a| a.to_string()).unwrap_or_default();
    let reader_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("swarm-node: could not clone stream for {peer_addr}: {e}");
            return;
        }
    };

    let peer_ip = peer_addr.rsplit_once(':').map(|(ip, _)| ip.to_string()).unwrap_or_else(|| peer_addr.clone());

    if initiator {
        send(
            &mut stream,
            &Sexp::list(vec![
                Sexp::atom("peer-hello"),
                Sexp::list(vec![Sexp::atom("protocol"), Sexp::atom("swarm/1")]),
                Sexp::list(vec![Sexp::atom("node"), Sexp::atom(&node.identity.node_id)]),
                Sexp::list(vec![Sexp::atom("epoch"), Sexp::atom(node.identity.epoch.to_string())]),
                Sexp::list(vec![Sexp::atom("project"), Sexp::atom(&node.project)]),
                Sexp::list(vec![Sexp::atom("listen-port"), Sexp::atom(node.listen_port.to_string())]),
            ]),
        );
        send_sync_hello(&node, &mut stream);
    }

    let mut peer_node_id: Option<String> = None;
    let reader = BufReader::new(reader_stream);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        let msg = match sexpr::parse(&line) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("swarm-node: bad message from {peer_addr}: {e}");
                continue;
            }
        };
        match msg.head() {
            Some("peer-hello") => {
                let their_node = msg.field_atom("node").unwrap_or("unknown").to_string();
                let their_epoch = msg.field_atom("epoch").unwrap_or("0");
                let their_port: u16 = msg.field_atom("listen-port").and_then(|s| s.parse().ok()).unwrap_or(0);
                eprintln!("swarm-node: peer-hello from {their_node} epoch={their_epoch}");
                send(
                    &mut stream,
                    &Sexp::list(vec![
                        Sexp::atom("peer-welcome"),
                        Sexp::list(vec![Sexp::atom("node"), Sexp::atom(&node.identity.node_id)]),
                        Sexp::list(vec![Sexp::atom("epoch"), Sexp::atom(node.identity.epoch.to_string())]),
                        Sexp::list(vec![Sexp::atom("swarm-id"), Sexp::atom("my-lisp-ecosystem")]),
                        Sexp::list(vec![Sexp::atom("protocol"), Sexp::atom("swarm/1")]),
                        Sexp::list(vec![Sexp::atom("listen-port"), Sexp::atom(node.listen_port.to_string())]),
                    ]),
                );
                send_sync_hello(&node, &mut stream);
                register_peer(&node, &their_node, &mut stream, &peer_ip, their_port);
                send_peer_list(&node, &their_node, &mut stream);
                peer_node_id = Some(their_node);
            }
            Some("peer-welcome") => {
                let their_node = msg.field_atom("node").unwrap_or("unknown").to_string();
                let their_port: u16 = msg.field_atom("listen-port").and_then(|s| s.parse().ok()).unwrap_or(0);
                eprintln!("swarm-node: peer-welcome from {their_node}");
                register_peer(&node, &their_node, &mut stream, &peer_ip, their_port);
                send_peer_list(&node, &their_node, &mut stream);
                peer_node_id = Some(their_node);
            }
            Some("peer-list") => {
                handle_peer_list(&node, &msg);
            }
            Some("sync-hello") => {
                handle_sync_hello(&node, &msg, &mut stream);
            }
            Some("sync-events") => {
                handle_sync_events(&node, &msg);
            }
            Some("push-event") => {
                handle_push_event(&node, &msg);
            }
            Some("emit") => {
                handle_emit(&node, &msg, &mut stream);
            }
            Some("claim-proposal") => {
                handle_claim_proposal(&node, &msg, &mut stream);
            }
            Some("claim-vote") => {
                handle_claim_vote(&node, &msg);
            }
            Some("claim-task") => {
                handle_claim_task(&node, &msg, &mut stream);
            }
            Some("release-task") => {
                handle_release_task(&node, &msg, &mut stream);
            }
            Some("complete-task") => {
                handle_complete_task(&node, &msg, &mut stream);
            }
            Some("task-state") => {
                handle_task_state(&node, &msg, &mut stream);
            }
            Some("list-task-state") => {
                handle_list_task_state(&node, &mut stream);
            }
            other => {
                eprintln!("swarm-node: unrecognized message head {other:?} from {peer_addr}");
            }
        }
    }
    if let Some(id) = peer_node_id {
        node.peers.lock().unwrap().remove(&id);
        eprintln!("swarm-node: connection to {id} closed");
    }
}

/// Records a freshly-handshaken peer: keeps its live stream for broadcast
/// and, if it told us its listen port, its dialable address for gossip.
fn register_peer(node: &Arc<Node>, their_node: &str, stream: &mut TcpStream, their_ip: &str, their_port: u16) {
    if let Ok(clone) = stream.try_clone() {
        node.peers.lock().unwrap().insert(their_node.to_string(), clone);
    }
    if their_port != 0 {
        let is_new = node
            .peer_addrs
            .lock()
            .unwrap()
            .insert(their_node.to_string(), (their_ip.to_string(), their_port))
            .is_none();
        if is_new {
            announce_peer(node, their_node, their_ip, their_port);
        }
    }
}

/// Tells every *other* currently-connected peer about a node that just
/// joined, so gossip reaches nodes that connected before the newcomer
/// existed and would otherwise never learn about it.
fn announce_peer(node: &Arc<Node>, new_id: &str, new_ip: &str, new_port: u16) {
    let announcement = Sexp::list(vec![
        Sexp::atom("peer-list"),
        Sexp::list(vec![
            Sexp::atom("peers"),
            Sexp::list(vec![Sexp::list(vec![Sexp::atom(new_id), Sexp::atom(new_ip), Sexp::atom(new_port.to_string())])]),
        ]),
    ]);
    let mut peers = node.peers.lock().unwrap();
    let mut dead = Vec::new();
    let line = format!("{}\n", announcement.to_text());
    for (peer_id, stream) in peers.iter_mut() {
        if peer_id == new_id {
            continue;
        }
        if stream.write_all(line.as_bytes()).is_err() {
            dead.push(peer_id.clone());
        }
    }
    for id in dead {
        peers.remove(&id);
    }
}

/// Shares everything we know about the mesh (minus the recipient itself)
/// so a node that joined through just one `--connect` learns the rest of
/// the members and can reach full mesh on its own.
fn send_peer_list(node: &Arc<Node>, recipient_id: &str, stream: &mut TcpStream) {
    let entries: Vec<Sexp> = node
        .peer_addrs
        .lock()
        .unwrap()
        .iter()
        .filter(|(id, _)| id.as_str() != recipient_id)
        .map(|(id, (ip, port))| Sexp::list(vec![Sexp::atom(id), Sexp::atom(ip), Sexp::atom(port.to_string())]))
        .collect();
    if entries.is_empty() {
        return;
    }
    send(stream, &Sexp::list(vec![Sexp::atom("peer-list"), Sexp::list(vec![Sexp::atom("peers"), Sexp::list(entries)])]));
}

/// Auto-connects to newly-learned peers. Only the lexicographically lower
/// node-id in a pair dials out, so gossip reaching both sides doesn't open
/// two redundant connections for the same pair.
fn handle_peer_list(node: &Arc<Node>, msg: &Sexp) {
    let entries: &[Sexp] = match msg.field("peers").and_then(|f| f.first()) {
        Some(Sexp::List(items)) => items,
        _ => &[],
    };
    for entry in entries {
        let Sexp::List(fields) = entry else { continue };
        let (Some(Sexp::Atom(id)), Some(Sexp::Atom(ip)), Some(Sexp::Atom(port))) = (fields.first(), fields.get(1), fields.get(2)) else { continue };
        let Ok(port) = port.parse::<u16>() else { continue };
        if id == &node.identity.node_id {
            continue;
        }
        node.peer_addrs.lock().unwrap().insert(id.clone(), (ip.clone(), port));
        let already_connected = node.peers.lock().unwrap().contains_key(id);
        if !already_connected && node.identity.node_id < *id {
            eprintln!("swarm-node: learned of {id} at {ip}:{port} via gossip, dialing");
            spawn_connect(node, format!("{ip}:{port}"));
        }
    }
}

fn send_sync_hello(node: &Arc<Node>, stream: &mut TcpStream) {
    let journal = node.journal.lock().unwrap();
    let seen: Vec<Sexp> = journal
        .all_node_ids()
        .into_iter()
        .map(|id| {
            let last = journal.last_seq(&id);
            Sexp::list(vec![Sexp::atom(id), Sexp::atom(last.to_string())])
        })
        .collect();
    send(
        stream,
        &Sexp::list(vec![
            Sexp::atom("sync-hello"),
            Sexp::list(vec![Sexp::atom("node"), Sexp::atom(&node.identity.node_id)]),
            Sexp::list(vec![Sexp::atom("seen"), Sexp::list(seen)]),
        ]),
    );
}

fn handle_sync_hello(node: &Arc<Node>, msg: &Sexp, stream: &mut TcpStream) {
    let their_node = msg.field_atom("node").unwrap_or("unknown");
    let seen_items: &[Sexp] = match msg.field("seen").and_then(|f| f.first()) {
        Some(Sexp::List(items)) => items,
        _ => &[],
    };
    let mut seen_map: HashMap<String, u64> = HashMap::new();
    for entry in seen_items {
        if let Sexp::List(pair) = entry {
            if let (Some(Sexp::Atom(n)), Some(Sexp::Atom(s))) = (pair.first(), pair.get(1)) {
                seen_map.insert(n.clone(), s.parse().unwrap_or(0));
            }
        }
    }
    let journal = node.journal.lock().unwrap();
    let mut missing_events = Vec::new();
    for node_id in journal.all_node_ids() {
        let their_seen = *seen_map.get(&node_id).unwrap_or(&0);
        for ev in journal.events_after(&node_id, their_seen) {
            missing_events.push(ev.to_sexp());
        }
    }
    if !missing_events.is_empty() {
        eprintln!(
            "swarm-node: sending {} catch-up event(s) to {their_node}",
            missing_events.len()
        );
        send(
            stream,
            &Sexp::list(vec![
                Sexp::atom("sync-events"),
                Sexp::list(vec![Sexp::atom("from"), Sexp::atom(&node.identity.node_id)]),
                Sexp::list(vec![Sexp::atom("events"), Sexp::list(missing_events)]),
            ]),
        );
    }
}

fn handle_sync_events(node: &Arc<Node>, msg: &Sexp) {
    let events: &[Sexp] = match msg.field("events").and_then(|f| f.first()) {
        Some(Sexp::List(items)) => items,
        _ => &[],
    };
    let mut journal = node.journal.lock().unwrap();
    let mut applied = 0;
    for ev_sexp in events {
        if let Ok(ev) = Event::from_sexp(ev_sexp) {
            if !journal.has(&ev.node, ev.seq) {
                let lamport = ev.lamport;
                if journal.append(ev).is_ok() {
                    applied += 1;
                    node.lamport.fetch_max(lamport, Ordering::SeqCst);
                }
            }
        }
    }
    if applied > 0 {
        eprintln!("swarm-node: applied {applied} event(s) from anti-entropy sync");
    }
}

fn handle_push_event(node: &Arc<Node>, msg: &Sexp) {
    let Some(ev_sexp) = msg.field("event").and_then(|f| f.first()) else { return };
    let Ok(ev) = Event::from_sexp(ev_sexp) else { return };
    let mut journal = node.journal.lock().unwrap();
    if journal.has(&ev.node, ev.seq) {
        return;
    }
    let lamport = ev.lamport;
    if journal.append(ev.clone()).is_ok() {
        node.lamport.fetch_max(lamport, Ordering::SeqCst);
        drop(journal);
        broadcast_event(node, &ev, Some(&ev.node));
    }
}

fn broadcast_event(node: &Arc<Node>, event: &Event, skip_origin: Option<&str>) {
    let msg = Sexp::list(vec![Sexp::atom("push-event"), Sexp::list(vec![Sexp::atom("event"), event.to_sexp()])]);
    let mut peers = node.peers.lock().unwrap();
    let mut dead = Vec::new();
    for (peer_id, stream) in peers.iter_mut() {
        if Some(peer_id.as_str()) == skip_origin {
            continue;
        }
        let line = format!("{}\n", msg.to_text());
        if stream.write_all(line.as_bytes()).is_err() {
            dead.push(peer_id.clone());
        }
    }
    for id in dead {
        peers.remove(&id);
    }
}

/// Local client injects a fact: `(emit (type evidence-created) (payload (...)))`.
fn handle_emit(node: &Arc<Node>, msg: &Sexp, stream: &mut TcpStream) {
    let Some(typ) = msg.field_atom("type") else {
        send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string("emit requires a `type` field")]));
        return;
    };
    let payload = msg.field("payload").and_then(|f| f.first()).cloned().unwrap_or(Sexp::List(vec![]));
    let lamport = node.tick_lamport(0);
    let mut journal = node.journal.lock().unwrap();
    let seq = journal.next_seq(&node.identity.node_id);
    let event = Event { node: node.identity.node_id.clone(), seq, lamport, typ: typ.to_string(), payload };
    match journal.append(event.clone()) {
        Ok(()) => {
            drop(journal);
            send(
                stream,
                &Sexp::list(vec![
                    Sexp::atom("ok"),
                    Sexp::list(vec![Sexp::atom("id"), Sexp::atom(event.id())]),
                ]),
            );
            broadcast_event(node, &event, None);
        }
        Err(e) => {
            send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string(format!("journal append failed: {e}"))]));
        }
    }
}

fn broadcast_to_peers(node: &Arc<Node>, msg: &Sexp) {
    let mut peers = node.peers.lock().unwrap();
    let mut dead = Vec::new();
    let line = format!("{}\n", msg.to_text());
    for (peer_id, stream) in peers.iter_mut() {
        if stream.write_all(line.as_bytes()).is_err() {
            dead.push(peer_id.clone());
        }
    }
    for id in dead {
        peers.remove(&id);
    }
}

/// Appends a task-ownership fact (`claim-committed`/`claim-released`/
/// `task-completed`) to our own journal and gossips it like any other
/// event — the vote only gates whether this function gets called, the
/// resulting fact itself is plain CRDT-style replication.
fn append_task_fact(node: &Arc<Node>, typ: &str, task: &str, generation: u64) -> std::io::Result<Event> {
    let payload = Sexp::list(vec![
        Sexp::list(vec![Sexp::atom("task"), Sexp::atom(task)]),
        Sexp::list(vec![Sexp::atom("agent"), Sexp::atom(&node.identity.node_id)]),
        Sexp::list(vec![Sexp::atom("generation"), Sexp::atom(generation.to_string())]),
    ]);
    let lamport = node.tick_lamport(0);
    let mut journal = node.journal.lock().unwrap();
    let seq = journal.next_seq(&node.identity.node_id);
    let event = Event { node: node.identity.node_id.clone(), seq, lamport, typ: typ.to_string(), payload };
    journal.append(event.clone())?;
    drop(journal);
    broadcast_event(node, &event, None);
    Ok(event)
}

/// A peer is asking us to vote on `(claim-proposal (task ..) (agent ..) (generation ..))`.
/// We vote yes only if the proposed generation is exactly the next one after
/// what we've derived locally and the task isn't already held/completed —
/// this is the fencing check; it does not lock out a *concurrent* competing
/// proposal for the same task (a known M0.2 simplification: two proposals in
/// flight at once can both reach quorum on disjoint peer sets in a genuine
/// network partition — full exclusion needs a per-task in-flight lock, left
/// for a later iteration since it needs its own timeout/cleanup story).
fn handle_claim_proposal(node: &Arc<Node>, msg: &Sexp, stream: &mut TcpStream) {
    let task = msg.field_atom("task").unwrap_or("").to_string();
    let agent = msg.field_atom("agent").unwrap_or("").to_string();
    let generation: u64 = msg.field_atom("generation").and_then(|s| s.parse().ok()).unwrap_or(0);

    let journal = node.journal.lock().unwrap();
    let current = state::task_state(&journal, &task);
    drop(journal);

    let vote = !current.completed && current.holder.is_none() && generation == current.generation + 1;
    eprintln!(
        "swarm-node: vote {} on claim-proposal task={task} agent={agent} generation={generation} (local gen={}, holder={:?})",
        if vote { "YES" } else { "NO" },
        current.generation,
        current.holder
    );
    send(
        stream,
        &Sexp::list(vec![
            Sexp::atom("claim-vote"),
            Sexp::list(vec![Sexp::atom("task"), Sexp::atom(&task)]),
            Sexp::list(vec![Sexp::atom("generation"), Sexp::atom(generation.to_string())]),
            Sexp::list(vec![Sexp::atom("voter"), Sexp::atom(&node.identity.node_id)]),
            Sexp::list(vec![Sexp::atom("vote"), Sexp::atom(if vote { "yes" } else { "no" })]),
        ]),
    );
}

fn handle_claim_vote(node: &Arc<Node>, msg: &Sexp) {
    let task = msg.field_atom("task").unwrap_or("");
    let generation = msg.field_atom("generation").unwrap_or("0");
    let vote = msg.field_atom("vote") == Some("yes");
    let key = format!("{task}:{generation}");
    if let Some(tx) = node.pending_votes.lock().unwrap().get(&key) {
        let _ = tx.send(vote);
    }
}

/// Local client op: `(claim-task (task <id>))`. Proposes the next generation
/// to all currently connected peers and only commits (appends
/// `claim-committed` to our own journal) once a majority of the total known
/// nodes (self included) have voted yes within `VOTE_TIMEOUT`.
fn handle_claim_task(node: &Arc<Node>, msg: &Sexp, stream: &mut TcpStream) {
    let Some(task) = msg.field_atom("task") else {
        send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string("claim-task requires a `task` field")]));
        return;
    };

    let journal = node.journal.lock().unwrap();
    let current = state::task_state(&journal, task);
    drop(journal);

    if current.completed {
        send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string(format!("task `{task}` is already completed"))]));
        return;
    }
    if let Some(holder) = &current.holder {
        send(
            stream,
            &Sexp::list(vec![
                Sexp::atom("error"),
                Sexp::string(format!("task `{task}` already claimed by `{holder}` at generation {}", current.generation)),
            ]),
        );
        return;
    }

    let generation = current.generation + 1;
    let key = format!("{task}:{generation}");
    let (tx, rx) = mpsc::channel::<bool>();
    node.pending_votes.lock().unwrap().insert(key.clone(), tx);

    let peer_count = node.peers.lock().unwrap().len();
    let total_nodes = peer_count + 1; // + self
    let quorum = total_nodes / 2 + 1;

    broadcast_to_peers(
        node,
        &Sexp::list(vec![
            Sexp::atom("claim-proposal"),
            Sexp::list(vec![Sexp::atom("task"), Sexp::atom(task)]),
            Sexp::list(vec![Sexp::atom("agent"), Sexp::atom(&node.identity.node_id)]),
            Sexp::list(vec![Sexp::atom("generation"), Sexp::atom(generation.to_string())]),
        ]),
    );

    let mut yes_votes = 1; // self
    let mut responses = 0;
    let deadline = Instant::now() + VOTE_TIMEOUT;
    while yes_votes < quorum && responses < peer_count {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        match rx.recv_timeout(remaining) {
            Ok(true) => {
                yes_votes += 1;
                responses += 1;
            }
            Ok(false) => responses += 1,
            Err(_) => break,
        }
    }
    node.pending_votes.lock().unwrap().remove(&key);

    if yes_votes >= quorum {
        match append_task_fact(node, "claim-committed", task, generation) {
            Ok(_) => {
                send(
                    stream,
                    &Sexp::list(vec![
                        Sexp::atom("ok"),
                        Sexp::list(vec![Sexp::atom("task"), Sexp::atom(task)]),
                        Sexp::list(vec![Sexp::atom("generation"), Sexp::atom(generation.to_string())]),
                        Sexp::list(vec![Sexp::atom("votes"), Sexp::atom(format!("{yes_votes}/{total_nodes}"))]),
                    ]),
                );
            }
            Err(e) => {
                send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string(format!("journal append failed: {e}"))]));
            }
        }
    } else {
        send(
            stream,
            &Sexp::list(vec![
                Sexp::atom("error"),
                Sexp::string(format!(
                    "quorum not reached for `{task}` generation {generation}: {yes_votes}/{quorum} needed (of {total_nodes} known nodes)"
                )),
            ]),
        );
    }
}

/// Local client op: `(release-task (task <id>) (generation <n>))`. Fencing
/// check: only the current holder, quoting the generation it was granted,
/// may release — a stale/recovered agent citing an old generation is
/// rejected rather than silently accepted.
fn handle_release_task(node: &Arc<Node>, msg: &Sexp, stream: &mut TcpStream) {
    release_or_complete(node, msg, stream, "claim-released", "release-task");
}

/// Local client op: `(complete-task (task <id>) (generation <n>))`. Same
/// fencing rule as `release-task`, but the resulting fact marks the task
/// permanently done rather than merely available again.
fn handle_complete_task(node: &Arc<Node>, msg: &Sexp, stream: &mut TcpStream) {
    release_or_complete(node, msg, stream, "task-completed", "complete-task");
}

fn release_or_complete(node: &Arc<Node>, msg: &Sexp, stream: &mut TcpStream, fact_type: &str, op_name: &str) {
    let (Some(task), Some(generation_str)) = (msg.field_atom("task"), msg.field_atom("generation")) else {
        send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string(format!("{op_name} requires `task` and `generation` fields"))]));
        return;
    };
    let Ok(generation) = generation_str.parse::<u64>() else {
        send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string("`generation` must be a number")]));
        return;
    };

    let journal = node.journal.lock().unwrap();
    let current = state::task_state(&journal, task);
    drop(journal);

    if current.holder.as_deref() != Some(node.identity.node_id.as_str()) || current.generation != generation {
        send(
            stream,
            &Sexp::list(vec![
                Sexp::atom("error"),
                Sexp::string(format!(
                    "STALE: `{task}` is at generation {} (holder {}), not generation {generation} held by us",
                    current.generation,
                    current.holder.as_deref().unwrap_or("none")
                )),
            ]),
        );
        return;
    }

    match append_task_fact(node, fact_type, task, generation) {
        Ok(_) => send(stream, &Sexp::list(vec![Sexp::atom("ok"), Sexp::list(vec![Sexp::atom("task"), Sexp::atom(task)])])),
        Err(e) => send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string(format!("journal append failed: {e}"))])),
    }
}

fn task_state_sexp(task: &str, s: &state::TaskState) -> Sexp {
    Sexp::list(vec![
        Sexp::list(vec![Sexp::atom("task"), Sexp::atom(task)]),
        Sexp::list(vec![Sexp::atom("generation"), Sexp::atom(s.generation.to_string())]),
        Sexp::list(vec![
            Sexp::atom("holder"),
            match &s.holder {
                Some(h) => Sexp::atom(h),
                None => Sexp::List(vec![]),
            },
        ]),
        Sexp::list(vec![Sexp::atom("completed"), Sexp::atom(if s.completed { "t" } else { "nil" })]),
    ])
}

fn handle_task_state(node: &Arc<Node>, msg: &Sexp, stream: &mut TcpStream) {
    let Some(task) = msg.field_atom("task") else {
        send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string("task-state requires a `task` field")]));
        return;
    };
    let journal = node.journal.lock().unwrap();
    let s = state::task_state(&journal, task);
    drop(journal);
    send(stream, &task_state_sexp(task, &s));
}

fn handle_list_task_state(node: &Arc<Node>, stream: &mut TcpStream) {
    let journal = node.journal.lock().unwrap();
    let entries: Vec<Sexp> = state::all_task_ids(&journal)
        .iter()
        .map(|task| task_state_sexp(task, &state::task_state(&journal, task)))
        .collect();
    drop(journal);
    send(stream, &Sexp::list(vec![Sexp::atom("task-states"), Sexp::list(entries)]));
}

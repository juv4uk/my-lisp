//! swarm-node — M0.1 of docs/swarm-mesh-v2.md.
//!
//! A separate binary from `my-lisp`'s `:9999` semantic oracle: this is the
//! *coordination plane*, not the *semantic plane*. Scope for M0.1 only:
//! persistent event journal, node-id + epoch, peer handshake, sequence
//! numbers, anti-entropy sync, deterministic derived state from replayed
//! events. No claim/quorum/consensus yet — that is M0.2.

mod journal;
mod sexpr;

use journal::{Event, Identity, Journal};
use sexpr::Sexp;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

struct Node {
    identity: Identity,
    project: String,
    journal: Mutex<Journal>,
    lamport: AtomicU64,
    peers: Mutex<HashMap<String, TcpStream>>,
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
        journal: Mutex::new(journal),
        lamport: AtomicU64::new(lamport_start),
        peers: Mutex::new(HashMap::new()),
    });

    for addr in &args.connect {
        let node = node.clone();
        let addr = addr.clone();
        thread::spawn(move || {
            match TcpStream::connect(&addr) {
                Ok(stream) => handle_connection(node, stream, true),
                Err(e) => eprintln!("swarm-node: could not connect to {addr}: {e}"),
            }
        });
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

    if initiator {
        send(
            &mut stream,
            &Sexp::list(vec![
                Sexp::atom("peer-hello"),
                Sexp::list(vec![Sexp::atom("protocol"), Sexp::atom("swarm/1")]),
                Sexp::list(vec![Sexp::atom("node"), Sexp::atom(&node.identity.node_id)]),
                Sexp::list(vec![Sexp::atom("epoch"), Sexp::atom(node.identity.epoch.to_string())]),
                Sexp::list(vec![Sexp::atom("project"), Sexp::atom(&node.project)]),
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
                eprintln!("swarm-node: peer-hello from {their_node} epoch={their_epoch}");
                send(
                    &mut stream,
                    &Sexp::list(vec![
                        Sexp::atom("peer-welcome"),
                        Sexp::list(vec![Sexp::atom("node"), Sexp::atom(&node.identity.node_id)]),
                        Sexp::list(vec![Sexp::atom("epoch"), Sexp::atom(node.identity.epoch.to_string())]),
                        Sexp::list(vec![Sexp::atom("swarm-id"), Sexp::atom("my-lisp-ecosystem")]),
                        Sexp::list(vec![Sexp::atom("protocol"), Sexp::atom("swarm/1")]),
                    ]),
                );
                send_sync_hello(&node, &mut stream);
                if let Ok(clone) = stream.try_clone() {
                    node.peers.lock().unwrap().insert(their_node.clone(), clone);
                }
                peer_node_id = Some(their_node);
            }
            Some("peer-welcome") => {
                let their_node = msg.field_atom("node").unwrap_or("unknown").to_string();
                eprintln!("swarm-node: peer-welcome from {their_node}");
                if let Ok(clone) = stream.try_clone() {
                    node.peers.lock().unwrap().insert(their_node.clone(), clone);
                }
                peer_node_id = Some(their_node);
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

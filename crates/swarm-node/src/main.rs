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

mod compact;
mod journal;
mod log;
mod sexpr;
mod state;
mod tasks_file;

use log::{log_info as info, log_warn as warn};

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
/// How long a voter's "I promised generation N to someone" holds before it
/// expires and can be re-promised. Must exceed `VOTE_TIMEOUT` with margin
/// so a proposer that's still legitimately waiting on votes doesn't get
/// undercut by its own promise expiring first; bounds how long a task can
/// get stuck if a proposer dies mid-vote without completing or retrying.
const PROMISE_TTL: Duration = Duration::from_secs(5);
/// How often each node pings every currently-connected peer with a
/// `heartbeat` message.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// A peer we haven't heard *anything* from (heartbeat or otherwise) in
/// this long is considered stale and its connection is forcibly closed —
/// closing it (rather than just noting it) is what actually triggers
/// `spawn_connect`'s retry loop to redial and re-handshake, since a
/// half-open TCP connection (peer process died without a clean FIN, or a
/// network partition) can otherwise sit unnoticed far longer than a
/// stalled write would take to fail.
const STALE_PEER_TIMEOUT: Duration = Duration::from_secs(20);

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
    /// Votes for an in-flight `claim-task` proposal, keyed by `task:generation`,
    /// carrying `(voter-node-id, vote)` so a tally can filter to actual voters.
    pending_votes: Mutex<HashMap<String, mpsc::Sender<(String, bool)>>>,
    /// How many `--connect` bootstrap addresses we were started with; 0
    /// means we're the first node and are trivially caught up.
    bootstrap_expected: usize,
    /// Node ids we've received a definitive sync answer from (either
    /// `sync-events` or `sync-complete`) since startup — see `synced()`.
    caught_up_with: Mutex<HashSet<String>>,
    /// Per-task voting promises: `task -> (generation we last voted yes
    /// for, when)`. Closes the concurrent-proposal gap noted as deferred
    /// in M0.2 — without this, two proposers racing for the same task
    /// could each collect yes votes from disjoint voter sets (e.g. across
    /// a network partition) and both reach quorum on the same generation.
    /// A voter now refuses to vote yes again for a task/generation it's
    /// already promised, until that promise expires (`PROMISE_TTL`).
    promised: Mutex<HashMap<String, (u64, Instant)>>,
    /// Last time we received *any* message (heartbeat or otherwise) from
    /// each connected peer — see `HEARTBEAT_INTERVAL`/`STALE_PEER_TIMEOUT`.
    last_seen: Mutex<HashMap<String, Instant>>,
    /// Process start time, for `(metrics)`'s uptime field.
    started_at: Instant,
}

impl Node {
    fn synced(&self) -> bool {
        self.bootstrap_expected == 0 || !self.caught_up_with.lock().unwrap().is_empty()
    }
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
    /// Interface to listen on. Defaults to `127.0.0.1` (unchanged,
    /// localhost-only behavior) so existing single-machine setups keep
    /// working exactly as before; pass `0.0.0.0` (or a specific interface
    /// IP, e.g. a Tailscale address) to accept connections from other
    /// machines. Not validated/firewalled by swarm-node itself — whatever
    /// network the bind address is reachable from is who can talk to this
    /// node, so this is a deliberate, explicit opt-in, not a new default.
    /// (No separate "advertise" address is needed: gossip already learns
    /// a peer's dialable address from the observed source IP of its
    /// connection — correct as-is for direct-routing overlays like
    /// Tailscale, which don't rewrite addresses in transit. A NAT'd
    /// deployment where the observed IP isn't reachable would need that
    /// as a later addition, not preemptively built for an unconfirmed case.)
    bind: String,
}

fn parse_args() -> Args {
    let mut port = 9101u16;
    let mut node_id = "node-1".to_string();
    let mut project = "unknown".to_string();
    let mut data_dir = PathBuf::from(".swarm-node");
    let mut connect = Vec::new();
    let mut bind = "127.0.0.1".to_string();

    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--help" | "-h" => print_usage_and_exit(),
            "--port" => port = it.next().and_then(|v| v.parse().ok()).unwrap_or(port),
            "--node-id" => node_id = it.next().unwrap_or(node_id),
            "--project" => project = it.next().unwrap_or(project),
            "--data-dir" => data_dir = it.next().map(PathBuf::from).unwrap_or(data_dir),
            "--bind" => bind = it.next().unwrap_or(bind),
            "--connect" => {
                if let Some(v) = it.next() {
                    connect.push(v);
                }
            }
            other => warn!("swarm-node: ignoring unknown argument `{other}`"),
        }
    }
    Args { port, node_id, project, data_dir, connect, bind }
}

/// `--help`/`-h` must exit before touching the network or filesystem at
/// all — the bug this fixes (`SWARM-NODE-HELP-FLAG-BUG`) was that an
/// unrecognized-looking `--help` fell through to `other => warn!(...)`
/// and then the process kept going and actually started a real node
/// under every default (relative `.swarm-node` data-dir, port 9101,
/// node-id `node-1`) — confirmed live: running `--help` while port 9101
/// was already in use crashed with `AddrInUse` instead of showing help,
/// and with the port free it would have silently joined/started a real
/// node under those defaults instead.
fn print_usage_and_exit() -> ! {
    println!(
        "swarm-node — P2P coordination-plane node (see docs/swarm-mesh-v2.md)\n\
         \n\
         USAGE:\n\
         \x20\x20swarm-node [OPTIONS]\n\
         \n\
         OPTIONS:\n\
         \x20\x20--port <PORT>          Listen port (default: 9101)\n\
         \x20\x20--node-id <ID>         Stable node identity (default: node-1 — change this)\n\
         \x20\x20--project <NAME>       Project label reported in handshakes (default: unknown)\n\
         \x20\x20--data-dir <PATH>      Journal/identity directory (default: .swarm-node — prefer an\n\
         \x20\x20                       absolute path, e.g. ~/.swarm-node/<node-id>; a relative one\n\
         \x20\x20                       resolves against wherever this process happens to be started)\n\
         \x20\x20--bind <ADDRESS>       Interface to listen on (default: 127.0.0.1, localhost-only;\n\
         \x20\x20                       pass 0.0.0.0 or a specific interface IP for cross-machine use)\n\
         \x20\x20--connect <HOST:PORT>  Bootstrap peer to dial on startup (repeatable; one is enough,\n\
         \x20\x20                       gossip discovers the rest of the mesh)\n\
         \x20\x20--help, -h             Show this message and exit\n\
         \n\
         See docs/swarm-mesh-v2.md's onboarding checklist for a full first-join walkthrough."
    );
    std::process::exit(0);
}

fn main() -> std::io::Result<()> {
    let args = parse_args();
    let identity = journal::load_or_init_identity(&args.data_dir, &args.node_id)?;
    let journal = Journal::open(&args.data_dir)?;
    let lamport_start = journal.max_lamport();
    info!(
        "swarm-node: node={} epoch={} project={} journal={} events={} listening on {}:{}",
        identity.node_id,
        identity.epoch,
        args.project,
        journal.path().display(),
        journal.events.len(),
        args.bind,
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
        bootstrap_expected: args.connect.len(),
        caught_up_with: Mutex::new(HashSet::new()),
        promised: Mutex::new(HashMap::new()),
        last_seen: Mutex::new(HashMap::new()),
        started_at: Instant::now(),
    });

    for addr in &args.connect {
        spawn_connect(&node, addr.clone());
    }

    spawn_heartbeat(&node);

    let listener = TcpListener::bind((args.bind.as_str(), args.port))?;
    for incoming in listener.incoming() {
        let stream = match incoming {
            Ok(s) => s,
            Err(e) => {
                warn!("swarm-node: accept error: {e}");
                continue;
            }
        };
        let node = node.clone();
        thread::spawn(move || handle_connection(node, stream, false));
    }
    Ok(())
}

const RECONNECT_INITIAL_BACKOFF: Duration = Duration::from_millis(500);
const RECONNECT_MAX_BACKOFF: Duration = Duration::from_secs(30);

/// Dials `addr` ("ip:port") on a background thread and runs the normal
/// handshake as initiator, retrying with capped exponential backoff for as
/// long as the process lives — whether the first attempt fails outright or
/// a previously-established connection drops later (e.g. the peer at that
/// address restarted). De-duplicates against addresses already being
/// dialed/held so gossip about the same peer arriving from multiple
/// directions doesn't spawn a second retry loop for it.
///
/// This closes real restart-churn pain: before this, a `--connect` (or a
/// gossip-discovered peer) was dialed exactly once at startup, so *any*
/// restart of the node on the other end silently and permanently dropped
/// that link until someone manually restarted this side too.
fn spawn_connect(node: &Arc<Node>, addr: String) {
    {
        let mut dialing = node.dialing.lock().unwrap();
        if !dialing.insert(addr.clone()) {
            return;
        }
    }
    let node = node.clone();
    thread::spawn(move || {
        let mut backoff = RECONNECT_INITIAL_BACKOFF;
        loop {
            match TcpStream::connect(&addr) {
                Ok(stream) => {
                    handle_connection(node.clone(), stream, true);
                    backoff = RECONNECT_INITIAL_BACKOFF; // connection lasted a while, reset
                }
                Err(e) => warn!("swarm-node: could not connect to {addr}: {e}, retrying in {backoff:?}"),
            }
            thread::sleep(backoff);
            backoff = (backoff * 2).min(RECONNECT_MAX_BACKOFF);
        }
    });
}

fn send(stream: &mut TcpStream, msg: &Sexp) {
    let line = format!("{}\n", msg.to_text());
    let _ = stream.write_all(line.as_bytes());
}

/// Background liveness maintenance (M0.10, `SWARM-P2P-HEARTBEAT`): every
/// `HEARTBEAT_INTERVAL`, ping every connected peer and forcibly close any
/// connection we haven't heard *anything* from in `STALE_PEER_TIMEOUT`.
/// TCP alone can leave a half-open connection (the peer's process died
/// without a clean FIN, or a network partition) sitting unnoticed for a
/// long time if there's nothing new to write on it — this bounds that to
/// roughly `STALE_PEER_TIMEOUT`. Closing (not just noting) a stale
/// connection matters: `spawn_connect`'s retry loop only redials once
/// `handle_connection` actually returns, which only happens once the
/// socket is genuinely closed.
fn spawn_heartbeat(node: &Arc<Node>) {
    let node = node.clone();
    thread::spawn(move || loop {
        thread::sleep(HEARTBEAT_INTERVAL);

        let beat = Sexp::list(vec![
            Sexp::atom("heartbeat"),
            Sexp::list(vec![Sexp::atom("node"), Sexp::atom(&node.identity.node_id)]),
            Sexp::list(vec![Sexp::atom("epoch"), Sexp::atom(node.identity.epoch.to_string())]),
        ]);
        broadcast_to_peers(&node, &beat);

        let now = Instant::now();
        let stale: Vec<String> = node
            .last_seen
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, seen)| now.duration_since(**seen) > STALE_PEER_TIMEOUT)
            .map(|(id, _)| id.clone())
            .collect();
        if stale.is_empty() {
            continue;
        }
        let mut peers = node.peers.lock().unwrap();
        for id in &stale {
            if let Some(stream) = peers.remove(id) {
                warn!("swarm-node: peer {id} silent for over {STALE_PEER_TIMEOUT:?}, closing connection");
                let _ = stream.shutdown(std::net::Shutdown::Both);
            }
        }
    });
}

fn handle_connection(node: Arc<Node>, mut stream: TcpStream, initiator: bool) {
    let peer_addr = stream.peer_addr().map(|a| a.to_string()).unwrap_or_default();
    let reader_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(e) => {
            warn!("swarm-node: could not clone stream for {peer_addr}: {e}");
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
        if let Some(id) = &peer_node_id {
            node.last_seen.lock().unwrap().insert(id.clone(), Instant::now());
        }
        let msg = match sexpr::parse(&line) {
            Ok(m) => m,
            Err(e) => {
                warn!("swarm-node: bad message from {peer_addr}: {e}");
                continue;
            }
        };
        match msg.head() {
            Some("peer-hello") => {
                let their_node = msg.field_atom("node").unwrap_or("unknown").to_string();
                let their_epoch = msg.field_atom("epoch").unwrap_or("0");
                let their_port: u16 = msg.field_atom("listen-port").and_then(|s| s.parse().ok()).unwrap_or(0);
                if identity_already_live(&node, &their_node) {
                    warn!("swarm-node: rejecting peer-hello claiming node-id `{their_node}` from {peer_addr} -- that id already has a live connection (possible duplicate identity / spoofing attempt)");
                    continue;
                }
                info!("swarm-node: peer-hello from {their_node} epoch={their_epoch}");
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
                if identity_already_live(&node, &their_node) {
                    warn!("swarm-node: rejecting peer-welcome claiming node-id `{their_node}` from {peer_addr} -- that id already has a live connection (possible duplicate identity / spoofing attempt)");
                    continue;
                }
                info!("swarm-node: peer-welcome from {their_node}");
                register_peer(&node, &their_node, &mut stream, &peer_ip, their_port);
                send_peer_list(&node, &their_node, &mut stream);
                peer_node_id = Some(their_node);
            }
            Some("peer-list") => {
                handle_peer_list(&node, &msg);
            }
            Some("heartbeat") => {
                // last_seen was already touched above for any message from a
                // known peer; a heartbeat carries no further action.
            }
            Some("sync-hello") => {
                handle_sync_hello(&node, &msg, &mut stream);
            }
            Some("sync-events") => {
                handle_sync_events(&node, &msg);
            }
            Some("sync-complete") => {
                if let Some(from) = msg.field_atom("from") {
                    mark_caught_up(&node, from);
                }
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
            Some("define-task") => {
                handle_define_task(&node, &msg, &mut stream);
            }
            Some("next-best-action") => {
                handle_next_best_action(&node, &msg, &mut stream);
            }
            Some("presence") => {
                handle_presence(&node, &mut stream);
            }
            Some("status") => {
                handle_status(&node, &mut stream);
            }
            Some("metrics") => {
                handle_metrics(&node, &mut stream);
            }
            Some("join") => {
                handle_join(&node, &msg, &mut stream);
            }
            Some("leave") => {
                handle_leave(&node, &mut stream);
            }
            Some("list-members") => {
                handle_list_members(&node, &mut stream);
            }
            Some("sync-tasks") => {
                handle_sync_tasks(&node, &msg, &mut stream);
            }
            Some("compact") => {
                handle_compact(&node, &mut stream);
            }
            other => {
                warn!("swarm-node: unrecognized message head {other:?} from {peer_addr}");
            }
        }
    }
    if let Some(id) = peer_node_id {
        node.peers.lock().unwrap().remove(&id);
        node.last_seen.lock().unwrap().remove(&id);
        info!("swarm-node: connection to {id} closed");
    }
}

/// Cheap partial mitigation for node-id spoofing (SWARM-NODE-IDENTITY-
/// VERIFICATION): Tailscale authenticates which *device* is on the
/// tailnet, but nothing in our own protocol ties a claimed `node-id` to
/// that device — before M0.11's real cross-machine deployment this
/// didn't matter (only localhost processes we already trusted could ever
/// claim an id), now it does. This can't verify identity cryptographically
/// (that needs the deferred node-id = hash(public-key) work), but it
/// closes the cheapest version of the hole: a second connection claiming
/// an id that already has a demonstrably-live connection (recent
/// `last_seen`) is refused rather than silently overwriting it — silently
/// overwriting is what would let a spoofed peer hijack an existing
/// voter's identity mid-session and start voting as them.
///
/// A stale entry (no traffic within `2 * HEARTBEAT_INTERVAL`) is treated
/// as abandoned and allowed to be reclaimed — that's the ordinary
/// reconnect-after-restart case, not spoofing, and must keep working.
fn identity_already_live(node: &Arc<Node>, their_node: &str) -> bool {
    let has_connection = node.peers.lock().unwrap().contains_key(their_node);
    if !has_connection {
        return false;
    }
    match node.last_seen.lock().unwrap().get(their_node) {
        Some(seen) => seen.elapsed() < HEARTBEAT_INTERVAL * 2,
        None => false,
    }
}

/// Records a freshly-handshaken peer: keeps its live stream for broadcast
/// and, if it told us its listen port, its dialable address for gossip.
fn register_peer(node: &Arc<Node>, their_node: &str, stream: &mut TcpStream, their_ip: &str, their_port: u16) {
    if let Ok(clone) = stream.try_clone() {
        node.peers.lock().unwrap().insert(their_node.to_string(), clone);
    }
    node.last_seen.lock().unwrap().insert(their_node.to_string(), Instant::now());
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
            info!("swarm-node: learned of {id} at {ip}:{port} via gossip, dialing");
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
        info!(
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
    } else {
        // Nothing missing is still a definitive answer — the requester needs
        // *some* reply to know it has fully caught up, not just silence.
        send(
            stream,
            &Sexp::list(vec![Sexp::atom("sync-complete"), Sexp::list(vec![Sexp::atom("from"), Sexp::atom(&node.identity.node_id)])]),
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
        info!("swarm-node: applied {applied} event(s) from anti-entropy sync");
    }
    if let Some(from) = msg.field_atom("from") {
        mark_caught_up(node, from);
    }
}

/// A definitive sync answer from `from_node` (whether or not it carried any
/// events) means we're no longer in the "still discovering the swarm's
/// history" state — see `Node::synced` and the `claim-task` gate on it.
fn mark_caught_up(node: &Arc<Node>, from_node: &str) {
    let was_synced = node.synced();
    node.caught_up_with.lock().unwrap().insert(from_node.to_string());
    if !was_synced && node.synced() {
        info!("swarm-node: caught up with the swarm via {from_node}, ready to claim work");
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
/// Two gates must both pass: the fencing check (proposed generation is
/// exactly the next one after what we've derived locally, task not already
/// held/completed) and the promise check (we haven't already voted yes for
/// this task at this-or-higher generation within `PROMISE_TTL`). The promise
/// is what actually excludes concurrent proposals — the fencing check alone
/// only rejects proposals *after* a commit lands; two proposers racing
/// before either commits would both pass fencing but only one can win the
/// promise.
fn handle_claim_proposal(node: &Arc<Node>, msg: &Sexp, stream: &mut TcpStream) {
    let task = msg.field_atom("task").unwrap_or("").to_string();
    let agent = msg.field_atom("agent").unwrap_or("").to_string();
    let generation: u64 = msg.field_atom("generation").and_then(|s| s.parse().ok()).unwrap_or(0);

    let journal = node.journal.lock().unwrap();
    let current = state::task_state(&journal, &task);
    drop(journal);

    let fencing_ok = !current.completed && current.holder.is_none() && generation == current.generation + 1;

    let mut promises = node.promised.lock().unwrap();
    let promise_free = match promises.get(&task) {
        Some((promised_gen, at)) => *promised_gen < generation || at.elapsed() > PROMISE_TTL,
        None => true,
    };
    let vote = fencing_ok && promise_free;
    if vote {
        promises.insert(task.clone(), (generation, Instant::now()));
    }
    drop(promises);

    info!(
        "swarm-node: vote {} on claim-proposal task={task} agent={agent} generation={generation} (local gen={}, holder={:?}, promise_free={promise_free})",
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
    let voter = msg.field_atom("voter").unwrap_or("").to_string();
    let vote = msg.field_atom("vote") == Some("yes");
    let key = format!("{task}:{generation}");
    if let Some(tx) = node.pending_votes.lock().unwrap().get(&key) {
        let _ = tx.send((voter, vote));
    }
}

/// Local client op: `(claim-task (task <id>))`. Proposes the next generation
/// to all currently connected peers and only commits (appends
/// `claim-committed` to our own journal) once a majority of the *voter*
/// nodes (self included, if self is a voter) have voted yes within
/// `VOTE_TIMEOUT`. If no membership has been declared via `join` yet
/// (`state::membership` is empty), falls back to treating every connected
/// peer as a voter — this keeps a bare mesh with no explicit roles working
/// exactly as before M0.4 introduced the voter/worker distinction.
fn handle_claim_task(node: &Arc<Node>, msg: &Sexp, stream: &mut TcpStream) {
    if !node.synced() {
        send(
            stream,
            &Sexp::list(vec![
                Sexp::atom("error"),
                Sexp::string("not yet caught up with the swarm (still syncing with peers) — retry shortly"),
            ]),
        );
        return;
    }
    let Some(task) = msg.field_atom("task") else {
        send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string("claim-task requires a `task` field")]));
        return;
    };

    let journal = node.journal.lock().unwrap();
    let current = state::task_state(&journal, task);
    let membership = state::membership(&journal);
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

    let declared_voters: HashSet<String> = membership.iter().filter(|(_, m)| m.present && state::is_voter(m)).map(|(id, _)| id.clone()).collect();
    let connected: Vec<String> = node.peers.lock().unwrap().keys().cloned().collect();
    let (voting_peers, self_votes): (Vec<String>, usize) = if declared_voters.is_empty() {
        // No membership declared anywhere yet: legacy behavior, everyone connected counts.
        (connected, 1)
    } else {
        let self_is_voter = declared_voters.contains(&node.identity.node_id);
        (connected.into_iter().filter(|id| declared_voters.contains(id)).collect(), if self_is_voter { 1 } else { 0 })
    };
    let total_nodes = (voting_peers.len() + self_votes).max(1);
    let quorum = total_nodes / 2 + 1;

    let generation = current.generation + 1;
    let key = format!("{task}:{generation}");
    let (tx, rx) = mpsc::channel::<(String, bool)>();
    node.pending_votes.lock().unwrap().insert(key.clone(), tx);

    broadcast_to_peers(
        node,
        &Sexp::list(vec![
            Sexp::atom("claim-proposal"),
            Sexp::list(vec![Sexp::atom("task"), Sexp::atom(task)]),
            Sexp::list(vec![Sexp::atom("agent"), Sexp::atom(&node.identity.node_id)]),
            Sexp::list(vec![Sexp::atom("generation"), Sexp::atom(generation.to_string())]),
        ]),
    );

    let mut yes_votes = self_votes;
    let mut counted_responses = 0;
    let deadline = Instant::now() + VOTE_TIMEOUT;
    while yes_votes < quorum && counted_responses < voting_peers.len() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        match rx.recv_timeout(remaining) {
            Ok((voter, vote)) if voting_peers.contains(&voter) => {
                counted_responses += 1;
                if vote {
                    yes_votes += 1;
                }
            }
            Ok(_) => {} // vote from a non-voter (or unknown sender): not tallied
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
                    "quorum not reached for `{task}` generation {generation}: {yes_votes}/{quorum} needed (of {total_nodes} voter nodes)"
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

/// Local client op: `(define-task (task <id>) (priority <n>) (capabilities
/// (a b)) (depends-on (x y)) (description "..."))`. Task metadata is a fact
/// like everything else (`task-defined`), so it replicates the same way
/// claims do — no separate sync path needed.
fn handle_define_task(node: &Arc<Node>, msg: &Sexp, stream: &mut TcpStream) {
    let Some(task) = msg.field_atom("task") else {
        send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string("define-task requires a `task` field")]));
        return;
    };
    let priority = msg.field_atom("priority").unwrap_or("1");
    let capabilities = msg.field("capabilities").and_then(|f| f.first()).cloned().unwrap_or(Sexp::List(vec![]));
    let depends_on = msg.field("depends-on").and_then(|f| f.first()).cloned().unwrap_or(Sexp::List(vec![]));
    let description = msg.field_atom("description");

    let mut payload_fields = vec![
        Sexp::list(vec![Sexp::atom("task"), Sexp::atom(task)]),
        Sexp::list(vec![Sexp::atom("priority"), Sexp::atom(priority)]),
        Sexp::list(vec![Sexp::atom("capabilities"), capabilities]),
        Sexp::list(vec![Sexp::atom("depends-on"), depends_on]),
    ];
    if let Some(d) = description {
        payload_fields.push(Sexp::list(vec![Sexp::atom("description"), Sexp::string(d)]));
    }
    let payload = Sexp::list(payload_fields);

    let lamport = node.tick_lamport(0);
    let mut journal = node.journal.lock().unwrap();
    let seq = journal.next_seq(&node.identity.node_id);
    let event = Event { node: node.identity.node_id.clone(), seq, lamport, typ: "task-defined".to_string(), payload };
    match journal.append(event.clone()) {
        Ok(()) => {
            drop(journal);
            send(stream, &Sexp::list(vec![Sexp::atom("ok"), Sexp::list(vec![Sexp::atom("task"), Sexp::atom(task)])]));
            broadcast_event(node, &event, None);
        }
        Err(e) => send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string(format!("journal append failed: {e}"))])),
    }
}

/// Local client op: `(next-best-action (capabilities (a b)))`. Picks the
/// highest-scoring unclaimed, uncompleted, dependency-satisfied task whose
/// required capabilities are a subset of the caller's. Capability mismatch
/// is a hard gate (excluded, not down-ranked) — matches the `:9999` design.
/// If `capabilities` is omitted, falls back to this node's own declared
/// capabilities from `join` (M0.4) rather than an empty set — a joined
/// agent shouldn't have to repeat its capabilities on every request.
fn handle_next_best_action(node: &Arc<Node>, msg: &Sexp, stream: &mut TcpStream) {
    let journal = node.journal.lock().unwrap();
    let capabilities = match msg.field("capabilities").and_then(|f| f.first()) {
        Some(Sexp::List(items)) => items
            .iter()
            .filter_map(|i| match i {
                Sexp::Atom(s) | Sexp::Str(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => state::membership(&journal).get(&node.identity.node_id).map(|m| m.capabilities.clone()).unwrap_or_default(),
    };
    let best = state::next_best_action(&journal, &capabilities);
    drop(journal);
    match best {
        Some((task, def, ts)) => send(
            stream,
            &Sexp::list(vec![
                Sexp::atom("next-best-action"),
                Sexp::list(vec![Sexp::atom("task"), Sexp::atom(&task)]),
                Sexp::list(vec![Sexp::atom("priority"), Sexp::atom(def.priority.to_string())]),
                Sexp::list(vec![Sexp::atom("generation"), Sexp::atom(ts.generation.to_string())]),
                Sexp::list(vec![
                    Sexp::atom("description"),
                    match &def.description {
                        Some(d) => Sexp::string(d),
                        None => Sexp::List(vec![]),
                    },
                ]),
            ]),
        ),
        None => send(stream, &Sexp::list(vec![Sexp::atom("next-best-action"), Sexp::List(vec![])])),
    }
}

/// Local client op: `(presence)`. Derived live from currently-open
/// connections rather than the event log — unlike claims and evidence,
/// "is this node up right now" is inherently ephemeral and shouldn't
/// survive a restart as a stale fact, so it deliberately isn't durable.
fn handle_presence(node: &Arc<Node>, stream: &mut TcpStream) {
    send(stream, &presence_sexp(node));
}

fn presence_sexp(node: &Arc<Node>) -> Sexp {
    let mut ids: Vec<String> = node.peers.lock().unwrap().keys().cloned().collect();
    ids.push(node.identity.node_id.clone());
    ids.sort();
    Sexp::list(vec![Sexp::atom("presence"), Sexp::list(ids.into_iter().map(Sexp::atom).collect())])
}

/// Local client op: `(status)`. One round trip instead of three —
/// `presence` + `list-members` + `list-task-state` bundled together, for
/// whoever's checking swarm health (a human, or an agent deciding what to
/// do next) without stitching three separate replies together by hand.
fn handle_status(node: &Arc<Node>, stream: &mut TcpStream) {
    let presence = presence_sexp(node);

    let journal = node.journal.lock().unwrap();
    let members = state::membership(&journal);
    let mut member_ids: Vec<&String> = members.keys().collect();
    member_ids.sort();
    let members_sexp = Sexp::list(vec![
        Sexp::atom("members"),
        Sexp::list(
            member_ids
                .into_iter()
                .map(|id| {
                    let m = &members[id];
                    Sexp::list(vec![
                        Sexp::list(vec![Sexp::atom("node"), Sexp::atom(id)]),
                        Sexp::list(vec![Sexp::atom("present"), Sexp::atom(if m.present { "t" } else { "nil" })]),
                        Sexp::list(vec![Sexp::atom("roles"), Sexp::list(m.roles.iter().map(Sexp::atom).collect())]),
                        Sexp::list(vec![Sexp::atom("capabilities"), Sexp::list(m.capabilities.iter().map(Sexp::atom).collect())]),
                    ])
                })
                .collect(),
        ),
    ]);
    let tasks_sexp = Sexp::list(vec![
        Sexp::atom("task-states"),
        Sexp::list(
            state::all_task_ids(&journal)
                .iter()
                .map(|task| task_state_sexp(task, &state::task_state(&journal, task)))
                .collect(),
        ),
    ]);
    drop(journal);

    send(
        stream,
        &Sexp::list(vec![
            Sexp::atom("status"),
            Sexp::list(vec![Sexp::atom("node"), Sexp::atom(&node.identity.node_id)]),
            Sexp::list(vec![Sexp::atom("epoch"), Sexp::atom(node.identity.epoch.to_string())]),
            Sexp::list(vec![Sexp::atom("synced"), Sexp::atom(if node.synced() { "t" } else { "nil" })]),
            presence,
            members_sexp,
            tasks_sexp,
        ]),
    );
}

/// Local client op: `(metrics)`. A handful of small, fixed fields meant
/// to be polled repeatedly and diffed/graphed over time (e.g. by
/// `SWARM-STATUS-DASHBOARD`) — deliberately lighter than `(status)`,
/// which re-serializes the full task/member list on every call and gets
/// more expensive as the swarm grows. No new derived-state computation
/// beyond what `(status)`/`(presence)` already do; this just bundles the
/// cheap scalar facts on their own.
fn handle_metrics(node: &Arc<Node>, stream: &mut TcpStream) {
    let journal = node.journal.lock().unwrap();
    let event_count = journal.events.len();
    // Report the *directory* the operator passed as --data-dir, not the
    // events.log file path itself — that's what a `--data-dir <this>` on
    // a restart actually needs (SWARM-NODE-DATA-DIR-DISCOVERY: this
    // replaces having to `find / -name events.log` blind when a running
    // node's --data-dir was never recorded anywhere).
    let data_dir = journal.path().parent().map(|p| p.display().to_string()).unwrap_or_else(|| journal.path().display().to_string());
    drop(journal);
    let peer_count = node.peers.lock().unwrap().len();
    let uptime_secs = node.started_at.elapsed().as_secs();

    send(
        stream,
        &Sexp::list(vec![
            Sexp::atom("metrics"),
            Sexp::list(vec![Sexp::atom("node"), Sexp::atom(&node.identity.node_id)]),
            Sexp::list(vec![Sexp::atom("epoch"), Sexp::atom(node.identity.epoch.to_string())]),
            Sexp::list(vec![Sexp::atom("uptime-secs"), Sexp::atom(uptime_secs.to_string())]),
            Sexp::list(vec![Sexp::atom("event-count"), Sexp::atom(event_count.to_string())]),
            Sexp::list(vec![Sexp::atom("peer-count"), Sexp::atom(peer_count.to_string())]),
            Sexp::list(vec![Sexp::atom("synced"), Sexp::atom(if node.synced() { "t" } else { "nil" })]),
            Sexp::list(vec![Sexp::atom("data-dir"), Sexp::string(data_dir)]),
        ]),
    );
}

/// Local client op: `(join (capabilities (a b)) (roles (worker)))`.
/// Declares this agent's capabilities/roles as an `agent-joined` fact — a
/// durable, replicated statement of "I am part of this swarm and here is
/// what I can do", independent of any one connection. Roles default to
/// `(worker)` when omitted; only a node with an explicit `voter` role
/// counts toward `claim-task` quorum (see `handle_claim_task`).
fn handle_join(node: &Arc<Node>, msg: &Sexp, stream: &mut TcpStream) {
    let capabilities = msg.field("capabilities").and_then(|f| f.first()).cloned().unwrap_or(Sexp::List(vec![]));
    let roles = msg.field("roles").and_then(|f| f.first()).cloned().unwrap_or(Sexp::List(vec![Sexp::atom("worker")]));
    let payload = Sexp::list(vec![
        Sexp::list(vec![Sexp::atom("node"), Sexp::atom(&node.identity.node_id)]),
        Sexp::list(vec![Sexp::atom("epoch"), Sexp::atom(node.identity.epoch.to_string())]),
        Sexp::list(vec![Sexp::atom("capabilities"), capabilities]),
        Sexp::list(vec![Sexp::atom("roles"), roles]),
    ]);
    let lamport = node.tick_lamport(0);
    let mut journal = node.journal.lock().unwrap();
    let seq = journal.next_seq(&node.identity.node_id);
    let event = Event { node: node.identity.node_id.clone(), seq, lamport, typ: "agent-joined".to_string(), payload };
    match journal.append(event.clone()) {
        Ok(()) => {
            drop(journal);
            send(stream, &Sexp::list(vec![Sexp::atom("ok"), Sexp::list(vec![Sexp::atom("node"), Sexp::atom(&node.identity.node_id)])]));
            broadcast_event(node, &event, None);
        }
        Err(e) => send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string(format!("journal append failed: {e}"))])),
    }
}

/// Local client op: `(leave)`. Records `agent-left` — membership history is
/// kept, not erased, matching the immutable-facts philosophy; `present`
/// just flips to false in the derived view.
fn handle_leave(node: &Arc<Node>, stream: &mut TcpStream) {
    let payload = Sexp::list(vec![
        Sexp::list(vec![Sexp::atom("node"), Sexp::atom(&node.identity.node_id)]),
        Sexp::list(vec![Sexp::atom("epoch"), Sexp::atom(node.identity.epoch.to_string())]),
    ]);
    let lamport = node.tick_lamport(0);
    let mut journal = node.journal.lock().unwrap();
    let seq = journal.next_seq(&node.identity.node_id);
    let event = Event { node: node.identity.node_id.clone(), seq, lamport, typ: "agent-left".to_string(), payload };
    match journal.append(event.clone()) {
        Ok(()) => {
            drop(journal);
            send(stream, &Sexp::list(vec![Sexp::atom("ok"), Sexp::list(vec![Sexp::atom("node"), Sexp::atom(&node.identity.node_id)])]));
            broadcast_event(node, &event, None);
        }
        Err(e) => send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string(format!("journal append failed: {e}"))])),
    }
}

/// Same lesson as `:9999`'s `sync-tasks`/`sync-milestone`/`validate-tasks`:
/// a relative path resolves against *this process's* cwd, not the
/// caller's, and would silently sync whatever unrelated file happens to
/// exist there. Rejected outright.
fn require_absolute_path(path: &str) -> Result<(), String> {
    if std::path::Path::new(path).is_absolute() {
        Ok(())
    } else {
        Err(format!(
            "`file` must be an absolute path — `{path}` would resolve against this node's own working directory, not the caller's, and silently read the wrong file"
        ))
    }
}

/// Local client op: `(sync-tasks (file "/absolute/path/to/tasks.my"))`.
/// Reads the same durable `tasks.my` format `:9999` reads, and emits a
/// `task-defined` fact per entry (plus a `task-completed` fact for any
/// entry already marked `done` — bulk-importing pre-existing ground truth
/// from durable evidence bypasses the live claim/quorum flow entirely,
/// since there's no real-time contention to arbitrate for work that's
/// already finished).
fn handle_sync_tasks(node: &Arc<Node>, msg: &Sexp, stream: &mut TcpStream) {
    let Some(path) = msg.field_atom("file") else {
        send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string("sync-tasks requires a `file` field")]));
        return;
    };
    if let Err(e) = require_absolute_path(path) {
        send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string(e)]));
        return;
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string(format!("could not read `{path}`: {e}"))]));
            return;
        }
    };
    let tasks = match tasks_file::parse_tasks_file(&text) {
        Ok(t) => t,
        Err(e) => {
            send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string(format!("parse error in `{path}`: {e}"))]));
            return;
        }
    };

    let mut defined = 0;
    let mut completed = 0;
    for t in &tasks {
        let mut fields = vec![
            Sexp::list(vec![Sexp::atom("task"), Sexp::atom(&t.id)]),
            Sexp::list(vec![Sexp::atom("priority"), Sexp::atom(t.priority.to_string())]),
            Sexp::list(vec![Sexp::atom("capabilities"), Sexp::list(t.capabilities.iter().map(Sexp::atom).collect())]),
            Sexp::list(vec![Sexp::atom("depends-on"), Sexp::list(t.depends_on.iter().map(Sexp::atom).collect())]),
        ];
        if let Some(d) = &t.description {
            fields.push(Sexp::list(vec![Sexp::atom("description"), Sexp::string(d)]));
        }
        let payload = Sexp::list(fields);
        if append_local_fact(node, "task-defined", payload).is_ok() {
            defined += 1;
        }
        if t.done {
            let journal = node.journal.lock().unwrap();
            let generation = state::task_state(&journal, &t.id).generation;
            drop(journal);
            if append_task_fact(node, "task-completed", &t.id, generation).is_ok() {
                completed += 1;
            }
        }
    }
    send(
        stream,
        &Sexp::list(vec![
            Sexp::atom("ok"),
            Sexp::list(vec![Sexp::atom("defined"), Sexp::atom(defined.to_string())]),
            Sexp::list(vec![Sexp::atom("marked-done"), Sexp::atom(completed.to_string())]),
        ]),
    );
}

/// Shared by `sync-tasks` and (indirectly) `define-task`: append an
/// arbitrary fact to our own journal and gossip it like any other event.
fn append_local_fact(node: &Arc<Node>, typ: &str, payload: Sexp) -> std::io::Result<Event> {
    let lamport = node.tick_lamport(0);
    let mut journal = node.journal.lock().unwrap();
    let seq = journal.next_seq(&node.identity.node_id);
    let event = Event { node: node.identity.node_id.clone(), seq, lamport, typ: typ.to_string(), payload };
    journal.append(event.clone())?;
    drop(journal);
    broadcast_event(node, &event, None);
    Ok(event)
}

/// Local client op: `(compact)`. Rewrites this node's own on-disk journal
/// to the minimal set of facts that reproduces the current derived state —
/// see `compact.rs` for the safety argument for why this can't corrupt any
/// peer's view even though it changes what's on disk. Broadcasts nothing:
/// peers only ever pull via `sync-hello`/`sync-events`, and after
/// compaction that path already serves the smaller equivalent set.
fn handle_compact(node: &Arc<Node>, stream: &mut TcpStream) {
    let mut journal = node.journal.lock().unwrap();
    match compact::compact(&mut journal, &node.identity.node_id) {
        Ok((before, after)) => {
            drop(journal);
            info!("swarm-node: compacted journal {before} -> {after} events");
            send(
                stream,
                &Sexp::list(vec![
                    Sexp::atom("ok"),
                    Sexp::list(vec![Sexp::atom("before"), Sexp::atom(before.to_string())]),
                    Sexp::list(vec![Sexp::atom("after"), Sexp::atom(after.to_string())]),
                ]),
            );
        }
        Err(e) => {
            drop(journal);
            send(stream, &Sexp::list(vec![Sexp::atom("error"), Sexp::string(format!("compaction failed: {e}"))]));
        }
    }
}

fn handle_list_members(node: &Arc<Node>, stream: &mut TcpStream) {
    let journal = node.journal.lock().unwrap();
    let members = state::membership(&journal);
    drop(journal);
    let mut ids: Vec<&String> = members.keys().collect();
    ids.sort();
    let entries: Vec<Sexp> = ids
        .into_iter()
        .map(|id| {
            let m = &members[id];
            Sexp::list(vec![
                Sexp::list(vec![Sexp::atom("node"), Sexp::atom(id)]),
                Sexp::list(vec![Sexp::atom("present"), Sexp::atom(if m.present { "t" } else { "nil" })]),
                Sexp::list(vec![Sexp::atom("roles"), Sexp::list(m.roles.iter().map(Sexp::atom).collect())]),
                Sexp::list(vec![Sexp::atom("capabilities"), Sexp::list(m.capabilities.iter().map(Sexp::atom).collect())]),
            ])
        })
        .collect();
    send(stream, &Sexp::list(vec![Sexp::atom("members"), Sexp::list(entries)]));
}

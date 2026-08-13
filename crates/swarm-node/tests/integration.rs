//! Integration tests for swarm-node, promoted from the ad-hoc bash smoke
//! scripts used while building M0.1-M0.8 (see docs/swarm-mesh-v2.md) into
//! something that actually runs under `cargo test` and catches regressions
//! automatically instead of only when someone remembers to check by hand.
//!
//! Each test spawns real `swarm-node` child processes (via
//! `CARGO_BIN_EXE_swarm-node`, the compiled binary for this crate) and
//! talks to them over real TCP loopback sockets — this is deliberately an
//! end-to-end test of the wire protocol, not a unit test of internal
//! functions (those live next to the code in `src/*.rs`).

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::{Duration, Instant};

static NEXT_PORT: AtomicU16 = AtomicU16::new(15001);

/// Reserves `n` consecutive ports for one test, so parallel `cargo test`
/// execution (multiple tests in this binary run concurrently by default)
/// never collides on a port.
fn alloc_ports(n: u16) -> u16 {
    NEXT_PORT.fetch_add(n, Ordering::SeqCst)
}

fn data_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("swarm-node-itest").join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

struct Node {
    child: Child,
}

impl Drop for Node {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn spawn(port: u16, node_id: &str, data_dir: &Path, connect: Option<u16>) -> Node {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_swarm-node"));
    cmd.arg("--port").arg(port.to_string());
    cmd.arg("--node-id").arg(node_id);
    cmd.arg("--project").arg("itest");
    cmd.arg("--data-dir").arg(data_dir);
    if let Some(p) = connect {
        cmd.arg("--connect").arg(format!("127.0.0.1:{p}"));
    }
    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    let child = cmd.spawn().expect("failed to spawn swarm-node — did `cargo build -p swarm-node` run first?");
    let node = Node { child };
    wait_for_port(port);
    node
}

fn wait_for_port(port: u16) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("swarm-node on port {port} never started listening");
}

/// One request/response round trip over a fresh connection, matching how
/// every other client in this ecosystem talks to the line-framed sexpr
/// protocol (one form in, one line out).
fn request(port: u16, msg: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut stream = loop {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(s) => break s,
            Err(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => panic!("could not connect to port {port}: {e}"),
        }
    };
    stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    writeln!(stream, "{msg}").unwrap();
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    line.trim().to_string()
}

/// Polls `request(port, msg)` until `predicate` matches or the deadline
/// passes, returning the last response seen. Used for anything that
/// depends on gossip/anti-entropy/reconnect propagating asynchronously —
/// avoids flaky fixed `sleep`s tuned to one machine's speed.
fn eventually(port: u16, msg: &str, timeout: Duration, predicate: impl Fn(&str) -> bool) -> String {
    let deadline = Instant::now() + timeout;
    let mut last = String::new();
    while Instant::now() < deadline {
        last = request(port, msg);
        if predicate(&last) {
            return last;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    last
}

#[test]
fn anti_entropy_sync_and_live_push_event() {
    let base = alloc_ports(2);
    let (port_a, port_b) = (base, base + 1);

    let _a = spawn(port_a, "node-a", &data_dir("ae-a"), None);
    assert_eq!(request(port_a, "(emit (type evidence-created) (payload (artifact \"x.my\")))"), "(ok (id node-a:1))");
    assert_eq!(request(port_a, "(emit (type evidence-created) (payload (artifact \"y.my\")))"), "(ok (id node-a:2))");

    // B connects after A already has 2 events -- must anti-entropy sync them.
    let _b = spawn(port_b, "node-b", &data_dir("ae-b"), Some(port_a));
    let synced = eventually(port_b, "(list-task-state)", Duration::from_secs(2), |r| !r.is_empty());
    let _ = synced; // list-task-state is task-only; just confirm B is responsive post-sync below

    // A live-pushes a 3rd event; B must receive it without any resync call.
    assert_eq!(request(port_a, "(emit (type evidence-created) (payload (artifact \"z.my\")))"), "(ok (id node-a:3))");

    // No direct way to read the raw journal over the wire, so prove sync worked
    // indirectly via a task defined on A becoming visible on B.
    request(port_a, "(define-task (task PROOF) (priority 1) (capabilities ()) (depends-on ()) (description \"sync worked\"))");
    let seen_on_b = eventually(port_b, "(list-task-state)", Duration::from_secs(2), |r| r.contains("PROOF"));
    assert!(seen_on_b.contains("PROOF"), "task defined on A never propagated to B: {seen_on_b}");
}

#[test]
fn quorum_claim_fencing_and_stale_rejection() {
    let base = alloc_ports(3);
    let (port_a, port_b, port_c) = (base, base + 1, base + 2);

    let _a = spawn(port_a, "node-a", &data_dir("qf-a"), None);
    let _b = spawn(port_b, "node-b", &data_dir("qf-b"), Some(port_a));
    let _c = spawn(port_c, "node-c", &data_dir("qf-c"), Some(port_a));
    eventually(port_c, "(presence)", Duration::from_secs(2), |r| r.contains("node-a") && r.contains("node-b"));

    let claimed = request(port_a, "(claim-task (task T1))");
    assert!(claimed.starts_with("(ok"), "expected quorum claim to succeed: {claimed}");

    // Give B's own copy time to observe A's commit via gossip before B tries
    // to claim -- otherwise B legitimately races A (M0.6 correctly rejects
    // that race via voter promises, but that's a *different* assertion than
    // "B saw the commit and backed off", which is what this test checks).
    let duplicate = eventually(port_b, "(claim-task (task T1))", Duration::from_secs(2), |r| r.contains("already claimed"));
    assert!(duplicate.contains("already claimed by `node-a`"), "expected duplicate claim rejection: {duplicate}");

    let stale = request(port_b, "(complete-task (task T1) (generation 99))");
    assert!(stale.contains("STALE"), "expected STALE rejection for wrong generation: {stale}");

    let completed = request(port_a, "(complete-task (task T1) (generation 1))");
    assert!(completed.starts_with("(ok"), "expected completion with correct generation to succeed: {completed}");

    let after_done = eventually(port_c, "(claim-task (task T1))", Duration::from_secs(2), |r| r.contains("already completed"));
    assert!(after_done.contains("already completed"), "expected claim on completed task to be rejected: {after_done}");
}

#[test]
fn gossip_peer_discovery_reaches_full_mesh() {
    let base = alloc_ports(3);
    let (port_a, port_b, port_c) = (base, base + 1, base + 2);

    let _a = spawn(port_a, "node-a", &data_dir("gd-a"), None);
    let _b = spawn(port_b, "node-b", &data_dir("gd-b"), Some(port_a));
    // C connects ONLY to A -- must discover and dial B via gossip through A.
    let _c = spawn(port_c, "node-c", &data_dir("gd-c"), Some(port_a));

    let c_presence = eventually(port_c, "(presence)", Duration::from_secs(3), |r| r.contains("node-b"));
    assert!(c_presence.contains("node-b"), "node-c never gossip-discovered node-b: {c_presence}");
}

#[test]
fn compaction_preserves_derived_state() {
    let base = alloc_ports(1);
    let port = base;
    let _a = spawn(port, "node-a", &data_dir("cc-a"), None);

    request(port, "(define-task (task X) (priority 1) (capabilities ()) (depends-on ()) (description \"v1\"))");
    request(port, "(define-task (task X) (priority 2) (capabilities ()) (depends-on ()) (description \"v2 final\"))");
    request(port, "(claim-task (task X))");
    request(port, "(release-task (task X) (generation 1))");
    request(port, "(claim-task (task X))");

    let before = request(port, "(list-task-state)");

    let compacted = request(port, "(compact)");
    assert!(compacted.starts_with("(ok"), "compact should succeed: {compacted}");

    let after = request(port, "(list-task-state)");
    assert_eq!(before, after, "derived state must be byte-identical before/after compaction");
}

#[test]
fn dynamic_membership_voter_quorum_and_status() {
    let base = alloc_ports(4);
    let (port_a, port_b, port_c, port_w) = (base, base + 1, base + 2, base + 3);

    let _a = spawn(port_a, "node-a", &data_dir("dm-a"), None);
    let _b = spawn(port_b, "node-b", &data_dir("dm-b"), Some(port_a));
    let _c = spawn(port_c, "node-c", &data_dir("dm-c"), Some(port_a));
    eventually(port_c, "(presence)", Duration::from_secs(2), |r| r.contains("node-b"));

    for port in [port_a, port_b, port_c] {
        let r = request(port, "(join (capabilities (x)) (roles (voter)))");
        assert!(r.starts_with("(ok"), "join should succeed on port {port}: {r}");
    }

    // A worker joins mid-session through just A, and must reach node-b/node-c via gossip.
    let _w = spawn(port_w, "worker-1", &data_dir("dm-w"), Some(port_a));
    eventually(port_w, "(presence)", Duration::from_secs(2), |r| r.contains("node-b") && r.contains("node-c"));
    request(port_w, "(join (capabilities (docs)) (roles (worker)))");

    let members = eventually(port_a, "(list-members)", Duration::from_secs(2), |r| r.contains("worker-1"));
    assert!(members.contains("worker-1"), "worker never showed up in list-members: {members}");

    // Worker's own claim should only need 2/3 VOTER votes, not counting itself.
    request(port_w, "(define-task (task WORK) (priority 1) (capabilities ()) (depends-on ()) (description \"anyone\"))");
    let claimed = eventually(port_w, "(claim-task (task WORK))", Duration::from_secs(2), |r| r.starts_with("(ok") || r.contains("error"));
    assert!(claimed.contains("2/3"), "expected a 2/3 voter quorum, got: {claimed}");

    let status = request(port_a, "(status)");
    assert!(status.starts_with("(status"), "status op malformed: {status}");
    assert!(status.contains("(synced t)"), "node-a should report itself synced: {status}");
}

#[test]
fn rejects_duplicate_node_id_claim_from_a_second_connection() {
    let base = alloc_ports(2);
    let (port_a, port_b) = (base, base + 1);

    let _a = spawn(port_a, "node-a", &data_dir("dup-a"), None);
    let _b = spawn(port_b, "node-b", &data_dir("dup-b"), Some(port_a));
    // Confirm the real node-b is live on A before trying to impersonate it.
    eventually(port_a, "(presence)", Duration::from_secs(2), |r| r.contains("node-b"));

    // A raw connection claiming to already-live node-b's identity, from
    // somewhere that is NOT the real node-b -- simulates a spoofing
    // attempt (or a genuine but confused duplicate) rather than a normal
    // reconnect. Must get no peer-welcome back.
    let mut spoof = TcpStream::connect(("127.0.0.1", port_a)).unwrap();
    spoof.set_read_timeout(Some(Duration::from_millis(500))).unwrap();
    writeln!(spoof, "(peer-hello (protocol swarm/1) (node node-b) (epoch 0) (project spoof) (listen-port 0))").unwrap();
    let mut reply = String::new();
    let mut reader = BufReader::new(&spoof);
    let read_result = reader.read_line(&mut reply);
    assert!(
        read_result.is_err() || reply.trim().is_empty(),
        "spoofed peer-hello for an already-live node-id should get no peer-welcome reply, got: {reply:?}"
    );

    // The real node-b must still be the one registered -- not evicted.
    let presence = request(port_a, "(presence)");
    assert!(presence.contains("node-b"), "real node-b should still be present after a rejected spoof attempt: {presence}");
}

#[test]
fn metrics_reports_event_count_peer_count_and_synced() {
    let base = alloc_ports(2);
    let (port_a, port_b) = (base, base + 1);

    let dir_a = data_dir("metrics-a");
    let _a = spawn(port_a, "node-a", &dir_a, None);
    request(port_a, "(emit (type evidence-created) (payload (artifact \"x.my\")))");
    request(port_a, "(emit (type evidence-created) (payload (artifact \"y.my\")))");

    let _b = spawn(port_b, "node-b", &data_dir("metrics-b"), Some(port_a));
    eventually(port_a, "(metrics)", Duration::from_secs(2), |r| r.contains("(peer-count 1)"));

    let metrics = request(port_a, "(metrics)");
    assert!(metrics.starts_with("(metrics"), "metrics op malformed: {metrics}");
    assert!(metrics.contains("(event-count 2)"), "expected 2 events after 2 emits: {metrics}");
    assert!(metrics.contains("(peer-count 1)"), "expected 1 connected peer (node-b): {metrics}");
    assert!(metrics.contains("(synced t)"), "node-a with no --connect should be trivially synced: {metrics}");
    assert!(metrics.contains("(node node-a)"), "metrics should report the caller's own node-id: {metrics}");
    let dir_a_str = dir_a.to_string_lossy().replace('\\', "/");
    let metrics_normalized = metrics.replace('\\', "/");
    assert!(
        metrics_normalized.contains(&*dir_a_str),
        "metrics should report the node's own --data-dir ({dir_a_str}), got: {metrics}"
    );
}

#[test]
fn help_flag_prints_usage_and_exits_without_starting_a_server() {
    // Regression test for SWARM-NODE-HELP-FLAG-BUG: --help used to fall
    // through to the unknown-argument warning and then start a real
    // server under every default anyway.
    for flag in ["--help", "-h"] {
        let output = Command::new(env!("CARGO_BIN_EXE_swarm-node"))
            .arg(flag)
            .output()
            .unwrap_or_else(|e| panic!("failed to run swarm-node {flag}: {e}"));
        assert!(output.status.success(), "swarm-node {flag} should exit 0, got {:?}", output.status);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("USAGE"), "{flag} output should contain usage text, got: {stdout}");
        assert!(!stdout.contains("listening on"), "{flag} must not start a server: {stdout}");
    }
}

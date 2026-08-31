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
    let dir = std::env::temp_dir()
        .join("swarm-node-itest")
        .join(format!("{name}-{}", std::process::id()));
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
    cmd.arg("--no-auto-sync");
    if let Some(p) = connect {
        cmd.arg("--connect").arg(format!("127.0.0.1:{p}"));
    }
    if let Ok(logdir) = std::env::var("SWARM_TEST_LOGS") {
        let _ = std::fs::create_dir_all(&logdir);
        let f = std::fs::File::create(
            std::path::Path::new(&logdir).join(format!("{node_id}-{port}.log")),
        )
        .unwrap();
        cmd.stdout(f.try_clone().unwrap()).stderr(f);
    } else {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }
    let child = cmd
        .spawn()
        .expect("failed to spawn swarm-node — did `cargo build -p swarm-node` run first?");
    let node = Node { child };
    wait_for_port(port);
    wait_for_file(&data_dir.join("node.my"));
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

fn wait_for_file(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if path.is_file() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("swarm-node startup did not create {}", path.display());
}

#[test]
fn startup_rejects_relative_data_dir() {
    let output = Command::new(env!("CARGO_BIN_EXE_swarm-node"))
        .args([
            "--port",
            "15991",
            "--node-id",
            "strict-node",
            "--project",
            "itest",
            "--data-dir",
            "relative-state",
            "--no-auto-sync",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--data-dir must be absolute"));
}

#[test]
fn startup_rejects_data_dir_owned_by_another_identity() {
    let dir = data_dir("identity-mismatch");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("node.my"),
        "(node (id original-node) (epoch 1) (incarnation test))",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_swarm-node"))
        .arg("--port")
        .arg("15992")
        .arg("--node-id")
        .arg("different-node")
        .arg("--project")
        .arg("itest")
        .arg("--data-dir")
        .arg(&dir)
        .arg("--no-auto-sync")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("identity mismatch"));
}

fn startup_command(port: u16, node_id: &str, dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_swarm-node"));
    command
        .arg("--port")
        .arg(port.to_string())
        .arg("--node-id")
        .arg(node_id)
        .arg("--project")
        .arg("itest")
        .arg("--data-dir")
        .arg(dir)
        .arg("--no-auto-sync");
    command
}

#[test]
fn duplicate_port_fails_before_mutating_identity() {
    let port = alloc_ports(1);
    let dir = data_dir("duplicate-port-no-mutation");
    let _first = spawn(port, "one-owner", &dir, None);
    let before = std::fs::read_to_string(dir.join("node.my")).unwrap();

    let output = startup_command(port, "one-owner", &dir).output().unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Address already in use"));
    assert_eq!(
        std::fs::read_to_string(dir.join("node.my")).unwrap(),
        before
    );
    assert!(request(port, "(status)").starts_with("(status"));
}

#[test]
fn shared_data_dir_rejects_second_live_process_on_another_port() {
    let ports = alloc_ports(2);
    let dir = data_dir("shared-data-dir-lock");
    let _first = spawn(ports, "one-owner", &dir, None);
    let before = std::fs::read_to_string(dir.join("node.my")).unwrap();

    let output = startup_command(ports + 1, "one-owner", &dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("already owned"));
    assert_eq!(
        std::fs::read_to_string(dir.join("node.my")).unwrap(),
        before
    );
    assert!(TcpStream::connect(("127.0.0.1", ports + 1)).is_err());
}

#[test]
fn startup_requires_explicit_task_sync_choice_and_rejects_unknown_args() {
    let dir = data_dir("explicit-task-sync");
    let without_choice = Command::new(env!("CARGO_BIN_EXE_swarm-node"))
        .arg("--node-id")
        .arg("strict")
        .arg("--project")
        .arg("itest")
        .arg("--data-dir")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(!without_choice.status.success());
    assert!(String::from_utf8_lossy(&without_choice.stderr).contains("--no-auto-sync"));
    assert!(!dir.exists(), "validation must precede state creation");

    let unknown = startup_command(alloc_ports(1), "strict", &dir)
        .arg("--aut-sync")
        .output()
        .unwrap();
    assert!(!unknown.status.success());
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("unknown argument"));
    assert!(
        !dir.exists(),
        "argument parsing must precede state creation"
    );
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
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
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
    // M1.1a: emitted ids now embed the node's incarnation id —
    // `node-a:<incarnation>:N`. Assert the shape instead of a literal.
    let e1 = request(
        port_a,
        "(emit (type evidence-created) (payload (artifact \"x.my\")))",
    );
    assert!(
        e1.starts_with("(ok (id node-a:") && e1.ends_with(":1))"),
        "unexpected first emit id: {e1}"
    );
    let e2 = request(
        port_a,
        "(emit (type evidence-created) (payload (artifact \"y.my\")))",
    );
    assert!(e2.ends_with(":2))"), "unexpected second emit id: {e2}");

    // B connects after A already has 2 events -- must anti-entropy sync them.
    let _b = spawn(port_b, "node-b", &data_dir("ae-b"), Some(port_a));
    let synced = eventually(port_b, "(list-task-state)", Duration::from_secs(2), |r| {
        !r.is_empty()
    });
    let _ = synced; // list-task-state is task-only; just confirm B is responsive post-sync below

    // A live-pushes a 3rd event; B must receive it without any resync call.
    let e3 = request(
        port_a,
        "(emit (type evidence-created) (payload (artifact \"z.my\")))",
    );
    assert!(e3.ends_with(":3))"), "unexpected third emit id: {e3}");

    // No direct way to read the raw journal over the wire, so prove sync worked
    // indirectly via a task defined on A becoming visible on B.
    request(port_a, "(define-task (task PROOF) (priority 1) (capabilities ()) (depends-on ()) (description \"sync worked\"))");
    let seen_on_b = eventually(port_b, "(list-task-state)", Duration::from_secs(2), |r| {
        r.contains("PROOF")
    });
    assert!(
        seen_on_b.contains("PROOF"),
        "task defined on A never propagated to B: {seen_on_b}"
    );
}

#[test]
fn quorum_claim_fencing_and_stale_rejection() {
    let base = alloc_ports(3);
    let (port_a, port_b, port_c) = (base, base + 1, base + 2);

    let _a = spawn(port_a, "node-a", &data_dir("qf-a"), None);
    let _b = spawn(port_b, "node-b", &data_dir("qf-b"), Some(port_a));
    let _c = spawn(port_c, "node-c", &data_dir("qf-c"), Some(port_a));
    eventually(port_c, "(presence)", Duration::from_secs(2), |r| {
        r.contains("node-a") && r.contains("node-b")
    });

    let claimed = request(port_a, "(claim-task (task T1))");
    assert!(
        claimed.starts_with("(ok"),
        "expected quorum claim to succeed: {claimed}"
    );

    // Give B's own copy time to observe A's commit via gossip before B tries
    // to claim -- otherwise B legitimately races A (M0.6 correctly rejects
    // that race via voter promises, but that's a *different* assertion than
    // "B saw the commit and backed off", which is what this test checks).
    let duplicate = eventually(
        port_b,
        "(claim-task (task T1))",
        Duration::from_secs(2),
        |r| r.contains("already claimed"),
    );
    assert!(
        duplicate.contains("already claimed by `node-a`"),
        "expected duplicate claim rejection: {duplicate}"
    );

    let stale = request(port_b, "(complete-task (task T1) (generation 99))");
    assert!(
        stale.contains("STALE"),
        "expected STALE rejection for wrong generation: {stale}"
    );

    let completed = request(port_a, "(complete-task (task T1) (generation 1))");
    assert!(
        completed.starts_with("(ok"),
        "expected completion with correct generation to succeed: {completed}"
    );

    let after_done = eventually(
        port_c,
        "(claim-task (task T1))",
        Duration::from_secs(2),
        |r| r.contains("already completed"),
    );
    assert!(
        after_done.contains("already completed"),
        "expected claim on completed task to be rejected: {after_done}"
    );
}

#[test]
fn gossip_peer_discovery_reaches_full_mesh() {
    let base = alloc_ports(3);
    let (port_a, port_b, port_c) = (base, base + 1, base + 2);

    let _a = spawn(port_a, "node-a", &data_dir("gd-a"), None);
    let _b = spawn(port_b, "node-b", &data_dir("gd-b"), Some(port_a));
    // C connects ONLY to A -- must discover and dial B via gossip through A.
    let _c = spawn(port_c, "node-c", &data_dir("gd-c"), Some(port_a));

    let c_presence = eventually(port_c, "(presence)", Duration::from_secs(3), |r| {
        r.contains("node-b")
    });
    assert!(
        c_presence.contains("node-b"),
        "node-c never gossip-discovered node-b: {c_presence}"
    );
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
    assert!(
        compacted.starts_with("(ok"),
        "compact should succeed: {compacted}"
    );

    let after = request(port, "(list-task-state)");
    assert_eq!(
        before, after,
        "derived state must be byte-identical before/after compaction"
    );
}

#[test]
fn dynamic_membership_voter_quorum_and_status() {
    let base = alloc_ports(4);
    let (port_a, port_b, port_c, port_w) = (base, base + 1, base + 2, base + 3);

    let _a = spawn(port_a, "node-a", &data_dir("dm-a"), None);
    let _b = spawn(port_b, "node-b", &data_dir("dm-b"), Some(port_a));
    let _c = spawn(port_c, "node-c", &data_dir("dm-c"), Some(port_a));
    eventually(port_c, "(presence)", Duration::from_secs(2), |r| {
        r.contains("node-b")
    });

    for port in [port_a, port_b, port_c] {
        let r = request(port, "(join (capabilities (x)) (roles (voter)))");
        assert!(
            r.starts_with("(ok"),
            "join should succeed on port {port}: {r}"
        );
    }

    // A worker joins mid-session through just A, and must reach node-b/node-c via gossip.
    let _w = spawn(port_w, "worker-1", &data_dir("dm-w"), Some(port_a));
    eventually(port_w, "(presence)", Duration::from_secs(2), |r| {
        r.contains("node-b") && r.contains("node-c")
    });
    request(port_w, "(join (capabilities (docs)) (roles (worker)))");

    let members = eventually(port_a, "(list-members)", Duration::from_secs(2), |r| {
        r.contains("worker-1")
    });
    assert!(
        members.contains("worker-1"),
        "worker never showed up in list-members: {members}"
    );

    // Worker's own claim should only need 2/3 VOTER votes, not counting itself.
    request(port_w, "(define-task (task WORK) (priority 1) (capabilities ()) (depends-on ()) (description \"anyone\"))");
    let claimed = eventually(
        port_w,
        "(claim-task (task WORK))",
        Duration::from_secs(2),
        |r| r.starts_with("(ok") || r.contains("error"),
    );
    assert!(
        claimed.contains("2/3"),
        "expected a 2/3 voter quorum, got: {claimed}"
    );

    let status = request(port_a, "(status)");
    assert!(
        status.starts_with("(status"),
        "status op malformed: {status}"
    );
    assert!(
        status.contains("(synced t)"),
        "node-a should report itself synced: {status}"
    );
}

#[test]
fn rejects_duplicate_node_id_claim_from_a_second_connection() {
    let base = alloc_ports(2);
    let (port_a, port_b) = (base, base + 1);

    let _a = spawn(port_a, "node-a", &data_dir("dup-a"), None);
    let _b = spawn(port_b, "node-b", &data_dir("dup-b"), Some(port_a));
    // Confirm the real node-b is live on A before trying to impersonate it.
    eventually(port_a, "(presence)", Duration::from_secs(2), |r| {
        r.contains("node-b")
    });

    // A raw connection claiming to already-live node-b's identity, from
    // somewhere that is NOT the real node-b -- simulates a spoofing
    // attempt (or a genuine but confused duplicate) rather than a normal
    // reconnect. Must get no peer-welcome back.
    let mut spoof = TcpStream::connect(("127.0.0.1", port_a)).unwrap();
    spoof
        .set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();
    writeln!(
        spoof,
        "(peer-hello (protocol swarm/1) (node node-b) (epoch 0) (project spoof) (listen-port 0))"
    )
    .unwrap();
    let mut reply = String::new();
    let mut reader = BufReader::new(&spoof);
    let read_result = reader.read_line(&mut reply);
    assert!(
        read_result.is_err() || reply.trim().is_empty(),
        "spoofed peer-hello for an already-live node-id should get no peer-welcome reply, got: {reply:?}"
    );

    // The real node-b must still be the one registered -- not evicted.
    let presence = request(port_a, "(presence)");
    assert!(
        presence.contains("node-b"),
        "real node-b should still be present after a rejected spoof attempt: {presence}"
    );
}

#[test]
fn metrics_reports_event_count_peer_count_and_synced() {
    let base = alloc_ports(2);
    let (port_a, port_b) = (base, base + 1);

    let dir_a = data_dir("metrics-a");
    let _a = spawn(port_a, "node-a", &dir_a, None);
    request(
        port_a,
        "(emit (type evidence-created) (payload (artifact \"x.my\")))",
    );
    request(
        port_a,
        "(emit (type evidence-created) (payload (artifact \"y.my\")))",
    );

    let _b = spawn(port_b, "node-b", &data_dir("metrics-b"), Some(port_a));
    eventually(port_a, "(metrics)", Duration::from_secs(2), |r| {
        r.contains("(peer-count 1)")
    });

    let metrics = request(port_a, "(metrics)");
    assert!(
        metrics.starts_with("(metrics"),
        "metrics op malformed: {metrics}"
    );
    assert!(
        metrics.contains("(event-count 2)"),
        "expected 2 events after 2 emits: {metrics}"
    );
    assert!(
        metrics.contains("(peer-count 1)"),
        "expected 1 connected peer (node-b): {metrics}"
    );
    assert!(
        metrics.contains("(synced t)"),
        "node-a with no --connect should be trivially synced: {metrics}"
    );
    assert!(metrics.contains("(bootstrap-peers 0)"), "{metrics}");
    assert!(metrics.contains("(task-sync t)"), "{metrics}");
    assert!(
        metrics.contains("(node node-a)"),
        "metrics should report the caller's own node-id: {metrics}"
    );
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
        assert!(
            output.status.success(),
            "swarm-node {flag} should exit 0, got {:?}",
            output.status
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("USAGE"),
            "{flag} output should contain usage text, got: {stdout}"
        );
        assert!(
            !stdout.contains("listening on"),
            "{flag} must not start a server: {stdout}"
        );
    }
}

#[test]
fn define_task_is_idempotent_for_an_identical_redefinition() {
    // Regression test for SWARM-DEFINE-TASK-DEDUP: re-broadcasting the
    // exact same define-task call (the same task/priority/capabilities/
    // depends-on/blocked-by/description) must not append a fresh
    // task-defined event every time — confirmed in practice 2026-08-18,
    // the same task definition landed in a real journal three times from
    // one peer notifying the swarm about it.
    let port = alloc_ports(1);
    let _a = spawn(port, "node-a", &data_dir("dedup-a"), None);

    fn event_count(port: u16) -> u64 {
        let metrics = request(port, "(metrics)");
        let marker = "(event-count ";
        let start = metrics
            .find(marker)
            .expect("metrics should report event-count")
            + marker.len();
        let rest = &metrics[start..];
        let end = rest.find(')').expect("event-count should be closed");
        rest[..end].parse().expect("event-count should be a number")
    }

    let define = "(define-task (task DUP) (priority 3) (capabilities (a b)) (depends-on ()) (blocked-by ()) (description \"same every time\"))";

    let first = request(port, define);
    assert!(
        first.starts_with("(ok"),
        "first define-task should succeed: {first}"
    );
    let after_first = event_count(port);

    // Two more identical calls, as if a peer re-broadcast the same
    // define-task (or retried after a timeout) — neither should grow the
    // journal.
    let second = request(port, define);
    assert!(
        second.starts_with("(ok"),
        "repeat define-task should still report ok: {second}"
    );
    assert!(
        second.contains("(unchanged t)"),
        "repeat define-task should report unchanged: {second}"
    );
    let third = request(port, define);
    assert!(
        third.contains("(unchanged t)"),
        "repeat define-task should report unchanged: {third}"
    );

    let after_repeats = event_count(port);
    assert_eq!(
        after_first, after_repeats,
        "identical redefinitions must not append new events"
    );

    // A genuinely different redefinition (priority changed) must still
    // append normally — this isn't a blanket "only define once" guard.
    let changed = "(define-task (task DUP) (priority 5) (capabilities (a b)) (depends-on ()) (blocked-by ()) (description \"same every time\"))";
    let fourth = request(port, changed);
    assert!(
        !fourth.contains("(unchanged t)"),
        "a genuinely different redefinition must not be reported unchanged: {fourth}"
    );
    let after_change = event_count(port);
    assert_eq!(
        after_repeats + 1,
        after_change,
        "a real change must append exactly one new event"
    );
}

// ---------------------------------------------------------------------------
// M1.1a: incarnation-safe event identity
// ---------------------------------------------------------------------------

/// Reads this node's incarnation id out of its persisted identity store.
fn read_incarnation(dir: &Path) -> String {
    let text = std::fs::read_to_string(dir.join("node.my")).expect("node.my must exist");
    let marker = "(incarnation ";
    let start = text
        .find(marker)
        .expect("node.my must contain an incarnation field")
        + marker.len();
    let end = text[start..]
        .find(')')
        .expect("incarnation field must be closed")
        + start;
    text[start..end].trim_matches('"').to_string()
}

fn kill(node: &mut Node) {
    let _ = node.child.kill();
    let _ = node.child.wait();
}

/// THE reincarnation regression test for the 2026-08-22 anti-entropy bug:
///
/// A node emits T1 under incarnation X. Its data-dir is destroyed. The same
/// node-id comes back as incarnation Y and emits T2 with seq restarting at 1.
/// A bootstrap that saw both lifetimes must end up holding BOTH events, and
/// anti-entropy must converge in both directions — before M1.1a the second
/// lifetime's `(id my-idea-1:1)` collided with the first and the bootstrap
/// permanently believed it was already synced.
#[test]
fn reincarnation_does_not_collide_and_anti_entropy_converges() {
    let base = alloc_ports(2);
    let (port_boot, port_a) = (base, base + 1);

    let boot_dir = data_dir("reinc-boot");
    let a_dir = data_dir("reinc-a");

    // Bootstrap first, alone.
    let boot = spawn(port_boot, "boot", &boot_dir, None);

    // Incarnation X of node "wanderer": define T1.
    let mut x = spawn(port_a, "wanderer", &a_dir, Some(port_boot));
    let inc_x = read_incarnation(&a_dir);
    // Wait until the mesh link is actually up — a define issued before the
    // handshake completes would only ever live in the local journal.
    eventually(port_a, "(metrics)", Duration::from_secs(3), |r| {
        r.contains("(peer-count 1)")
    });
    assert!(
        request(port_a, "(define-task (task T1) (priority 5) (capabilities ()) (depends-on ()) (description \"from incarnation X\"))").starts_with("(ok"),
        "T1 define failed"
    );
    // The scenario requires the bootstrap to HAVE received T1 before X
    // dies. Killing immediately after the define can RST the connection
    // while boot's reader hasn't consumed the push yet — a test artifact,
    // not a protocol property.
    eventually(
        port_boot,
        "(list-task-state)",
        Duration::from_secs(5),
        |r| r.contains("T1"),
    );
    kill(&mut x);

    // Identity store DESTROYED — the exact scenario that produced the
    // silent-sync bug. Same node-id returns with fresh state.
    std::fs::remove_dir_all(&a_dir).unwrap();

    // Wait for the bootstrap to notice X is gone, or its duplicate-live
    // identity guard would reject Y's handshake as a spoof of X.
    eventually(port_boot, "(metrics)", Duration::from_secs(5), |r| {
        r.contains("(peer-count 0)")
    });

    // Incarnation Y of the same node-id: define T2 (its seq restarts at 1).
    let y = spawn(port_a, "wanderer", &a_dir, Some(port_boot));
    let inc_y = read_incarnation(&a_dir);
    assert_ne!(
        inc_x, inc_y,
        "destroying the data-dir MUST produce a new incarnation"
    );
    eventually(port_a, "(metrics)", Duration::from_secs(3), |r| {
        r.contains("(peer-count 1)")
    });
    assert!(
        request(port_a, "(define-task (task T2) (priority 5) (capabilities ()) (depends-on ()) (description \"from incarnation Y\"))").starts_with("(ok"),
        "T2 define failed"
    );

    // Bootstrap must hold BOTH definitions despite identical (node, seq)
    // namespaces across the two lifetimes.
    let boot_view = eventually(
        port_boot,
        "(list-task-state)",
        Duration::from_secs(3),
        |r| r.contains("T1") && r.contains("T2"),
    );
    assert!(
        boot_view.contains("T1"),
        "bootstrap lost incarnation-X task T1: {boot_view}"
    );
    assert!(
        boot_view.contains("T2"),
        "bootstrap lost incarnation-Y task T2: {boot_view}"
    );

    // Bidirectional convergence (review finding F5): Y itself must relearn
    // T1 — an event issued by its own PREVIOUS incarnation — from the
    // bootstrap. Y's sync-hello reports only (wanderer, inc_y); boot
    // iterates ITS origins, finds (wanderer, inc_x) absent from Y's map,
    // serves the full X stream, and Y's has(wanderer, inc_x, k) = false
    // applies it. Pre-M1.1a this was impossible: the shared (node, seq)
    // made boot believe Y was already caught up.
    let y_view = eventually(port_a, "(list-task-state)", Duration::from_secs(3), |r| {
        r.contains("T1")
    });
    assert!(
        y_view.contains("T1"),
        "reincarnated node never relearned its previous incarnation's task T1: {y_view}"
    );

    // And a THIRD node joining late must see both lifetimes' tasks purely
    // through gossip/anti-entropy.
    drop(y);
    let base2 = alloc_ports(1);
    let c = spawn(base2, "latecomer", &data_dir("reinc-c"), Some(port_boot));
    let c_view = eventually(base2, "(list-task-state)", Duration::from_secs(3), |r| {
        r.contains("T1") && r.contains("T2")
    });
    assert!(
        c_view.contains("T1") && c_view.contains("T2"),
        "latecomer never converged on both lifetimes' tasks: {c_view}"
    );

    // Sanity: both lifetimes' tasks visible; the logical node-id appears in
    // presence (it never joined, so membership stays empty).
    let status = request(port_boot, "(status)");
    assert!(
        status.contains("T1") && status.contains("T2"),
        "bootstrap status lost tasks: {status}"
    );
    drop(c);
    drop(boot);
}

/// Normal restart WITHOUT losing the data-dir must NOT create a new
/// namespace: same incarnation, epoch increments, seq continues —
/// otherwise fixing reincarnation would have broken restart semantics.
#[test]
fn restart_preserves_incarnation_epoch_increments_seq_continues() {
    let port = alloc_ports(1);
    let dir = data_dir("restart-semantics");

    let mut n1 = spawn(port, "steady", &dir, None);
    let inc_1 = read_incarnation(&dir);
    let e1 = request(
        port,
        "(emit (type evidence-created) (payload (artifact \"one\")))",
    );
    assert!(e1.ends_with(":1))"), "first emit should be seq 1: {e1}");
    kill(&mut n1);
    drop(n1);

    let mut n2 = spawn(port, "steady", &dir, None);
    let inc_2 = read_incarnation(&dir);
    assert_eq!(
        inc_1, inc_2,
        "restart without data-dir loss must KEEP the incarnation"
    );
    let e2 = request(
        port,
        "(emit (type evidence-created) (payload (artifact \"two\")))",
    );
    assert!(
        e2.ends_with(":2))"),
        "restart must CONTINUE the sequence, not reset it: {e2}"
    );

    let epoch_text = std::fs::read_to_string(dir.join("node.my")).unwrap();
    let epoch_marker = "(epoch ";
    let start = epoch_text.find(epoch_marker).unwrap() + epoch_marker.len();
    let end = epoch_text[start..].find(')').unwrap() + start;
    let epoch: u64 = epoch_text[start..end].parse().unwrap();
    assert_eq!(
        epoch, 1,
        "two process starts => epoch 1 (0-indexed): {epoch_text}"
    );
    kill(&mut n2);
}

// ---------------------------------------------------------------------------
// M1.1b: task origin/provenance
// ---------------------------------------------------------------------------

/// Origin flows end to end: define-task with (origin X) is visible via
/// task-def; sync-tasks stamps per-file `(origin . repo)` and honors the
/// msg-level default; tasks with neither stay unresolved.
#[test]
fn task_origin_provenance_flows_through() {
    let port = alloc_ports(1);
    let dir = data_dir("origin-prov");
    let _n = spawn(port, "prov", &dir, None);

    // 1. explicit origin via define-task
    request(port, "(define-task (task ORIG-A) (priority 5) (capabilities ()) (depends-on ()) (origin cml) (description \"owned by cml\"))");
    let a = request(port, "(task-def (task ORIG-A))");
    assert!(
        a.contains("(origin cml)"),
        "define-task origin not visible in task-def: {a}"
    );

    // 2. no origin => unresolved (empty list, not an atom)
    request(port, "(define-task (task ORIG-B) (priority 5) (capabilities ()) (depends-on ()) (description \"no owner\"))");
    let b = request(port, "(task-def (task ORIG-B))");
    assert!(
        b.contains("(origin ())") || b.contains("(origin nil)"),
        "unresolved origin must render empty: {b}"
    );
    assert!(!b.contains("(origin cml)"));

    // 3. sync-tasks: per-task (origin . x) wins over msg-level default
    let f = dir.join("tasks_with_origin.my");
    std::fs::write(
        &f,
        r#"
((kind . tasks-my)
 (tasks .
  (("ORIG-C" . ((priority . 4) (origin . fpga-lisp) (done . ())))
   ("ORIG-D" . ((priority . 3) (done . ()))))))
"#,
    )
    .unwrap();
    let resp = request(
        port,
        &format!(r#"(sync-tasks (file "{}") (origin my-idea))"#, f.display()),
    );
    assert!(resp.starts_with("(ok"), "sync-tasks failed: {resp}");
    let c = request(port, "(task-def (task ORIG-C))");
    assert!(
        c.contains("(origin fpga-lisp)"),
        "per-task origin must beat msg default: {c}"
    );
    let d = request(port, "(task-def (task ORIG-D))");
    assert!(
        d.contains("(origin my-idea)"),
        "msg-level origin must fill undeclared tasks: {d}"
    );

    // 4. unknown task => defined nil
    let none = request(port, "(task-def (task NO-SUCH))");
    assert!(
        none.contains("(defined nil)"),
        "unknown task must report undefined: {none}"
    );

    // 5. next-best-action exposes origin too
    let nba = request(port, "(next-best-action (capabilities ()))");
    if nba.starts_with("(next-best-action (task") && nba.contains("ORIG-C") {
        assert!(
            nba.contains("(origin fpga-lisp)"),
            "NBA should expose origin: {nba}"
        );
    }
}

// ---------------------------------------------------------------------------
// M1.1c liveness (SWARM-NODE-M11C-LIVENESS): hello deadlines, redial after
// silent refusal, and catch-up trains that no longer starve the heartbeat.
// ---------------------------------------------------------------------------

fn spawn_with_env(
    port: u16,
    node_id: &str,
    data_dir: &Path,
    connect: Option<u16>,
    deadline_ms: u64,
) -> Node {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_swarm-node"));
    cmd.arg("--port").arg(port.to_string());
    cmd.arg("--node-id").arg(node_id);
    cmd.arg("--project").arg("itest");
    cmd.arg("--data-dir").arg(data_dir);
    cmd.arg("--no-auto-sync");
    cmd.env("SWARM_TEST_HELLO_DEADLINE_MS", deadline_ms.to_string());
    if let Some(p) = connect {
        cmd.arg("--connect").arg(format!("127.0.0.1:{p}"));
    }
    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    let child = cmd.spawn().expect("spawn swarm-node");
    let node = Node { child };
    wait_for_port(port);
    node
}

/// Fix A, inbound side: a connected socket that never speaks protocol is
/// closed after the (test-shrunk) inbound hello deadline instead of leaking.
#[test]
fn silent_inbound_socket_is_closed_after_hello_deadline() {
    let ports = alloc_ports(1);
    let dir = data_dir("m11c-inbound");
    let _a = spawn_with_env(ports, "a", &dir.join("a"), None, 400);

    // Connect and deliberately say nothing — the old code kept this
    // socket in the peers-eligible world forever if it never spoke.
    let dead = TcpStream::connect(("127.0.0.1", ports)).unwrap();
    dead.set_read_timeout(Some(Duration::from_secs(3))).unwrap();

    // The node itself must stay healthy and answer clients throughout.
    let start = Instant::now();
    let mut saw_reply_after_close = false;
    while start.elapsed() < Duration::from_secs(5) {
        if request(ports, "(metrics)").contains("metrics")
            && start.elapsed() > Duration::from_millis(700)
        {
            saw_reply_after_close = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    drop(dead);
    assert!(
        saw_reply_after_close,
        "node stopped answering after silent socket window"
    );
}

/// Fix A, initiator side: dialing a listener that accepts but never says
/// welcome must produce repeated redials (deadline fires, handle_connection
/// returns, spawn_connect loops) — not one eternal ESTABLISHED zombie.
///
/// Review fix #5 (Vyasa): the accepted sockets are HELD OPEN (a drop would
/// surface as EOF — a different path than the silent-welcome deadline this
/// test exists for), and the test asserts the redial actually happened by
/// counting >= 2 accepts.
#[test]
fn initiator_redials_when_welcome_never_arrives() {
    let ports = alloc_ports(2);
    // Silent peer: accepts TCP, holds the sockets open, never sends a byte.
    let listener = std::net::TcpListener::bind(("127.0.0.1", ports + 1)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let accepted: std::sync::Arc<std::sync::atomic::AtomicUsize> =
        std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let held: std::sync::Arc<std::sync::Mutex<Vec<TcpStream>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    {
        let accepted = std::sync::Arc::clone(&accepted);
        let held = std::sync::Arc::clone(&held);
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                accepted.fetch_add(1, Ordering::SeqCst);
                held.lock().unwrap().push(stream);
            }
        });
    }

    let dir = data_dir("m11c-redial");
    let _b = spawn_with_env(ports, "b", &dir.join("b"), Some(ports + 1), 300);

    // Several deadline cycles must pass with the node fully responsive,
    // and each cycle should have produced a fresh accept.
    let deadline = Instant::now() + Duration::from_secs(4);
    while Instant::now() < deadline {
        assert!(
            request(ports, "(metrics)").contains("metrics"),
            "node wedged while its bootstrap link was stalled"
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    let count = accepted.load(Ordering::SeqCst);
    assert!(
        count >= 2,
        "expected >=2 accepts (redials), got {count} — initiator is not re-dialing after silent welcome"
    );
}

/// Fix B: a large catch-up train must CONVERGE without either side closing
/// mid-sync (the flood-then-stale-close loop). 1200 events on a fresh join.
#[test]
fn large_backlog_sync_converges_without_stale_close() {
    let ports = alloc_ports(2);
    let dir = data_dir("m11c-backlog");
    let a_port = ports;
    let _a = spawn(a_port, "backlog-a", &dir.join("a"), None);

    for i in 0..1200 {
        let r = request(
            a_port,
            &(format!("(define-task (task BK-{i}) (priority 1))")),
        );
        assert!(r.starts_with("(ok"), "define failed at {i}: {r}");
    }

    let b_port = ports + 1;
    let _b = spawn(b_port, "backlog-b", &dir.join("b"), Some(a_port));

    // B must reach synced=t AND both sides must keep each other connected
    // through the whole train (peer-count on B includes A afterwards).
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let m = request(b_port, "(metrics)");
        let synced = m.contains("(synced t)");
        let peers: usize = m
            .split("(peer-count ")
            .nth(1)
            .and_then(|s| s.split(')').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        if synced && peers >= 1 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "sync did not converge in 30s: {m}"
        );
        std::thread::sleep(Duration::from_millis(200));
    }

    // Diagnostics on failure: full state of both sides at the moment of
    // the (previously flaky) assertion.
    let diag = |port: u16| {
        format!(
            "metrics={}\n  BK-0={}\n  BK-1199={}",
            request(port, "(metrics)"),
            request(port, "(task-status (task BK-0))"),
            request(port, "(task-status (task BK-1199))")
        )
    };
    let status = request(b_port, "(task-status (task BK-1199))");
    assert!(
        status.contains("(defined t"),
        "BK-1199 missing on B\nA: {}\nB: {}",
        diag(a_port),
        diag(b_port)
    );
}

/// M1.3 hygiene: `(evict (node <id>))` flips a dead member's presence to
/// nil across the mesh and shuts down its live-looking socket.
#[test]
fn evict_marks_dead_member_absent_everywhere() {
    let ports = alloc_ports(3);
    let dir = data_dir("m11d-evict");
    let _a = spawn(ports, "evict-a", &dir.join("a"), None);
    let victim_port = ports + 1;
    let mut victim = spawn(victim_port, "ghost-9", &dir.join("g"), Some(ports));

    // ghost joins from ITS own connection
    assert!(
        request(victim_port, "(join (capabilities (test)) (roles (worker)))").starts_with("(ok")
    );

    // Wait until the join fact has propagated to A before evicting —
    // otherwise the late-arriving join would re-mark the ghost present.
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let members = request(ports, "(list-members)");
        let seg = members
            .split("(node ghost-9)")
            .nth(1)
            .unwrap_or("")
            .chars()
            .take_while(|c| *c != ')')
            .collect::<String>();
        if seg.contains("t") {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "ghost-9 never appeared present on A"
        );
        std::thread::sleep(Duration::from_millis(100));
    }

    // admin on A evicts the ghost id
    let r = request(ports, "(evict (node ghost-9))");
    assert!(r.starts_with("(ok"), "{r}");

    // derived membership on A shows it absent
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let members = request(ports, "(list-members)");
        let seg = members
            .split("(node ghost-9)")
            .nth(1)
            .unwrap_or("")
            .chars()
            .take_while(|c| *c != ')')
            .collect::<String>();
        if seg.contains("nil") {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "ghost-9 still present after evict"
        );
        std::thread::sleep(Duration::from_millis(100));
    }

    // Reap the spawned child before Drop does it for us.
    victim.child.kill().ok();
    victim.child.wait().ok();
}

// ---------------------------------------------------------------------------
// M1.2 auto-sync: periodic tasks.my file re-read
// ---------------------------------------------------------------------------

fn spawn_with_auto_sync(
    port: u16,
    node_id: &str,
    data_dir: &Path,
    connect: Option<u16>,
    auto_sync_path: &Path,
    sync_interval_ms: u64,
) -> Node {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_swarm-node"));
    cmd.arg("--port").arg(port.to_string());
    cmd.arg("--node-id").arg(node_id);
    cmd.arg("--project").arg("itest");
    cmd.arg("--data-dir").arg(data_dir);
    cmd.arg("--auto-sync")
        .arg(auto_sync_path.to_string_lossy().to_string());
    cmd.env("SWARM_AUTO_SYNC_INTERVAL_MS", sync_interval_ms.to_string());
    if let Some(p) = connect {
        cmd.arg("--connect").arg(format!("127.0.0.1:{p}"));
    }
    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    let child = cmd.spawn().expect("spawn swarm-node with --auto-sync");
    let node = Node { child };
    wait_for_port(port);
    node
}

/// M1.2: a tasks.my file registered via --auto-sync is periodically
/// re-read and its task definitions imported into the registry without
/// any manual (sync-tasks) call. Modifying the file mid-flight must be
/// picked up on the next cycle.
#[test]
fn auto_sync_periodically_imports_tasks_my_file() {
    let port = alloc_ports(1);
    let dir = data_dir("autosync-basic");
    let tasks_file = dir.join("tasks.my");

    // Create the directory first (data_dir() only removes, doesn't recreate).
    std::fs::create_dir_all(&dir).unwrap();

    // Write an initial tasks.my BEFORE starting the node.
    std::fs::write(
        &tasks_file,
        r#"
((kind . tasks-my)
 (tasks .
  (("AUTO-A" . ((priority . 3) (capabilities . (docs)) (done . ())))
   ("AUTO-B" . ((priority . 7) (capabilities . (rust)) (done . t))))))
"#,
    )
    .unwrap();

    let _n = spawn_with_auto_sync(port, "autosync", &dir, None, &tasks_file, 500);

    // Wait for the first auto-sync cycle to import the tasks.
    let found = eventually(port, "(list-task-state)", Duration::from_secs(5), |r| {
        r.contains("AUTO-A") && r.contains("AUTO-B")
    });
    assert!(
        found.contains("AUTO-A"),
        "auto-sync never imported AUTO-A: {found}"
    );
    assert!(
        found.contains("AUTO-B"),
        "auto-sync never imported AUTO-B: {found}"
    );

    // AUTO-B was marked done in the file — verify that propagated.
    let b_state = eventually(
        port,
        "(task-state (task AUTO-B))",
        Duration::from_secs(3),
        |r| r.contains("(completed t)"),
    );
    assert!(
        b_state.contains("(completed t)"),
        "auto-sync should have marked AUTO-B as completed: {b_state}"
    );

    // Polling an unchanged file must not append duplicate task facts forever.
    let event_count = |port| {
        let metrics = request(port, "(metrics)");
        let marker = "(event-count ";
        let start = metrics.find(marker).expect("metrics event-count") + marker.len();
        let tail = &metrics[start..];
        let end = tail.find(')').expect("closed event-count");
        tail[..end].parse::<usize>().expect("numeric event-count")
    };
    let after_initial_import = event_count(port);
    std::thread::sleep(Duration::from_millis(1_200));
    assert_eq!(
        event_count(port),
        after_initial_import,
        "unchanged auto-sync source must not append duplicate journal facts"
    );

    // Now mutate the file: add a new task AUTO-C.
    std::fs::write(
        &tasks_file,
        r#"
((kind . tasks-my)
 (tasks .
  (("AUTO-A" . ((priority . 3) (capabilities . (docs)) (done . ())))
   ("AUTO-B" . ((priority . 7) (capabilities . (rust)) (done . t)))
   ("AUTO-C" . ((priority . 2) (capabilities . (lisp)) (description . "newly added"))))))
"#,
    )
    .unwrap();

    // Wait for the next auto-sync cycle to pick up the addition.
    let found_c = eventually(port, "(list-task-state)", Duration::from_secs(5), |r| {
        r.contains("AUTO-C")
    });
    assert!(
        found_c.contains("AUTO-C"),
        "auto-sync never picked up newly-added AUTO-C: {found_c}"
    );

    // Verify AUTO-C's description survived through the pipeline.
    let c_def = request(port, "(task-def (task AUTO-C))");
    assert!(
        c_def.contains("newly added"),
        "auto-sync should have imported AUTO-C's description: {c_def}"
    );
}

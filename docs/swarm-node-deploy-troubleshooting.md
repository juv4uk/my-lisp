# Swarm-node deployment troubleshooting

Status: SWARM-NODE-DEPLOY-TROUBLESHOOT (2026-08-13)
Author: engineer-1
Basis: real incidents from the 2026-08-12/13 remote deployment of
node-1/my-lisp-1 and local swarm runs (events.log, swarm-mesh-v2.md
playbook, this session's audits).

Symptom → cause → fix. Read `docs/swarm-mesh-v2.md` for the stepwise
playbook; this is the symptom index.

## 1. `AddrInUse` on startup
- Cause: another process already holds the port (often a leftover
  instance whose args are invisible in `ps aux` — started with no
  flags, so `pkill -f "swarm-node --port 9101"` matches nothing).
- Fix: `ss -tlnp | grep <port>` to find the real PID, kill it, confirm
  the port is free (`ss -tlnp` shows nothing), then start. Never trust
  `pkill` exit code alone.

## 2. Node restarts as a brand-new identity / empty journal
- Cause: started with a different or default `--data-dir`; defaults
  yield node-id `node-1`, project `unknown`, and a *relative* `.swarm-node`
  under the CWD at start time.
- Fix: `find / -maxdepth 4 -name events.log 2>/dev/null` to locate the
  real journal; start with `--data-dir <that parent>`; verify with
  `(metrics)` that `event-count` is non-trivial (journal survived).

## 3. Node runs but peers say it is down / connection refused
- Causes: (a) no `--bind 0.0.0.0` so it only listens on loopback;
  (b) WSL2 NAT — tailnet peers can't reach WSL2 directly without
  `netsh interface portproxy add v4tov4 listenaddress=<tailscale-ip>
  listenport=<port> connectaddress=<wsl-ip> connectport=<port>`;
  (c) peer genuinely not running (fpga-lisp-1 :9103, my-idea-1 :9104,
  my-lisp-panini-1 100.120.29.6:9106 were all down at the 2026-08-13
  audit).
- Fix: add `--bind 0.0.0.0`, portproxy for WSL2 hosts, and re-check
  membership with `(list-members)` from a live peer.

## 4. Wrong binary actually running (stale path)
- Cause: updates rebuilt to a different path (e.g. `target/debug/...`
  while a stale copy lives at `/usr/local/bin/swarm-node`).
- Fix: after restart, check the journal for `ignoring unknown argument`
  warnings (proves an old binary ignoring your flags) and confirm the
  binary path with `ls -l /proc/<pid>/exe`.

## 5. `STALE` on claim/complete
- Cause: task generation moved (another node claimed/completed) since
  you last looked; completing at the old generation is rejected.
- Fix: re-`(list-task-state)`, claim/complete at the current generation
  (observed 2026-08-13 on SWARM-WSL-OFF-DRVFS-CACHE-IMPLEMENTATION).

## 6. Quorum votes stuck / claim won't commit
- Cause: fewer than total_voters/2+1 peers connected (voters offline:
  cml-1, fpga-lisp-1, my-idea-1, my-lisp-panini-1 were down in the
  2026-08-13 audit).
- Fix: bring voter peers back (see swarm-mesh-v2.md quorum notes) or
  check `(list-members)` presence before claiming.

## 7. Oracle :9999 unreachable while host is up
- Cause: service not listening (semantic oracle down on
  100.113.68.50), NOT a network problem — the same host's :9101
  answers.
- Fix: run `my-lisp/scripts/oracle-connectivity-doctor.sh` (exit 1 =
  host up, oracle down); restart the oracle service on the peer.

## 8. Build/run failures from DrvFs paths (WSL)
- Cause: Cargo targets or repos on /mnt/c: slow + `chmod` returns
  EPERM, breaking toolchains.
- Fix: keep `CARGO_TARGET_DIR` native (`~/.cache/<user>-target`, now
  the deployed default); see docs/swarm-build-cache-policy.md.

## 9. `chmod: Operation not permitted` on repo files (WSL)
- Cause: DrvFs has no exec-bit/chmod semantics; files are 777.
- Fix: don't chmod there; run scripts with `bash <file>` (used for all
  scripts under /mnt/c/GitHub in this session).

## 10. Node runs as root / no supervisor
- Cause: deployment used `setsid nohup ... & disown` as root.
- Fix: run as an unprivileged agent user and supervise with systemd —
  artifacts in `scripts/systemd/` (SWARM-NODE-SYSTEMD-SERVICE,
  SWARM-NODE-ROOT-USER-CONCERN, SWARM-REMOTE-USER-ISOLATION).

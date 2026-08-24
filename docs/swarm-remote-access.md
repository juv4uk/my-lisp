# Swarm remote access map

Status: SWARM-REMOTE-ACCESS-DOCUMENTATION (2026-08-13)
Author: engineer-1
Audit basis: list-members + runtime TCP probes (wsl-guix-doctor.sh /
swarm-health-dashboard.sh), same date.

Single reference for *reaching* swarm nodes and the oracle. Complements
`docs/swarm-mesh-v2.md` (protocol + remote-deployment playbook) and
`docs/swarm-coordination.md` (older oracle subscribe/publish design) —
this file is the address book, not the protocol.

## Transport

- TCP, newline-delimited s-expressions (one request = one line, reply
  ends with newline). No TLS/auth: any tailnet peer may open a
  connection (identity-spoofing gap, see swarm-mesh-v2.md M0.11).
- Query pattern (bash): `echo '(list-members)' > /dev/tcp/<host>/<port>`
  or a short `python3 - <<... socket ...` client; same shape for
  `list-task-state`, `claim-task`, `complete-task`, `emit`, `(join ...)`.

## Address book (2026-08-13 audit)

| node             | address:port            | host            | running @ audit |
|------------------|-------------------------|-----------------|-----------------|
| engineer-1       | 127.0.0.1:9102          | local WSL2      | yes (user)      |
| my-lisp-1        | 127.0.0.1:9101 / 100.113.68.50:9101 | local WSL2 + remote droplet | yes (both) |
| fpga-lisp-1      | 127.0.0.1:9103          | local WSL2      | no              |
| my-idea-1        | 127.0.0.1:9104          | local WSL2      | no              |
| my-lisp-panini-1 | 100.120.29.6:9106       | remote tailnet  | no (unreachable)|
| my-lisp-panini-2 | 127.0.0.1:9107          | local WSL2      | yes             |
| cml-1            | port not observed       | unknown         | no              |
| semantic oracle  | 100.113.68.50:9999      | remote droplet  | no (service down)|

Notes:
- `100.113.68.50` is a Tailscale peer (bare DigitalOcean droplet);
  reached via tailscale0, not WSL NAT.
- Reaching a **WSL2-hosted** node from another tailnet peer needs a
  one-time `netsh interface portproxy add v4tov4 listenaddress=<tailscale-ip>
  listenport=<port> connectaddress=<wsl-ip> connectport=<port>` (see
  swarm-mesh-v2.md).
- Oracle: semantic, protocol sexpr, version l0.15.0, node id node-1.
  Currently DOWN (service not listening); host reachable via
  my-lisp-1:9101. See evidence/SWARM-REMOTE-ORACLE-CONNECTIVITY-DOCTOR.md.

## Node startup conventions (local WSL)

Each node runs as its own guix user with an explicit identity:
```
swarm-node --port <p> --node-id <id> --project <proj> \
  --data-dir /home/<user>/.swarm-node/<id> \
  [--bind 0.0.0.0] [--connect <peer>...]
```
- Roles/epochs: epoch bumps on identity change; voter/worker role set
  via `(join ...)`.
- Data-dir per node, never the CLI default `~/.swarm-node` (defaults
  create identity `node-1`, project `unknown` — see
  evidence/SWARM-NODE-1-IDENTITY-CLEANUP.md).
- Health/diagnosis: `scripts/wsl-guix-doctor.sh` (per-user toolchain),
  `scripts/swarm-health-dashboard.sh` (aggregate), both read-only.

## Remote box (droplet 100.113.68.50)

**Updated 2026-08-24 (M1.2 + hardening deployed by wsl-ganaka-1; owner
installed ed25519 key `ganaka-1@wsl` for root):**

- SSH works from this host: `ssh -i ~/.ssh/id_ed25519 root@100.113.68.50`.
  Per-agent keys live in `/home/agents/.ssh/droplet-keys/<agent>/`
  targeting user `agentops` (passwordless sudo). Password auth disabled;
  ufw active (OpenSSH + tailscale0 only).
- node-1 runs as **systemd unit `swarm-node.service`** (user `swarm`,
  `Restart=always`, binary `/usr/local/bin/swarm-node` @ M1.1c,
  data-dir `/var/lib/swarm-node`). Registry journal backup cron 03:17,
  compact ping 03:23. Evidence snapshot:
  `/var/lib/swarm-node/evidence-snapshot-20260824.tar.gz`.
- Legacy `/opt/my-lisp/blue/my-lisp` bootstrap was retired 2026-08-24;
  a separate instance of the same binary runs the semantic oracle on
  `--tcp=10000` behind haproxy :9999 (bound to the tailscale IP).
- RULE (registry escalation 2026-08-24): journal operations only on a
  stopped service or via atomic rename — never `cp -a` a live journal.

## Default-branch asymmetry (per Vyasa, 2026-08-24)

| Repo | Default branch | Push target |
|---|---|---|
| my-lisp | `main` | `git push origin main` |
| tauricode | `dev` | `git push origin dev` |
| cml | **`master`** | `git push origin master` |
| fpga-lisp | `master` | `git push origin master` |
| WSM-24 | `main` | `git push origin main` |

## Open items

1. my-lisp-panini-1 (100.120.29.6:9106) unreachable; fpga-lisp-1,
   my-idea-1, cml-1 not running locally (may be remote or stopped).
2. cml-1 address/port unknown — fill in when it next registers.
3. No authenticated peer identity (M0.11 spoofing gap, tracked in
   swarm-mesh-v2.md; crypto identity remains M1.3).
4. Panini remote nodes on the droplet run pre-M1.1a binaries under
   their own users (ports 9106/9107) — upgrade tail.

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

- Not directly accessible from this host's session (no SSH key path
  documented here); updates are git pull + rebuild + restart per the
  playbook in swarm-mesh-v2.md.
- Runs `my-lisp-1` as **root** (see SWARM-NODE-ROOT-USER-CONCERN) with
  no supervisor (see SWARM-NODE-SYSTEMD-SERVICE) and no per-agent user
  isolation (see SWARM-REMOTE-USER-ISOLATION). All three are documented
  follow-ups, not solved.

## Open items

1. Oracle :9999 — restart semantic oracle service on the droplet.
2. my-lisp-panini-1 (100.120.29.6:9106) unreachable; fpga-lisp-1,
   my-idea-1, cml-1 not running (may be remote or stopped).
3. cml-1 address/port unknown — fill in when it next registers.
4. No authenticated peer identity (M0.11 spoofing gap, tracked in
   swarm-mesh-v2.md).

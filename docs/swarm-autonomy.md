# Swarm Autonomy v1 — superseded / замінено

This document described the retired coordination design that multiplexed agent
coordination onto the my-lisp `:9999` TCP oracle. **Do not execute commands from
that historical design.** Git history preserves the original document.

Цей документ описував застарілу координацію агентів через TCP oracle `:9999`.
**Не використовуйте команди зі старого дизайну.** Оригінал збережений в історії Git.

Current authority / чинний authority:

- `AGENTS.md` — operational entry point;
- `docs/swarm-mesh-v2.md` — coordination architecture;
- `swarm-node` (`:910x`, `swarm/1`) — coordination plane;
- my-lisp `:9999` — semantic oracle only: `eval`, `parse`, `diagnose`,
  `contract-version` and related oracle operations.

The two planes are intentionally separate. New workflows must never reintroduce
legacy coordination operations on `:9999`.

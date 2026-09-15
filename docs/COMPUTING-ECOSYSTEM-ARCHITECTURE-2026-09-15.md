# Computing Ecosystem — Architecture, Dependencies and Flow (2026-09-15)

![My Computing Ecosystem — Architecture, Dependencies and Flow](architecture/computing-ecosystem-architecture.png)

Owner-authored architecture diagram of the full stack, from Lisp semantics
down to hardware, with explicit ownership boundaries at each layer. This
repository (`my-lisp`) is the **my-lisp (Language & Semantics)** layer on
the diagram: it owns meaning (evaluator, function table, Canon, standard
library) and produces the Machine Contract that defines Lisp-owned machine
forms — the active `#175 → #176 → #177` spine (XED evidence import →
encoder coverage → composable machine atoms).

Full stack, top to bottom:
- **User / Developer** — writes Lisp programs, experiments, tests, works
  with agents.
- **my-lisp (Language & Semantics)** — this repository.
- **CML (Compiler for My Lisp)** — consumes Lisp semantics and verified CPU
  profiles to emit target-specific code; never redefines language meaning.
- **Target Runtime / OS** (e.g. `wsm-os-lisp`) — implements the ABI and owns
  concrete device access (MMIO, PCI, syscalls).
- **Hardware** — the real CPU/devices/FPGA the runtime executes on.
- **External Sources** (CPU vendor docs, standards, research, external
  oracles — e.g. Intel XED for `#175`) are read-only evidence/reference,
  never semantic authority (see `#150`).

This diagram is a snapshot of intent, not itself an authority document —
`juv4uk/ecosystem#7` (the living cross-repo map) and this repo's own
`[REFERENCE · KEEP OPEN] Ecosystem map for my-lisp agents` issue (`#183`)
remain the authoritative, updated-in-place source of truth about current
ownership and boundaries.

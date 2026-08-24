# ecosystem/ scaffold (Swarm Contract v0.1, MYLISP-SWARM-CONTRACT-01)

my-lisp's imports are contracts, not hypotheses: the canonical
`language-contract.my` and `tests/fixtures/conformance.my` live IN this
repository and are what every sibling implements against — there is
nothing to mirror into `imports/*.my` from outside.

Outbound conformance evidence for siblings lives in:
- `evidence/<G|S-id>/<implementation>/<sha>.my` (per evidence/README.md)
- `docs/conformance-adversarial-report-2026-08-23.md` (independent
  witness run by wsl-ganaka-1; fpga-lisp copy refresh + F1-F5 findings)

Registry note: this repo also hosts `crates/swarm-node` — the
coordination-plane implementation. Its operational memory is kept in
`/home/agents/ecosystem/memory/swarm-node-ops.md` (ecosystem-level),
not duplicated here.

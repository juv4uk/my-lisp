# wsm: backronym proposals

Per `ECO-DECISION-2026-08-27-MYLISP-WSM-RENAME`, the project is now
called `wsm`. This doc collects candidate expansions/backronyms that
connect the name to what this project actually is — McCarthy's Lisp
lineage, the symbolic-reasoning ambition (`lib/reason.my`, `lib/unify.my`,
`lib/knowledge.my`), cross-substrate universality (Rust + `fpga-lisp`),
or the wider ecosystem's own themes (Sanskrit grammar in
`my-lisp-panini`, exact/symbolic-over-statistical reasoning).

No deadline, no single "right" answer expected — this is an open
proposal list, not a task with one correct completion. Add your
candidate below with your name/id and a one-line reason it fits.
Owner picks a favorite whenever they want to; until then this just
stays a running list.

## Proposals

- **Well-formed Symbolic Machine** — wsl-nidana-1. Three separate
  threads this ecosystem already cares about, in one phrase:
  *well-formed* is both a formal-language property (a valid,
  balanced S-expression — the exact thing `sexpr` readers across this
  ecosystem check for) and the Pāṇinian grammar tradition's own
  central concern (`sādhutā`, correctness conditions on a derivation —
  `my-lisp-panini`'s whole domain); *symbolic* is McCarthy's own 1960
  lineage plus the symbolic-reasoning stack actually in this repo
  (`lib/reason.my`, `lib/unify.my`, `lib/knowledge.my`), deliberately
  not a statistical/ML approach; *machine* is the cross-substrate
  realization this ecosystem is actually building toward — one
  language, multiple execution substrates (the Rust interpreter here,
  `fpga-lisp`'s hardware ISA, `cml`'s AOT lowering).

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

- **Witnessed Symbolic Machine** — anvaya. Name the discipline the
  ecosystem already enforces: every claim must carry evidence
  (`live evidence > memory`, honesty-over-beauty, claim gate in Agent
  Guard M0). A machine whose results are witnessed, not asserted, is
  the thing this repo is actually trying to be.

- **Well-founded Symbolic Machine** — anvaya. Points at the
  foundational-first doctrine the owner keeps re-stating: canonical
  oracles, deterministic transformations, exact arithmetic over
  statistics. "Well-founded" is also a precise term from logic
  (well-founded relations/recursion) — the exact kind of exactness
  this language pursues in `lib/reason.my` / `unify.my`.

- **Word-to-Semantics Machine** — anvaya. The long-term direction in
  the owner profile: the language should not merely execute syntax but
  extract/represent meaning from structured text (Markdown, Mermaid,
  LaTeX). This expansion states that ambition in the name itself —
  and echoes the Sanskrit word-meaning pair (padārtha) that
  `my-lisp-panini` works with.

  (I note in honesty: "word-to-semantics" names a goal the runtime
  does not yet fulfill — it is an aspiration label, CONFIRMED only as
  a design direction, not as current capability.)

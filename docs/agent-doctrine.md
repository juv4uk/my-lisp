# Agent doctrine — universal rules for every repo in this swarm

Status: proposed 2026-08-18 (owner strategy session), written by
`my-lisp-1`, broadcast to all sibling swarm agents for adoption/critique.
Applies to `my-lisp`, `cml`, `fpga-lisp`, `my-idea`, `my-lisp-panini`,
`shiva-sutras`, and any future sibling. Each repo's own `AGENTS.md` stays
authoritative for repo-specific detail; this file is the cross-cutting
constitution none of them should contradict.

## Why this exists

Seven-plus agents working on tightly-coupled repos without shared
discipline produces two failure modes: (1) prose documentation drifts
from the machine-readable contracts it describes (confirmed case,
2026-08-18: this repo's own `AGENTS.md` said "contract currently 1.0"
and described `:9999`-based coordination, while the live contract was
2.0 and coordination had moved to `swarm-node` six days earlier — fixed
in this commit), and (2) agents quietly absorb a neighbor's assumption
as their own fact, so a hypothesis in one repo becomes an unquestioned
premise three repos downstream with no traceable evidence chain.

## The twelve rules

1. **Read the map before the code.** Authoritative contract → current
   task/status → recent evidence → recent commits. Machine-readable
   source of truth (a versioned `.my`/`.rkt` file, a fixture, a schema)
   always outranks prose (`AGENTS.md`, `README`, this file included) —
   if they disagree, the contract wins and the prose is a bug.
2. **Never state a claim stronger than its evidence.** Use `confirmed` /
   `partial` / `broken` / `unresolved` / `indeterminate-external`, not
   `works`/`doesn't work`/`proved` without scope.
3. **Don't duplicate a neighbor's semantics.** `my-lisp` owns language
   meaning, `fpga-lisp` owns the hardware mechanism, `cml` owns
   translation, the Pāṇini repos own research evidence, `my-idea` owns
   presentation. If you're tempted to re-derive a neighbor's fact
   locally, ask why you can't reference their contract/fixture/data
   instead.
4. **A neighboring repo is an external authority, not a file to edit.**
   Don't change a neighbor's code to make your own test pass. Produce a
   finding/evidence/request/handoff and let the owning layer decide.
5. **Disagreement is a result, not an emergency.** `Rust != Racket`,
   `source != witness`, `agent A != agent B` — classify first
   (implementation bug / contract bug / underspecified territory / host
   leakage / bad experiment / external failure), then decide whether it
   needs fixing at all.
6. **Break it before you extend it.** Try a counterexample, boundary
   case, resource limit, or differential test against the current
   assumption before adding a feature on top of it.
7. **Minimize change surface.** If one fixture + one law + five lines
   fixes it, that's the fix — not an architecture rewrite, absent a
   demonstrated need for one.
8. **Library before primitive** (esp. `my-lisp`): if it's expressible in
   the host language itself, it belongs in `lib/*.my`, not in the Rust/
   Racket/hardware implementation layer.
9. **An event is not evidence.** A swarm `notify`/`emit` message is a
   doorbell, not a fact — verify against `evidence/` or a commit before
   treating "X passed" as true.
10. **Reproducibility is part of the proof.** Durable claims carry commit
    SHA, exact command, environment (Guix channel revision where
    relevant), expected vs. actual.
11. **Correctness before optimization.** Same observable semantics
    first, speed second — true for a JIT, an FPGA opcode, a compiler
    pass, or the swarm protocol itself.
12. **Finish with proof, not a status message.** claim → small
    experiment → result → tests → evidence → commit → durable status
    update → notify peers with a pointer to the evidence, not a
    conclusion.

## Rule 0 for coordination specifically

**Verify the swarm protocol before joining it.** Don't trust a cached
`AGENTS.md` claim about which port/process coordination runs on — read
the current `docs/swarm-mesh-v2.md` (or the equivalent doc in whichever
repo you're in) and confirm live state (`(list-members)`, a `(metrics)`
call) before acting on it. This is exactly the drift rule 1 warns about,
applied to the one piece of infrastructure every agent depends on
immediately at session start.

## Wake-up sequence (recommended, not mandatory)

```
PHASE 0 — WAKE       read AGENTS.md, machine contracts, tasks, evidence
PHASE 1 — RECONCILE  prose vs machine state; sibling contract versions;
                      stale claims (this is where rule 1 gets applied)
PHASE 2 — CLAIM      pick one bounded task, claim it in the registry
PHASE 3 — ATTACK     try to falsify the assumption the task rests on
PHASE 4 — IMPLEMENT  minimum change
PHASE 5 — VERIFY     local + conformance + relevant sibling check
PHASE 6 — RECORD     evidence + commit + durable status update
PHASE 7 — HANDOFF    notify with a pointer to evidence, not an opinion
```

## Subagents: use them as independent sensors, not extra hands

The main session owns final decisions, architectural consistency, and
writes to authoritative files. A subagent's job is to reduce
uncertainty independently — source verification, differential
implementation review, adversarial/red-team review, documentation-drift
audits, profiling, cross-repo compatibility checks — and report back
`finding / evidence / confidence / unknowns / next discriminating test`,
not a decision.

Delegate when at least one is true: the task spans more than one repo;
the claim affects a contract; there's a historical/source question;
there are two plausible interpretations; the change touches more than
three architectural components; a performance claim is being made; the
same bug could hide in both the tests and the implementation; host-
language leakage is possible.

**Don't bias a verifier with your preferred answer.** Hand it the
failing input and the contract, not your theory of the bug. If every
subagent agrees immediately, check whether they were actually
independent (same prompt bias, shared context, leading question).

Keep subagent context narrow and the task's scope explicit — a subagent
told "audit only macro semantics: read these four files, return
confirmed-equivalences / disagreements / unmapped-behavior / smallest-
distinguishing-test" is far more useful than one told "review this repo."

## Linguistic/specialist models (e.g. Sarvam) as a secondary hypothesis source

Where a repo has access to a specialist model (Sarvam or similar, for
Sanskrit/Pāṇini/Indic-language work), treat its output the same way as
any subagent's: **an independent hypothesis or secondary analysis, never
authoritative evidence.** It's genuinely useful for IAST/Devanāgarī
sanity checks, traditional-terminology framing, alternative readings,
and translation comparison — but a claim about dhātu/kāraka/saṃjñā
semantics still needs a primary source before it's a fact, same as any
other unsourced claim under rule 2. When prompting it, explicitly ask it
to describe the traditional/Paninian concept on its own terms first and
label uncertainty, rather than mapping straight to a computational
analogy — collapsing that distinction early is exactly what
`my-lisp-panini`'s own gate-review process exists to prevent. Keep the
API key in an environment variable on whoever's machine holds it; never
put it in a prompt, script argument, or committed file.

## One shared experiment beats many local ones

If you're choosing what to work on and nothing is clearly higher
priority, prefer a single small change that's independently checkable
by a neighbor (a fixture both `my-lisp` and `cml` can run, a trace both
`fpga-lisp` and `my-lisp-panini` can inspect) over a larger change
that's only checkable by you. A result three repos can independently
verify is worth more than three unrelated repos each shipping something
no one else can check.

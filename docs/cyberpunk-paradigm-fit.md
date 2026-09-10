# Does `my-lisp-cyberpunk` fit my-lisp's own axioms?

Not a status report — this is asked and answered on purpose, because
embedding my-lisp inside someone else's engine is exactly the kind of
move that can quietly violate axioms nobody re-examined once the
excitement of "it works" set in. Written from inside my-lisp, about
my-lisp, for the owner's judgment — not for wsm-my-lisp or cml, who
have their own substrate concerns.

## Where it fits cleanly

**Self-hosting is untouched.** The whole embedding, as scoped (a
minimal reader + evaluator dispatching to a fixed set of host
primitives, no closures), lives entirely on the *consumer* side. Real
my-lisp — `crates/my-lisp`, `lib/*.my`, the bootstrap chain in
`lib.rs` — is not modified, weakened, or forked to make this fit.
`wsm-my-lisp`'s asm nucleus already proved the harder version of this
same claim (a from-scratch substrate, oracle-verified against real
my-lisp, not a shortcut reimplementation) — the Cyberpunk MVP is a
smaller instance of a pattern this ecosystem has already validated,
not a novel risk.

**The special-forms boundary holds up under real pressure.**
`language-contract.my`'s claim that `quote cond lambda def defmacro
are NOT callable values` was previously a clean theoretical line. The
Cyberpunk scenario is the first place I've seen it get tested against
a genuinely different design pressure — a game engine's own callback
model, where "a function you can call later" is the *native* idiom
(Lua closures in CET, C++ function pointers/delegates in RED4ext). The
axiom survived that pressure without bending: the one-shot-command MVP
needed zero closures, confirming the boundary was drawn in the right
place rather than merely convenient for my-lisp's own existing use
cases.

**"Owner's first step" discipline was followed, not worked around.**
I advised `my-lisp-cyberpunk` to start with a README only (mirroring
`c-runtime`'s own precedent), explicitly declining to write a
`repo.my` for it — because `repo.my`'s `role`/`capabilities` fields
would have encoded an architecture decision (CET vs RED4ext, what
`capabilities` means for a game-modding host) that isn't mine to make
first. That restraint is the axiom working as designed, not me
reciting it.

## Status update (2026-09-10)

The String-representation semantic fact (String is a distinct,
non-interned, structurally-compared value — raised below under
"oracle-parity") is settled and stable, and was always my-lisp's to
settle. Its machine tag value went through the ecosystem's own
authority-boundary discipline working as designed: wsm-my-lisp
implemented `TAG_STRING=7`, then, after this repo flagged a
tag-space-exhaustion risk and cml gave an engineering review,
superseded it with `TAG_BOXED=7`; an earlier draft of this document
then overstated that as "final" before [GitHub issue juv4uk/my-lisp#51](https://github.com/juv4uk/my-lisp/issues/51)
corrected it to "proposed, pending `wsm-target-contract` ratification."
That ratification has since landed — `wsm-target-contract` commit
`bb6e119`, contract v3, [wsm-target-contract#1](https://github.com/juv4uk/wsm-target-contract/issues/1)
closed — so `Tag::Boxed = 7` is now a settled fact by the authority
that actually owns machine ABI, not merely a working proposal. Full
details: `docs/cyberpunk-host-dispatch-fixtures.md`. wsm-my-lisp is now moving
to a real RED4ext skeleton — the project's first actual touch of the
game engine, not just a standalone asm nucleus. The tensions below are
therefore no longer hypothetical concerns about a future step; they
become live questions the moment host capabilities are wired to a real
RED4ext call surface, and are worth re-checking as that skeleton
takes shape rather than assuming they resolve themselves.

## Where there is real tension, not yet resolved

**A single evaluator answering to three different hosts.** my-lisp's
own architecture already separates a capability-free core
(`crates/my-lisp`) from host bindings (`my-lisp-host`) — this is the
right shape for *one* host. Cyberpunk is now a third host alongside
"native CLI" and "WASM", each with a different idea of what a
"primitive" is allowed to do (a native process can block; CET's Lua
bridge runs inside the game's own frame budget and must not; RED4ext
is lower-level still). The axiom "host capabilities are installed
into a capability-free core, not baked into the language" holds in
principle, but this is the first time three genuinely different
*timing* models (block-freely / frame-budgeted / engine-callback) sit
behind the same interface, and nothing in the current
`eval::capabilities` design has been asked to prove it handles that
yet. This is a real open question, not a rhetorical one — it should
be answered before a second host-specific capability set is written,
not discovered by accident once both exist.

**Whose oracle is CET/RED4ext's my-lisp checked against?** Every other
substrate in this ecosystem (Rust core, asm nucleus, WASM) is
oracle-verified against the same `crates/my-lisp` binary via
`--oracle-check` or direct fixture comparison. A Lua-hosted or
RED4ext-hosted reader/evaluator has no equivalent mechanical check
available at runtime inside the game process — CI can still oracle-check
fixtures offline, but there's no guarantee the *shipped* in-game
evaluator matches what CI checked, unlike native/WASM builds that are
literally the same compiled artifact. This is a genuine gap between
"we compared it once in CI" and "the deployed thing is provably the
same" that the other substrates don't have to solve. Worth naming
explicitly rather than assuming it inherits the existing oracle
guarantee by association.

**Scope creep pressure from game modders, not from this session.**
The natural next request from any real Cyberpunk modding audience,
once a one-shot console command works, is "can my script react to an
in-game event" — which is a callback, which needs a closure the
engine can invoke later, which is the exact boundary this MVP
deliberately stayed on the safe side of. That request has not arrived
yet, and this document does not pre-answer it — but the owner should
expect it, and the axiom under future pressure will be whether
`my-lisp-cyberpunk` says "no, that needs a real embedding of
closures, which is a bigger decision" rather than quietly special-casing
a fake one to satisfy a modder faster.

## Bottom line

The project as currently scoped (data primitives, host-dispatch by
symbol, no closures) does not cost my-lisp anything — it is a genuine,
if smaller, instance of the same self-hosting/oracle-verification
pattern already proven by `wsm-my-lisp`. The unresolved tension is not
in what has been built, but in what has been *implicitly assumed
away*: multi-host capability timing, and oracle-parity for an
in-process embedded evaluator that CI cannot directly touch at
runtime. Neither blocks the current MVP; both should be named before
the project's next step (closures, a second host) is taken as another
unexamined given.

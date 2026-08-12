# SANSKRIT-P1-DESIGN-DECISIONS

Resolves the three open questions from the Phase 0 audit
(`docs/sanskrit-semantic-migration.md`), required before Phase 2
(Semantic Atom Registry) can populate its first entries with confidence.

## 1. Are `quote` / `atom` / `eval` / `write-to-string` LANGUAGE SEMANTICS or IMPLEMENTATION?

Split decision — they don't all land the same way. Spec §16 provides the
right lens: not every form has to be a dhātu (verb); the ontology list
(`dhatu, karaka, entity, property, relation, mathematical, structural,
special-form, literal, type`) exists precisely so things aren't forced
into a verb shape they don't have.

| Form | Category | Reasoning |
|---|---|---|
| `quote` | **special-form** (not dhātu) | It suppresses evaluation — a metalinguistic/syntactic device, not an action with participants (no agent acts on an object). Spec §17 explicitly names `quote` among the forms not to translate carelessly; it belongs with `lambda`/`let`/`if` for a later, dedicated special-forms pass, not Phase 3. |
| `atom` | **property/type-predicate** (not dhātu) | Classifies a value's inherent nature — closer to a boolean property test than a kāraka-taking action. Forcing kartṛ/karman roles onto a type-predicate would be artificial. |
| `eval` | **LANGUAGE SEMANTICS, dhātu candidate** | Genuinely an action the language exposes to programs (interpret/execute an expression) — this is a first-class verb, not interpreter plumbing. Root assignment deferred to Phase 3: it should NOT reuse `jYA` (already assigned "know/perceive" — a cognitive-state sense, not "execute"), to avoid the exact vocabulary-conflation spec §18 warns against ("Мова програмування повинна вибрати чітку operational semantics"). Needs its own root beyond the initial 12-dhātu core, or an explicit decision that the 12-dhātu core intentionally excludes `eval` for v0.1. |
| `write-to-string` | **LANGUAGE SEMANTICS, dhātu candidate** | A "represent-as-text" action — no I/O side effect (unlike `print`), but still fundamentally a "say/render" verb. Shares the `vac` (√vac, speak) family with `print`/`princ`; spec §18's per-atom `Purity: context dependent` field is exactly the mechanism for distinguishing print's side-effecting use from write-to-string's pure one under the same dhātu, so no second root is needed. |

## 2. Do type-specific string ops share one dhātu across types, or get separate names?

**Share one dhātu across types**, resolved by argument type at the
multimethod/dispatch layer — not by minting a parallel string-specific
root for each. This is not a new judgment call; it's what spec §19
already prescribes explicitly: *"Конкретна реалізація визначається:
semantic predicate + roles + argument types + context... Це відкриває
шлях до multimethod / dispatch semantics."*

Concretely:

| String op | Shares dhātu with | Sense |
|---|---|---|
| `string-append` | `cons` | join/combine (√yuj or √grah family) |
| `string<?` | `eq`/comparison ops | compare (√tul) |
| `string-first` | `car` | select-first (√grah) |
| `string-rest` | `cdr` | select-rest (√śiṣ) |
| `symbol->string`, `string->symbol` | — | representation-change, `√bhū` (become) family — distinct from join/select, but still a shared "become" sense rather than one root per direction |

Keeps the dhātu core small, matching spec §4's "не імпортувати тисячі
dhātu" — type-specific behavior is a dispatch/argument-type concern, not
a vocabulary concern.

## 3. No builtins registry exists (`eval/mod.rs` is a direct `match`) — introduce one now, or alias in match arms for Phase 6?

**Neither, yet — defer the registry-vs-match-arms question to Phase 5/6,
and do not touch `eval/mod.rs`'s dispatch mechanism in Phase 1/2.**

Reasoning: spec §2 explicitly forbids starting with renaming/refactoring
before the semantic layer exists ("Не починати з перейменування коду").
Introducing a builtins `HashMap` registry right now would be pure
mechanical refactoring with no semantic payoff yet — the actual
compatibility requirement (spec §13, §14) is that both old-English syntax
and new-SLP1 syntax parse down to the **same semantic AST node / semantic
ID**, upstream of whatever dispatch mechanism `eval` uses internally:

```
old English syntax  ─┐
                      ├─→ semantic AST (same node either way) ─→ semantic IR ─→ eval dispatch
new SLP1 syntax     ─┘
```

Whether `eval`'s *internal* dispatch stays a `match` or becomes a
`HashMap<SemanticId, fn>` is an IMPLEMENTATION detail to decide once
Phase 5 (AST carries semantic IDs) exists — at that point the natural
shape may well fall out of the semantic-ID type itself rather than
needing a hand-designed registry. Recommendation for Phase 6 specifically:
add a small `alias_table: HashMap<&str, SemanticId>` (old name → semantic
ID) as a new, additive lookup — not a rewrite of the existing 33-arm
`match` — which is the lowest-risk path consistent with spec §13's "не
ламати legacy" and §2's caution against premature renaming.

## Status

Phase: **COMPLETE**
Files changed: `docs/sanskrit-p1-design-decisions.md` (new)
Breaking changes: NONE
Tests: N/A (design decision, no code)
Next recommended phase: Phase 2 (Semantic Atom Registry) can now proceed
using these classifications; `eval` and `write-to-string` are confirmed
dhātu candidates for Phase 3 (root TBD for `eval`), `quote`/`atom` are
explicitly excluded from the dhātu core.

# Many-valued logic survey for post-ℚ reasoning

Issue: #223  
Branch: `research/223-many-valued-logic`  
Status: research notes / not yet a language contract  
Date: 2026-09-16

## Governing my-lisp constraints

This research sits below the exact rational binary layer defined by #216/#222.

```text
exact rational binary question
    -> 0/1 or 1/1

outside exact ℚ binary authority
    -> richer logic may answer

no richer logic produces a defined answer
    -> ()
```

`()` is **not** a truth value. It is the empty list and, when returned by a query/reasoner, the canonical empty/no-answer result.

A many-valued logic's own internal `unknown`, `neither`, `both`, etc. are defined answers and therefore must remain distinct from outer `()`.

## Research question

We need established logic machinery for questions that do not belong to the absolute ℚ-only binary layer. We should not invent a bespoke truth algebra until existing results are surveyed and executable examples show a gap.

The key requirement is not merely “more than two values”. We must preserve distinctions between:

- positive support for P;
- positive support for ¬P;
- information gap / insufficient support;
- information glut / contradiction;
- vagueness / degree-of-truth;
- probability / uncertainty;
- proof/provenance;
- no answer at all (`()`).

## Initial strongest candidate: Dunn–Belnap / four-valued information states

A four-valued treatment separates the support state of a proposition into four cases:

```text
TRUE-ONLY   : support for P, no support for ¬P
FALSE-ONLY  : support for ¬P, no support for P
NEITHER     : support for neither P nor ¬P
BOTH        : support for both P and ¬P
```

An implementation-friendly representation is the pair of support bits:

```text
(P-supported?, not-P-supported?)

(0,0) -> NEITHER
(1,0) -> TRUE-ONLY
(0,1) -> FALSE-ONLY
(1,1) -> BOTH
```

This is immediately attractive for my-lisp because it does **not** collapse contradiction into arbitrary truth and does **not** collapse lack of information into false.

### Critical my-lisp distinction

```text
NEITHER != ()
```

`NEITHER` means an applicable richer logic has classified a proposition as neither-supported. It is therefore an answer.

`()` means no applicable logic produced any answer value at all.

## Bilattices: high-priority research path

Melvin Fitting's work on bilattices and logic-programming semantics is a particularly strong lead because it provides two orderings rather than one:

```text
truth order      — comparison by truth/falsity orientation
knowledge order  — comparison by amount/consistency of information
```

This appears structurally well matched to our needs because “how true?” and “how much information?” are different questions.

A bilattice-style core could allow my-lisp to retain a small algebra while domain-specific reasoning layers add provenance, proof objects, confidence, source identity, timestamps, or other evidence without forcing everything into a boolean.

### Literature lead

- Melvin Fitting, **“Bilattices and the semantics of logic programming”**.
  OpenAlex result: https://openalex.org/W2001805670
  PDF lead: https://www.sciencedirect.com/science/article/pii/074310669190014G/pdf

## Four-valued / paraconsistent literature leads

- Ofer Arieli, Arnon Avron, **“The value of the four values”**.
  OpenAlex result: https://openalex.org/W1995375207
  PDF lead: https://www.sciencedirect.com/science/article/pii/S0004370298000320/pdf

- Didier Dubois, **“On Ignorance and Contradiction Considered as Truth-Values”**.
  OpenAlex result: https://openalex.org/W2114806338
  PDF lead: https://academic.oup.com/jigpal/article-pdf/16/2/195/2141754/jzn003.pdf

- Davide Ciucci, Didier Dubois, Jonathan Lawry, **“Borderline vs. unknown: comparing three-valued representations of imperfect information”**.
  OpenAlex result: https://openalex.org/W1989393769
  PDF lead: https://www.sciencedirect.com/science/article/pii/S0888613X14001157/pdf

- Umberto Rivieccio, **“An Algebraic Study of Bilattice-based Logics”**.
  OpenAlex result: https://openalex.org/W1650095214
  PDF lead: https://arxiv.org/pdf/1010.2552

- Félix Bou, Umberto Rivieccio, **“The logic of distributive bilattices”**.
  OpenAlex result: https://openalex.org/W2138601456
  PDF lead: https://philpapers.org/archive/BOUTLO-9.pdf

## Logic programming relevance

The research is not limited to philosophical many-valued truth tables. Existing work explicitly connects bilattices / four-valued semantics to logic programming and inconsistent knowledge bases.

Useful leads:

- Melvin Fitting, **“The family of stable models”**.
  https://openalex.org/W1979129485

- Bamshad Mobasher, Don Pigozzi, Giora Slutzki, **“Multi-valued logic programming semantics: an algebraic approach”**.
  https://openalex.org/W2022729059

- João Alcântara, Carlos Viegas Damásio, Luís Moniz Pereira, **“An encompassing framework for Paraconsistent Logic Programs”**.
  https://openalex.org/W2048639548

- Ofer Arieli, Marc Denecker, Bert Van Nuffelen, Maurice Bruynooghe, **“Coherent Integration of Databases by Abductive Logic Programming”**.
  https://openalex.org/W2117694283

This path is especially relevant to `lib/unify.lisp` and `lib/reason.lisp`: we want to preserve unification, substitutions, proof trees, provenance, standardizing apart, and backward chaining while replacing the old boolean/no-proof collapse.

## Candidate families — current assessment

### Kleene K3

Core shape:

```text
TRUE
FALSE
INDETERMINATE
```

Potential strength: partial computation / undefinedness.

Primary risk for my-lisp: a single `INDETERMINATE` can lose the distinction between:

```text
no positive or negative support
vs
simultaneous positive and negative support
```

Therefore K3 remains a candidate/baseline but is unlikely to be sufficient by itself for inconsistent knowledge.

### Łukasiewicz many-valued logics

Important historical and mathematical family. Needs closer study for our use-case.

Research question: does its interpretation of intermediate truth values model the epistemic/provenance distinctions we need, or does it answer a different problem?

Do not equate “intermediate truth degree” with ignorance or contradiction without evidence.

### Priest LP / paraconsistent logic

High relevance for contradiction without explosion.

Potential weakness as the sole richer logic: it is especially good at information gluts (`P` and `¬P`) but may not by itself provide the symmetric treatment of information gaps that Belnap/FDE-style systems do.

### Fuzzy logic

Relevant only when the problem is genuine vagueness or degree-of-truth.

Example:

```text
“temperature is hot” = degree 0.7
```

That is not the same as:

```text
we have insufficient evidence
we have conflicting evidence
probability is 0.7
```

Do not use fuzzy values as a universal replacement for uncertainty.

### Probabilistic logic

Treat separately from many-valued logical status.

Probability may quantify uncertainty, but it does not automatically encode whether the system has support for `P`, support for `¬P`, both, or neither.

## Working architecture hypothesis — NOT YET RATIFIED

The most promising direction currently is:

```text
outer ()
    = no answer produced

exact ℚ binary layer
    = 0/1 or 1/1

information/reasoning layer
    = Belnap/FDE-like support state

possible algebraic implementation
    = bilattice / Fitting-style semantics

orthogonal domain enrichments
    = proof/provenance
    = fuzzy degree when genuinely vague
    = probability when genuinely stochastic/uncertain
```

An even stronger possibility is that the richer layer should not expose a magic enum at all. It may derive the logical information state from explicit support objects:

```lisp
(support P source/proof/...)
(support (not P) source/proof/...)
```

and calculate the four-state classification as a projection.

This would preserve information instead of replacing evidence with labels.

## Required experiments before any choice

1. P supported only -> TRUE-ONLY.
2. ¬P supported only -> FALSE-ONLY.
3. P and ¬P both supported -> BOTH; no explosion to arbitrary Q.
4. neither supported inside an applicable richer logic -> NEITHER.
5. no applicable richer logic at all -> outer `()`.
6. attach independent proof/provenance to positive and negative support.
7. run a backward-chaining query through the same distinction.
8. compare K3 behavior on cases 3 and 4 to Belnap/FDE behavior.
9. test whether bilattice operators can remain Lisp-authored ordinary data/functions rather than host semantic magic.

## Comparison matrix to fill

| Family | Gap | Conflict | Paraconsistent | Logic-programming semantics | Degree/continuum | Outer `()` compatible | Main risk |
|---|---|---|---|---|---|---|---|
| Kleene K3 | yes | conflates/weak | generally no | yes/related partial semantics | no | yes | one indeterminate state may lose gap-vs-glut distinction |
| Belnap–Dunn / FDE | yes | yes | yes | yes, esp. via bilattice work | no | yes | need careful separation of information-state vs truth-value language |
| Bilattices / Fitting | yes | yes | yes | strong | not inherently | yes | richer algebra; must keep implementation small and language-owned |
| Priest LP | weaker gap story | yes | yes | yes | no | yes | glut-focused if used alone |
| Łukasiewicz | intermediate values | not primary aim | depends on variant | some | finite/infinite | yes | may model degree/intermediate truth rather than evidence structure |
| Fuzzy | degree | not by itself | not by itself | domain-specific | yes | yes | vagueness != ignorance != probability |
| Probabilistic | uncertainty | not by itself | not by itself | probabilistic LP exists | yes | yes | probability != logical information state |

This matrix is provisional and must be replaced/refined with primary-source definitions and executable witnesses.

## Research discipline

Do not select a “winner” because its vocabulary resembles our current architecture. Selection requires:

- primary-source definitions;
- exact operations / tables / orderings;
- executable Lisp examples;
- interaction with #219 reasoning/provenance;
- preservation of outer `()`;
- explicit trade-offs.

## Next reading pass

Priority order:

1. Belnap/Dunn four-valued semantics and information interpretation.
2. Fitting bilattices and logic-programming fixed-point semantics.
3. Arieli/Avron four-valued computational reasoning.
4. Kleene K3 as a minimal partiality baseline.
5. Priest LP as contradiction-focused comparison.
6. Łukasiewicz as a different many-valued tradition.
7. Fuzzy/probabilistic approaches only as orthogonal domain logics, unless evidence suggests otherwise.

## Current non-conclusion

No richer logic has been ratified yet.

The strongest evidence lead is currently **Belnap–Dunn + bilattice/Fitting semantics**, because it separately represents gaps and conflicts and already has logic-programming literature. But #223 remains a research issue until we have primary-source comparison and executable my-lisp witnesses.

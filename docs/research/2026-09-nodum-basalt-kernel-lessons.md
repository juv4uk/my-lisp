# Lessons from Nodum and Basalt for the ground graph kernel

Status: research input, not semantic authority.

This note records mechanisms observed in the forked projects `juv4uk/nodum`
and `juv4uk/basalt`, and separates ideas useful to the experimental kernel
from UI/storage details that should not become semantic authority.

## 1. Graph as a derived projection, not the source of truth

Nodum stores note content separately and derives graph edges from it. Its graph
API returns a projection:

- nodes
- edges as node-index pairs
- optional local center
- metadata such as degree and unresolved status

The renderer is therefore not authoritative. It can be rebuilt.

Kernel lesson:

```text
authoritative relation data
        |
        +--> derived index/cache
        |
        +--> local graph projection
        |
        +--> Mermaid/WebGL visualization
```

The visual graph must never define meaning.

## 2. Unresolved / ghost nodes

Nodum persists a link even when the target note does not yet exist.
The target text survives while the concrete target ID is null. When a matching
note appears later, the existing link can resolve without rewriting the source
note.

This is highly relevant to the ground graph experiment.

Possible kernel analogue:

```text
relation exists
A --------?> X

X has not yet acquired / exposed a concrete representation
```

Later:

```text
X <-> 00101100
```

The pre-existing relation can now become traversable through the newly known
representation.

Important: "ghost" is only our explanatory term. Do not encode a privileged
Ghost type unless an experiment demonstrates that such a type is necessary.

## 3. Late binding without rewriting the source relation

Nodum resolves old links when a target later becomes available. It does not
silently rewrite the user's Markdown just because the resolved title/path
changed.

Kernel lesson:

- preserve the originally asserted relation;
- maintain resolution as derived state;
- identity resolution may change while the original evidence remains intact.

This fits the provenance discipline:

```text
asserted relation != current resolution
```

## 4. Aliases as multiple surfaces of one identity

Nodum can resolve title, path, and aliases to the same note identity. It also
tries to avoid silently choosing an ambiguous alias when editing links.

Kernel lesson:

```text
representation/surface A \
representation/surface B  >-- same identity
representation/surface C /
```

This is close to the direction previously attempted with language columns, but
the graph formulation is cleaner: EN/UK/SA spellings need not be columns.
They can become peers in an equivalence relation.

Do not copy Nodum's fallback rule where an arbitrary stable match can win among
colliding titles. For a semantic kernel, ambiguity should remain explicit.

## 5. Backlinks are a derived inverse view

Nodum stores directed outgoing links, but can derive backlinks efficiently.

Kernel lesson:

A graph relation does not need two separately authoritative records:

```text
stored/evidenced: A --R--> B
derived query:    who --R--> B ?
```

This is distinct from reversible computation. A derived backlink is an inverse
*query view*, not proof that the transition itself is physically reversible.

That distinction should remain explicit.

## 6. Local graph / bounded observation horizon

Nodum exposes a local graph around a note with a configurable hop depth.

Kernel lesson:

Instead of requiring a whole graph traversal, experiments can define an
observation horizon:

```text
observe(start, relation-space, depth=N)
```

This can become useful for:
- bounded proofs;
- hardware execution;
- avoiding accidental global search;
- comparing graph growth step by step.

Depth is an observer parameter, not intrinsic meaning of the graph.

## 7. Multiplicity without duplicating edges

Nodum stores one logical source->target link row plus a count of occurrences.

Kernel lesson:

If the same relation is witnessed multiple times, separate:
- identity of the relation;
- multiplicity / evidence count.

Do not multiply semantic edges merely because the same evidence appeared twice.

## 8. Self-links are legitimate graph structures

Nodum permits self-links in the graph.

Kernel lesson:

Do not ban:

```text
A --R--> A
```

Self-relation may later be useful for fixed points, identity witnesses,
recursive structures, or stable states.

## 9. Incremental maintenance

Nodum updates the link graph when a note changes instead of reconstructing all
content semantically from scratch.

Kernel lesson:

If graph state eventually becomes large, keep:
- authoritative relation data;
- incremental derived indexes;
- reproducible rebuild path.

An index is expendable mechanism, never meaning.

## 10. Time/growth as an observation dimension

Nodum's graph UI contains a time-travel view based on node creation order.

Kernel lesson:

Our experiments should preserve provenance/order so graph growth can be
observed:

```text
G0 -> G1 -> G2 -> ...
```

This does not mean time must become a semantic primitive. It is initially an
observation/debugging dimension.

## 11. Basalt lesson: plain files and minimal authority

The forked `juv4uk/basalt` is a Rust terminal application for working with
Obsidian vault files. It is not the graph-heavy Basalt previously assumed.

The useful lesson is simpler:

- plain files remain usable outside the application;
- the UI does not own the knowledge;
- a tool can operate over existing content without converting it into a
  proprietary store.

For our ecosystem:

```text
ground graph / research files
        |
        +--> Lisp apparatus
        +--> Markdown
        +--> Mermaid
        +--> Nodum-like graph viewer
```

No viewer should become required to interpret the experiment.

## 12. Ideas to adopt next

Most promising:

1. **Unresolved relation endpoints**: allow relation evidence to exist before a
   concrete endpoint representation is known.
2. **Late resolution**: connect that endpoint to a representation later without
   rewriting the original relation.
3. **Equivalence aliases**: multiple spellings/representations resolve to one
   identity without fixed registry columns.
4. **Derived inverse queries**: backlinks without storing duplicate authority.
5. **Bounded local observation**: explore a graph by hop depth.
6. **Provenance-preserving growth**: retain the order/evidence by which graph
   structure appeared.

Do not adopt from Nodum into the kernel:

- UUID as semantic identity;
- note/title/path categories;
- PostgreSQL schema;
- `[[wikilink]]` syntax as semantic syntax;
- degree-based importance as meaning;
- renderer indices as identities;
- arbitrary collision resolution.

Those are application mechanisms, not kernel semantics.

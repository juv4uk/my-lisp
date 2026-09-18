# Nodum / cosmos.gl graph-view study

Date: 2026-09-19
Status: research input for the ground-graph experiment.

## Why Nodum is useful

Nodum is an open-source Obsidian-style knowledge base. Its graph view uses
`@cosmos.gl/graph` on WebGL2. The useful lesson for this repository is not
Nodum's server architecture; it is the thin renderer contract:

- points form one index space;
- links are source/target point-index pairs;
- the renderer owns layout, zoom, dragging, forces and visual channels.

Upstream references:

- https://github.com/nodummd/nodum
- https://github.com/cosmosgl/graph
- https://github.com/nodummd/nodum/blob/main/tasks/nodum-master-plan.md

Nodum's own plan records that the graph view ships with node size by degree,
ghost nodes, search/tag filters, force sliders and an HTML label overlay. It
also records an important implementation detail: cosmos.gl receives a point
space and links, while settled positions are observed separately from the
initial seed positions.

## cosmos.gl data contract

The current cosmos.gl API consumes flat typed arrays:

```text
point positions:
[x0, y0, x1, y1, ...]

links:
[src0, dst0, src1, dst1, ...]
```

Point sizes/colors/shapes and link widths/colors are parallel channels.

This is intentionally lower-level than the experimental semantic graph. That is
good: the renderer should not know what an empty list, representation, relation
or rewrite means.

## Important mismatch with our graph

Our experimental edge is ternary:

```text
(left relation right)
```

and `relation` is itself an identity, not an external edge label.

Flattening this to a conventional renderer edge:

```text
left -------- right
       "relation"
```

would throw away the strongest property of the experiment.

Instead project one ternary relation to two ordinary visual links:

```text
left ---- relation ---- right
```

All three values stay ordinary graph nodes.

For the current ground graph:

```text
() -- 00000010 -- 00000000 -- 00000011 -- 00000001
```

The visual renderer is therefore a projection only. It is never graph
authority.

## Repository experiment

`experiments/ground-graph-cosmos.lisp` performs the first projection:

```text
ground-graph
    |
    v
two-endpoint link pairs
    |
    v
renderer-facing points + links
```

It deliberately does not assign coordinates or numeric point IDs yet. That
next adapter should be semantics-blind interning only:

```text
opaque graph value -> renderer point index
```

For example:

```text
()       -> point 0
00000010 -> point 1
00000000 -> point 2
00000011 -> point 3
00000001 -> point 4
```

The numbers above are renderer-local indices, not semantic IDs.

Then cosmos.gl can receive links such as:

```text
[0,1, 1,2, 2,3, 3,4]
```

without learning anything about what those nodes mean.

## What not to copy from Nodum

Do not import its Postgres/Redis/FastAPI note graph as graph authority.
Do not use note titles or strings as semantic identities.
Do not let WebGL point indices become stable language IDs.
Do not make UI directionality define logical directionality.
Do not flatten first-class relation nodes into decorative labels.

## What is worth copying

- GPU point/link renderer separation.
- local graph extraction as a view, not an authority.
- hover/click selection.
- force controls.
- labels as an overlay rather than graph identity.
- graceful WebGL failure/fallback.
- ability to patch the visual graph incrementally.

## Next experiment

Add a tiny renderer-index interner:

```text
ground graph values
        -> deterministic local point indices
        -> Float32Array links
        -> optional cosmos.gl viewer
```

The interner must prove that changing renderer indices cannot alter graph
identity or experimental evaluation.

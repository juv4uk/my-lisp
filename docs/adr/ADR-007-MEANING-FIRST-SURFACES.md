# ADR-007 — Meaning-first human surfaces

**Status:** Accepted  
**Date:** 2026-09-08  
**Authority migration:** completed by ADR-008

## Decision

`my-lisp` has no privileged human programming language.

The semantic identity comes first. Human-language spellings are peer names of
that identity. No human surface may be implemented by resolving through
another human surface.

```text
                         meaning
                            │
              ┌─────────────┼─────────────┐
              │             │             │
             UK            EN            SA
```

The same shape applies to Canon 0+7, ordinary builtins, Lisp-defined public
functions, macros and future public semantic identities.

For Canon, the meaning is immutable. For ordinary public API, the meaning may
evolve through the normal language-contract process, but the surface topology
is the same.

## Fundamental invariant

Forbidden:

```text
UK -> EN -> meaning
SA -> EN -> meaning
EN -> UK -> meaning
```

Required:

```text
UK --\
EN ----> meaning
SA --/
```

A surface spelling is not an alias of another surface spelling. It is a direct
name of the same semantic identity.

## Numeric semantic identities

Semantic identity handles contain digits only. This is a neutrality rule, not
a formatting preference. Alphabetic prefixes are forbidden because even a
seemingly generic prefix can import a human-language word into the machine
identity layer.

Required:

```text
0001
0101
0104
```

Forbidden:

```text
m0104
id0104
meaning0104
```

The digits are opaque handles. They do not name the operation.

## Registry shape

```lisp
(sr/1
  (0001
    (uk як-є stable)
    (en quote stable)
    (sa svarūpa stable)
    (sym ' stable))

  (0101
    (uk відобразити stable)
    (en map stable)
    (sa āvartana candidate)))
```

The order of surface rows has no semantic significance. A future language is
added by one more peer row; no schema redesign is required.

`+`, `-`, `<` and other punctuation are not human-language spellings. ADR-008
makes them explicit members of the shared non-human `sym` surface.

## Complete human surface

A human surface may be called complete only when it has, for every selected
public semantic identity:

- a stable spelling;
- direct resolution to that identity, not through another human surface;
- executable semantic-equivalence evidence;
- an acceptance program using that surface;
- human presentation and diagnostics appropriate to that surface.

A new or incomplete surface may honestly use `candidate` or `missing`; it must
not silently fall back through EN, UK, SA, or any other human language.

## Migration status

The transitional rule in the first revision of this ADR allowed
`lib/surface/uk-sa-coverage.wsm` to remain authoritative while the numeric
registry was only a seed. **That transition is finished.** ADR-008 establishes
`lib/surface/semantic-registry.wsm` as the sole machine authority and demotes
the old EN-shaped table to historical audit.

This completes authority/schema neutrality. It does **not** claim that every
public runtime value already has one shared `Rc`/closure for all human
spellings. Runtime peer-binding parity remains separate executable work.

## Relationship to ADR-005

ADR-005 established that EN, UK and SA are peer human surfaces and that core is
not English. This ADR generalizes that decision to every present and future
human surface. ADR-008 supersedes ADR-005's temporary legacy-registry
interpretation while preserving its peer-language principle.

> **Meaning first. Languages are peers.**
>
> **Значення первинне. Мови рівноправні.**

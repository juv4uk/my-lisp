# ADR-007 — Meaning-first human surfaces

**Status:** Accepted  
**Date:** 2026-09-08

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

Semantic identity handles contain digits only.

This is a neutrality rule, not a formatting preference. Alphabetic prefixes are
forbidden because even a seemingly generic prefix such as `m` silently imports
a word from one human language (for example, "meaning") into the machine
identity layer.

Required examples:

```text
0001
0101
0104
```

Forbidden examples:

```text
m0104
id0104
meaning0104
```

The digits are opaque handles. They do not name the operation and do not encode
a human-language word. Their only job is to provide one stable, language-neutral
reference to one semantic identity.

## Minimal registry form

The migration uses a deliberately small Lisp-shaped data format:

```lisp
(sr/1
  (0001
    (uk як-є stable)
    (en quote stable)
    (sa svarūpa stable))

  (0101
    (uk відобразити stable)
    (en map stable)
    (sa āvartana candidate)))
```

`0001`, `0101`, ... are opaque numeric machine identities. They are
intentionally not words from UK, EN, SA, or any future human surface.

The order of `(uk ...)`, `(en ...)`, `(sa ...)` has no semantic significance.
A future language is added by adding another peer form, for example:

```lisp
(0101
  (uk відобразити stable)
  (en map stable)
  (sa āvartana candidate)
  (pl odwzoruj stable))
```

No schema redesign is required.

## What this does not mean

1. Human languages do not need equal completion status at every moment. A new
   surface may be partial while it is being built.
2. A partial surface must not be called complete.
3. Adding a language must not create another implementation of the operation.
4. Internal Rust names are implementation details; they are not human-surface
   authority.
5. Canon remains more strongly protected than ordinary API: peer naming does
   not make every ordinary function immutable.

## Complete human surface

A human surface may be called complete only when it has, for every selected
public semantic identity:

- a stable spelling;
- direct resolution to that identity, not through another human surface;
- executable semantic-equivalence evidence;
- an acceptance program using that surface;
- human presentation and diagnostics appropriate to that surface.

## Migration rule

The existing `lib/surface/uk-sa-coverage.wsm` remains authoritative during the
migration so current proofs are not weakened. `lib/surface/semantic-registry.wsm`
is the neutral successor schema and begins as a small executable-design seed.
Rows move into it in proved slices.

A row is migrated only when all currently represented surface spellings can be
shown to designate the same semantic identity directly. The legacy table is
removed only after the neutral registry covers the full selected public
surface and existing coverage/acceptance gates consume it.

## Relationship to ADR-005

ADR-005 established that EN, UK and SA are peer human surfaces and that the core
is not English. This ADR generalizes that decision:

- peer status applies to every present and future human surface;
- no surface may be the implementation substrate of another;
- the machine registry itself must not use a human-language spelling as the
  semantic identity;
- semantic identity handles themselves must be numeric-only.

The short project law is:

> **Meaning first. Languages are peers.**
>
> **Значення первинне. Мови рівноправні.**

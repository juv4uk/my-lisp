# Host Semantic Surface (HSS)

Status: living architecture inventory. This document measures what the host must still provide to make the current language/system observable. It is not a language-contract version.

## Definition

**Host Semantic Surface** is the set of host operations whose removal would eliminate an observation/effect that the language cannot derive from already-available values.

This is deliberately stricter than counting Rust functions or lines of code.

```text
host code size != host semantic surface
```

A large parser implementation can represent one necessary mechanism. A one-line host builtin can be semantically redundant if Lisp can derive it from another observation.

## Decision test

For every host-facing operation ask, in order:

1. Does it expose a new external observation or effect?
2. If not, can the same result be expressed from existing language values?
3. If yes, move that meaning/policy to Lisp, add deterministic tests, prove language ownership, then remove the host duplicate.
4. Do not delete the old host path before the replacement is exercised by CI.

This is the same surgical sequence used for `mono-ms` and now for `utc-now`.

## Current time boundary

| Operation | Current owner | Classification | Direction |
|---|---|---|---|
| `mono-ns` | Rust host | irreducible monotonic observation | KEEP |
| `mono-ms` | `lib/time.my` | derived unit view | HOST REMOVED |
| `unix-time-now` | Rust host | wall-clock observation | KEEP |
| `civil-from-days` | `lib/time.my` | deterministic Gregorian semantics | LANGUAGE-OWNED |
| `utc-from-unix` | `lib/time.my` | deterministic UTC interpretation | LANGUAGE-OWNED |
| `utc-now` | `lib/time.my` after time library load | derived public clock meaning | MIGRATING: remove old Rust duplicate after ownership/consumer audit |
| `internet-time-sync` | Rust host | UDP/NTP observation | KEEP mechanism; interpretation stays in Lisp |
| `internet-time-observation->utc` | `lib/time.my` | interpretation/policy | LANGUAGE-OWNED |
| `timezone-detect` | Rust host | host environment observation | KEEP observation |
| `timezone-config` and selectors | `lib/time.my` | configuration semantics | LANGUAGE-OWNED |
| deadline arithmetic | `lib/time.my` | deterministic policy | LANGUAGE-OWNED |

The intended wall-clock chain is:

```text
OS clock
  ↓
Rust: unix-time-now
  ↓
(unix-time seconds nanosecond)
  ↓
Lisp: utc-now / utc-from-unix
  ↓
(utc year month day hour minute second nanosecond)
```

Rust should eventually have no reason to know Gregorian month/day conversion merely to expose the current time.

## Core capability boundary

The core crate should remain capability-minimal. Host/environment access belongs either in an explicit capability crate or in a narrowly justified observation primitive.

Current broad classes:

| Capability class | Host responsibility | Lisp responsibility |
|---|---|---|
| monotonic time | read monotonic counter | units, elapsed time, deadlines, scheduling policy |
| wall clock | read Unix timestamp | calendar interpretation, formatting, comparison policy |
| network time | bounded packet I/O / raw timestamp observation | acceptance policy, calendar meaning, sync strategy |
| timezone | observe host declaration | explicit configuration and conversion policy |
| filesystem | bytes/text read-write mechanisms | naming policy, package formats, transactional/world semantics |
| subprocess | process creation/I/O mechanism | command policy, orchestration, interpretation |
| TCP | socket mechanism | protocol, routing, retries, claims, application semantics |
| terminal/stdin | host input/output mechanism | parsing/evaluation meaning |

## Suspect list

The following categories should be repeatedly audited because they often drift from mechanism into policy:

- calendar/date shaping in Rust;
- unit conversions implemented as separate builtins;
- protocol result shaping that can be expressed as Lisp data transforms;
- retries/timeouts/backoff policy hidden inside host adapters;
- path/package naming policy in filesystem adapters;
- Guard/swarm policy inside Rust rather than data/rules;
- special forms whose evaluation control can be decomposed into a smaller substrate plus Lisp-owned expansion.

“Suspect” does not mean “wrong”. It means the operation must justify remaining host-owned.

## What is not a target for blind removal

Do not optimize HSS by pretending mechanisms do not exist. The following may legitimately remain implementation-level:

- memory-safe runtime representation;
- parser/reader machinery;
- lexical environment/closure mechanism;
- stack-safety mechanisms;
- exact integer/rational low-level arithmetic where the language contract requires exactness;
- structured diagnostics/error transport;
- genuine OS/hardware capabilities;
- implementation-specific performance mechanisms that do not change observable language semantics.

The rule is not “Rust bad, Lisp good.” The rule is ownership by semantic nature.

## Measurement

Track HSS qualitatively first. A numeric score is useful only when entries have stable identity and scope.

A valid decrease looks like:

```text
host primitive A
+ language replacement
+ deterministic equivalence tests
+ ownership test
+ all consumers migrated
+ host primitive removed
= one evidenced HSS reduction
```

Simply moving lines of code does not count.

## Recent evidenced reductions

### `mono-ms`

Before:

```text
Rust: mono-ns + mono-ms
```

After:

```text
Rust: mono-ns
Lisp: milliseconds-from-nanoseconds + mono-ms
```

The language-owned binding is tested, and the Rust `mono-ms` builtin was removed.

### UTC calendar semantics

Current transition:

```text
Rust: unix-time-now + legacy utc-now duplicate
Lisp: civil-from-days + utc-from-unix + utc-now
```

Next removal gate: prove all intended time-library consumers load the Lisp definition, then delete the legacy Rust `utc-now` calendar implementation and its helper without breaking conformance/CI.

## Principle

> Rust gives the system doors to the world. Lisp decides what the observations mean.

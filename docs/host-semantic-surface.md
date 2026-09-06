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

This is the same surgical sequence used for `mono-ms`, `utc-now`, and the NTP interpretation split.

## Current time boundary

| Operation | Current owner | Classification | Direction |
|---|---|---|---|
| `mono-ns` | Rust host | irreducible monotonic observation | KEEP |
| `mono-ms` | `lib/time.my` | derived unit view | HOST REMOVED |
| `unix-time-now` | Rust host | wall-clock observation | KEEP |
| `civil-from-days` | `lib/time.my` | deterministic Gregorian semantics | LANGUAGE-OWNED |
| `utc-from-unix` | `lib/time.my` | deterministic UTC interpretation | LANGUAGE-OWNED |
| `utc-now` | `lib/time.my` | derived public clock meaning | HOST REMOVED |
| raw NTP query | Rust host | bounded UDP query + extraction of fixed-width response fields | KEEP mechanism |
| `internet-time-sync` | `lib/time.my` after time-layer load | public NTP interpretation | LANGUAGE-OWNED |
| `internet-time-fields->observation` | `lib/time.my` | mode/stratum validation, NTP epoch conversion, fraction-to-nanoseconds | LANGUAGE-OWNED |
| `internet-time-observation->utc` | `lib/time.my` | calendar interpretation/policy | LANGUAGE-OWNED |
| `timezone-detect` | Rust host | host environment observation | KEEP observation |
| `timezone-config` and selectors | `lib/time.my` | configuration semantics | LANGUAGE-OWNED |
| deadline arithmetic | `lib/time.my` | deterministic policy | LANGUAGE-OWNED |

The wall-clock chain is now:

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

Rust no longer contains Gregorian month/day conversion merely to expose the current time. The former `civil_from_days`, `utc_now_value`, and root `utc-now` builtin were removed after the language-owned loader path, ownership tests, consumer bootstrap audit, and CI were green.

The NTP chain is now:

```text
UDP socket / NTP packet
  ↓
Rust: bounded query + fixed-width field extraction
  ↓
(ntp-fields host mode stratum ntp-seconds fraction)
  ↓
Lisp: internet-time-raw->observation
  ↓
Lisp: internet-time-fields->observation
  ↓
(accepted host unix-seconds nanosecond)
  ↓
Lisp: internet-time-observation->utc
```

Rust no longer decides whether mode/stratum are semantically acceptable, no longer translates the NTP epoch to Unix time, and no longer computes the fractional second in nanoseconds. Those transformations are deterministic language-owned semantics. Transport failures and short packets remain host-level observations because they arise before a complete protocol field set exists.

The root host builtin still temporarily carries the historical public name `internet-time-sync`; `lib/time.my` captures it as the raw capability and replaces the public binding with a Lisp closure. Renaming the host-only capability to a neutral mechanism name is cleanup, not a semantic migration, and should happen only with the same ownership tests and green CI discipline.

## Core capability boundary

The core crate should remain capability-minimal. Host/environment access belongs either in an explicit capability crate or in a narrowly justified observation primitive.

Current broad classes:

| Capability class | Host responsibility | Lisp responsibility |
|---|---|---|
| monotonic time | read monotonic counter | units, elapsed time, deadlines, scheduling policy |
| wall clock | read Unix timestamp | calendar interpretation, formatting, comparison policy |
| network time | bounded packet I/O + extraction of raw response fields | protocol acceptance, epoch conversion, units, calendar meaning, sync strategy |
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

### UTC calendar semantics and `utc-now`

Before:

```text
Rust: unix-time-now + utc-now + Gregorian calendar conversion
Lisp: civil-from-days + utc-from-unix + utc-now replacement
```

After:

```text
Rust: unix-time-now
Lisp: civil-from-days + utc-from-unix + utc-now
```

The normal CLI, plain TCP REPL, and sexpr/oracle bootstrap paths load the language-owned time layer. The ownership test requires `utc-now` to be absent from the root host environment and to appear only as a Lisp closure after `load_time_library`. CI passed before the Rust duplicate and Gregorian helper were removed.

### NTP response semantics

Before:

```text
Rust:
  UDP query
  + mode/stratum acceptance
  + NTP epoch conversion
  + fraction -> nanoseconds
  + public accepted/rejected timestamp shape
```

After:

```text
Rust:
  UDP query
  + fixed-width field extraction

Lisp:
  mode/stratum acceptance
  + NTP epoch conversion
  + fraction -> nanoseconds
  + public internet-time-sync meaning
```

Deterministic fixtures prove mode 4/5 acceptance, invalid mode/stratum rejection, epoch rejection, and exact `2147483648 -> 500000000 ns` conversion. The public `internet-time-sync` binding is required to be a Lisp closure after the time layer loads while the captured raw capability remains a Rust builtin. CI #884 passed before this HSS status was updated.

## Principle

> Rust gives the system doors to the world. Lisp decides what the observations mean.

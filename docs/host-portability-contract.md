# Portable host substrate contract / Контракт переносимого host-substrate

Status: architectural contract v0, evidence-oriented. This document does **not** claim that the current HSS is final or minimal. It defines the direction required for one Lisp semantic layer to move between different hosts with the smallest possible adapter rewrite.

## Goal

The portability target is not "port every library to every platform". The target is:

```text
Host A ─┐
Host B ─┼─> minimal mechanism contract ─> the same Lisp semantic libraries
Host C ─┘
```

A host implementation may be Rust/Linux, Rust/Windows, WASM, an embedded runtime, FPGA/SoC firmware, or another implementation. If it exposes the same mechanism-level observations/effects, the higher Lisp layers should not need platform-specific semantic rewrites.

## Ownership rule

A host operation belongs in the portability substrate only when its removal would remove a genuinely external observation/effect or an irreducible runtime representation mechanism.

Deterministic interpretation of already-available values belongs above the substrate whenever Lisp can express it.

```text
host: mechanism / observation / effect
Lisp: interpretation / policy / protocol / derived meaning
```

This is the same distinction already evidenced by the time, process and TCP reductions.

## Current evidenced raw boundaries

### Process

```text
OS process
  ↓
process-run-raw
  ↓
(exit-status-or-() stdout-bytes stderr-bytes)
  ↓
Lisp UTF-8 / result policy
  ↓
process-run
```

The host starts the process and observes raw output bytes. Lisp owns text interpretation and the public result shape.

### TCP

```text
public listen policy:
Lisp tcp-listen(port)
  ↓ chooses "0.0.0.0" compatibility default
Lisp tcp-listen-on(address, port)
  ↓
host tcp-listen-raw(address, port)

read:
socket → tcp-read-raw → bytes → Lisp decode → tcp-read

write:
Lisp tcp-write → Lisp encode → bytes → tcp-write-raw → socket
```

The host owns socket mechanisms. UTF-8 encoding/decoding and the historical default listen address are language-owned.

### Time

```text
host observation → raw timestamp/field/declaration values → Lisp interpretation
```

Examples already in the repository include `mono-ns`, `unix-time-now`, `ntp-query-raw`, and `timezone-declarations-raw`, with derived/public meanings in `lib/time.my`.

## Candidate portable mechanism set

The following is a **candidate**, not a frozen ABI. Every entry must continue to justify itself under HSS audit.

| Area | Mechanism-level surface | Semantic layer above it |
|---|---|---|
| Process | `process-run-raw` | text decoding, exit/result policy, orchestration |
| TCP transport | `tcp-connect`, `tcp-listen-raw`, `tcp-accept`, `tcp-read-raw`, `tcp-write-raw`, `tcp-close` | default bind policy, UTF-8, framing, protocol, retry/backoff, routing |
| Files | `read-file-bytes`, `write-file-bytes`, directory observation | text decoding, package formats, naming/policy |
| Monotonic clock | raw monotonic observation | units, deadlines, scheduling policy |
| Wall clock | raw Unix-time observation | UTC/calendar semantics |
| Network time | raw packet/field observation | acceptance rules, epoch conversion, sync policy |
| Timezone | raw declaration candidates | precedence and public detection meaning |

Current convenience/text capabilities such as `read-file`, `write-file`, and `load` are **not automatically part of the final minimal portability ABI** merely because they exist today. They remain audit targets.

## Runtime representation bridges are separate

Some mechanisms are not OS host capabilities but still cannot be derived from Lisp data alone because they materialize runtime representation. Examples are the Unicode scalar/string bridges used by language-owned UTF-8:

```text
codepoint->string
string->codepoint
```

These do not define UTF-8. They expose the runtime's string representation at the smallest useful boundary. A different runtime may implement them differently while preserving the same language-level contract.

## Porting test

A new host should be judged in this order:

1. Implement the smallest required observation/effect mechanisms.
2. Run the same language-owned semantic libraries unchanged.
3. Run the same deterministic conformance and ownership tests.
4. Add platform-specific code only when an external mechanism truly differs.
5. If a platform port requires reimplementing UTF-8, calendar rules, NTP meaning, bind defaults, protocol policy, or similar deterministic semantics in the host, treat that as evidence that the boundary leaked upward again.

## Non-goal

The contract does not require every host to support every capability. A WASM host may intentionally omit raw TCP or process execution; an FPGA target may expose UART, Ethernet frames, or memory-mapped devices instead. Capability absence should be explicit.

Portability therefore means:

```text
same capability identity + same observable contract
when that capability exists
```

not "all platforms pretend to have identical hardware".

## FPGA consequence

For an FPGA/SoC target, the long-term objective is not to reproduce Rust. It is to implement the required mechanism contract directly against hardware/firmware and then reuse the Lisp-owned layers above it.

```text
Linux syscalls ─┐
Windows APIs  ──┼─> mechanism contract ─> Lisp semantics
FPGA hardware ──┘
```

This is why each successful HSS reduction matters: every deterministic rule moved out of the host is one fewer rule that a future host port must reproduce.

## Evidence rule

Do not declare an operation portable merely because two implementations have the same name. Evidence requires observable equivalence tests at the boundary and ownership tests showing that derived public semantics are not silently duplicated in the host.

> The language defines the meaning; each host proves that it supplies the required mechanisms.

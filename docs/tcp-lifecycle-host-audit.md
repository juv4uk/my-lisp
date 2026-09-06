# TCP lifecycle host audit / Аудит lifecycle-межі TCP

Status: architectural audit; no lifecycle cut yet. Text read/write semantics are already language-owned. This document audits the remaining `tcp-connect`, `tcp-listen`, `tcp-accept`, and `tcp-close` host surface before changing it.

## Current boundary

After the completed text I/O split, the TCP surface is conceptually:

```text
Lisp protocol / UTF-8 / application meaning
            ↓
  tcp-connect / tcp-listen / tcp-accept
  tcp-read-raw / tcp-write-raw
  tcp-close
            ↓
        OS sockets
```

The byte transport boundary is already mechanism-only. The remaining question is whether lifecycle calls still hide deterministic policy that can move upward.

## `tcp-connect`

Current host behavior:

```text
(host string, port integer)
  ↓
TcpStream::connect((host, port))
  ↓
TcpConnection
```

### Mechanism

- ask the OS/network stack to resolve/connect to the requested endpoint;
- create a runtime TCP connection handle;
- report transport failure.

### Semantics still mixed in

- port range/type validation is performed in the host adapter;
- transport errors are shaped directly as `LanguageError::InvalidForm` with host-generated text.

The first is boundary validation and may legitimately remain near the mechanism. The second is a future structured-error audit target: portable hosts should ideally expose equivalent failure identity without requiring identical OS error strings.

No cut is justified yet merely to move these few checks.

## `tcp-listen`

Current public call is:

```text
(tcp-listen port)
```

but the Rust host implements it by binding:

```text
("0.0.0.0", port)
```

This is the strongest lifecycle policy leak found in this audit.

`0.0.0.0` is not an unavoidable socket mechanism. It is a deterministic choice of bind scope: listen on all IPv4 interfaces. A different host could make a different default, which would make the same Lisp program observably platform-dependent.

### Candidate split

Do **not** implement this until the replacement is proved.

```text
Rust: tcp-listen-raw(address, port)
            ↑
Lisp: tcp-listen(port)
      chooses the compatibility default
      "0.0.0.0"
```

A future richer Lisp API could expose explicit loopback/interface binding without changing the raw host mechanism.

### Evidence gate

Before removing current host `tcp-listen` semantics:

1. add `tcp-listen-raw` with explicit bind address + port;
2. prove raw loopback/all-interface behavior with deterministic socket tests where practical;
3. define public `tcp-listen` in Lisp preserving today's `0.0.0.0` behavior;
4. prove `tcp-listen` is absent from host registry and appears as a Lisp closure after the TCP layer loads;
5. migrate consumers;
6. require workspace tests/build/clippy green;
7. only then remove the host-owned public form.

## `tcp-accept`

Current host behavior performs `listener.accept()` and returns only the accepted stream. The peer address observed by the OS is discarded.

Discarding the peer address is not automatically a semantic bug. HSS minimization does not require exposing every observable fact the OS can provide. If current Lisp semantics need only an accepted connection, returning the connection alone is a smaller capability.

Therefore **do not expand** `tcp-accept` merely because peer metadata exists. Add peer observation only if a language-level requirement demonstrates that it is needed.

This is an important anti-expansion rule for the portability contract.

## `tcp-close`

Current host calls:

```text
shutdown(Shutdown::Both)
```

This combines the mechanism "request socket shutdown" with the public meaning "close both directions".

There are two possible interpretations:

- `Both` is simply the current language contract for `tcp-close`, in which case the host may implement that effect directly;
- half-close semantics become language-relevant later, in which case a lower `tcp-shutdown-raw(direction)` substrate may be justified.

No present evidence requires half-close. Therefore introducing direction controls now would expand the substrate without a demonstrated need. Keep `tcp-close` for now and audit again only when a real language use-case requires finer lifecycle control.

## Result

The audit does **not** justify a broad TCP lifecycle rewrite.

It identifies one focused candidate:

```text
tcp-listen
  currently = socket bind mechanism + hidden 0.0.0.0 policy

candidate:
tcp-listen-raw(address, port)
  + Lisp-owned default bind policy
```

`tcp-connect`, `tcp-accept`, and `tcp-close` remain provisionally host-owned mechanisms/contracts until stronger evidence shows removable policy.

## Portability consequence

For "Lisp jumping between hosts", the important property is not that every OS uses the same socket API. It is that the same Lisp program does not silently inherit different host defaults.

A hidden bind-address default is exactly the kind of difference that should be moved into the language layer if we can preserve behavior with a smaller explicit mechanism.

## Principle

> Do not make the host richer to make it look raw. Make the boundary only as expressive as the language has evidence that it needs.

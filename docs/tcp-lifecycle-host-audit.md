# TCP lifecycle host audit / Аудит lifecycle-межі TCP

Status: focused `tcp-listen` policy cut completed; `tcp-connect`, `tcp-accept`, and `tcp-close` remain under audit. Text read/write semantics are already language-owned.

## Current boundary

The TCP surface now separates public listen policy from the host mechanism:

```text
Lisp protocol / UTF-8 / application meaning
            ↓
  tcp-connect
  Lisp tcp-listen → tcp-listen-raw(address, port)
  tcp-accept
  tcp-read-raw / tcp-write-raw
  tcp-close
            ↓
        OS sockets
```

The byte transport boundary is mechanism-only, and the historical bind-address default is no longer hidden in Rust.

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

## `tcp-listen` — completed focused cut

Historically the public call was:

```text
(tcp-listen port)
```

while the Rust host silently bound:

```text
("0.0.0.0", port)
```

That mixed mechanism with deterministic bind-scope policy. The split is now:

```text
Lisp: tcp-listen(port)
      ↓ chooses compatibility default "0.0.0.0"
Lisp: tcp-listen-on(address, port)
      ↓
Rust: tcp-listen-raw(address, port)
      ↓
OS bind
```

The host registry contains `tcp-listen-raw` and no public `tcp-listen`. After `load_tcp_library`, `tcp-listen` appears as a Lisp closure. Existing callers keep the historical all-IPv4 default, while callers can now request an explicit address without expanding the host API.

The dedicated raw-bind witness binds `127.0.0.1` through `tcp-listen-raw` and proves that a real TCP client can connect to that requested address. Existing TCP integration tests continue to exercise the public compatibility path through Lisp-owned `tcp-listen`.

This is an HSS reduction because a platform-dependent default moved out of the host without removing the underlying socket effect.

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

The lifecycle audit has produced one evidenced reduction:

```text
before:
Rust tcp-listen(port)
  = socket bind + hidden 0.0.0.0 policy

after:
Rust tcp-listen-raw(address, port)
Lisp tcp-listen(port)
  = explicit language-owned 0.0.0.0 compatibility policy
```

`tcp-connect`, `tcp-accept`, and `tcp-close` remain provisionally host-owned mechanisms/contracts until stronger evidence shows removable policy.

## Portability consequence

For "Lisp jumping between hosts", the important property is not that every OS uses the same socket API. It is that the same Lisp program does not silently inherit different host defaults.

The bind-address choice is now part of the Lisp layer rather than a Rust/Linux accident. A future Windows, WASM-like, embedded, or FPGA/SoC host only needs to implement the explicit bind mechanism when TCP listening exists.

## Principle

> Do not make the host richer to make it look raw. Make the boundary only as expressive as the language has evidence that it needs.

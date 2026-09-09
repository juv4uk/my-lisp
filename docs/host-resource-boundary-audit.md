# Аудит host-resource boundary / Host-resource boundary audit

Status: focused issue #27 decision with executable evidence.

## Питання

Core `Value` досі зберігає TCP handles як concrete Rust payloads:

```text
TcpConnection(Rc<RefCell<TcpStream>>)
TcpListener(Rc<TcpListener>)
```

Це реальна representation coupling. Питання аудиту вужче: чи ця coupling уже переносить OS-поведінку/семантичну політику в core, і чи opaque migration зараз дає доведену користь.

## Поточна межа ownership

| Resource / boundary | Хто створює concrete object | Хто зберігає handle | Хто інтерпретує concrete type | OS operations | Observable language meaning |
|---|---|---|---|---|---|
| `TcpConnection` | `my-lisp-host` (`tcp-connect`, `tcp-accept`) | core `Value` | `my-lisp-host` | host: connect/read/write/shutdown | class `<tcp-connection>`, pointer identity, capability behavior |
| `TcpListener` | `my-lisp-host` (`tcp-listen-raw`) | core `Value` | `my-lisp-host` | host: bind/accept | class `<tcp-listener>`, pointer identity, capability behavior |
| filesystem | `my-lisp-host` | ordinary Lisp values only | `my-lisp-host` | read/write/read-dir | bytes/text/result values; no live resource handle |
| process | `my-lisp-host` | ordinary Lisp values only | `my-lisp-host` | spawn/wait/capture | result data; no live process handle |

Core-side TCP branches do not call socket APIs. They only:

- retain the live payload in `Value`;
- compare handles by `Rc` identity;
- render the resource class, never endpoint/fd/socket metadata;
- tag the handle pointer in the current layout bridge;
- reject conversion of a live resource back into executable source data.

FASL snapshots serialize parser `Expr` trees, not live runtime TCP resources, so FASL does not depend on `TcpStream`/`TcpListener` representation.

## Executable evidence

`crates/my-lisp/tests/host_resource_boundary.rs` is a source-level boundary guard:

1. concrete `std::net` TCP types may occur in core only in `value.rs` storage;
2. socket operations such as connect/bind/accept/shutdown/write are forbidden in core source;
3. a bare core process installs no TCP capabilities.

`crates/my-lisp-host/tests/tcp_handle_semantics.rs` pins the language-observable handle contract against real sockets:

```text
same listener handle      -> eq = t
different live listeners  -> eq = ()
listener display          -> <tcp-listener>
connection after close    -> same identity, <tcp-connection>
```

Existing `crates/my-lisp-host/tests/tcp.rs` remains the broader real-socket proof for connect/listen/accept/read/write/close behavior and named errors. Existing WASM CI is the portability gate that catches an accidental native-socket build dependency in targets that currently compile the core.

## Falsification experiment: opaque payload

The focused experiment does **not** change production representation. In test-only code it replaces the concrete payload shape with:

```text
Rc<dyn Any>
```

and stores both:

```text
TcpListener
RefCell<TcpStream>
```

behind that opaque carrier. The host test then recovers the concrete resource locally with `downcast_ref`, performs real:

```text
accept -> read -> write -> shutdown
```

and proves that `Rc::ptr_eq` still preserves resource identity.

Result:

> The current TCP mechanism does not require the core to know the concrete `std::net` payload type in order to preserve today's observable class, identity, lifecycle and transport behavior.

So an opaque migration is technically feasible.

## Decision

Classification:

```text
confirmed-legitimate-host-mechanism
```

with an explicit qualification:

```text
representation coupling exists;
semantic/operation coupling is not evidenced.
```

We intentionally **do not migrate production `Value` to `Rc<dyn Any>` in issue #27**.

Reason: the experiment proves feasibility, not necessity. A production opaque carrier would add a generic host-resource abstraction plus runtime downcasts while changing no current language behavior, named error, authorization boundary, WASM target result, or host mechanism. Under the project rule “do not expand the substrate without evidence”, that is abstraction work without a demonstrated semantic or portability payoff.

This decision is stronger than saying “leave it because refactoring is hard”: the alternative was constructed and exercised. It is retained as a known migration path, but not promoted into runtime ontology merely because it works.

## Migration triggers

Re-open the representation decision if at least one concrete trigger appears:

1. a supported target cannot compile the core because `std::net` types occur in `Value`;
2. an embedder needs to supply a host-defined live resource class without changing core source;
3. a third/fourth live host resource makes per-resource concrete variants repeat the same storage/identity machinery;
4. FFI or another runtime boundary requires resource payloads to be type-erased behind a stable host-owned ABI;
5. executable evidence shows concrete payload knowledge changing language semantics or authorization.

At that point the test-only `Rc<dyn Any>` experiment is a concrete starting point, not a speculative design.

## Ownership consequence

The precise ownership statement for the broader self-growth map is:

```text
TCP authorization          = host-authorization
TCP socket effects         = host-mechanism
TCP live-handle storage    = host-mechanism, confirmed legitimate for current targets
TCP text/protocol meaning  = Lisp-owned
```

The important distinction is **meaning vs mechanism**, not Rust LOC vs Lisp LOC.

## Non-claims

This audit does not claim:

- that concrete `std::net` storage is the final ideal representation;
- that all future host resources should become TCP-shaped variants;
- that `Rc<dyn Any>` is already the chosen production design;
- that socket failure text is fully portable (that is a separate structured-error concern);
- that current layout tags are a final memory ABI.

## Stop condition

No production representation change is justified until one of the migration triggers has executable evidence. The source guard and handle-semantic tests should fail first if the current narrow boundary begins to widen.

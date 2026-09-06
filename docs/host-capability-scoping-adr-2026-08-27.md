# ADR: Scoped host capabilities (filesystem/network) — 2026-08-27

**Статус:** PARTIALLY IMPLEMENTED / EMBEDDING MECHANISM CONFIRMED — 2026-09-07.

The original design was owner-ratifiable and began as PROPOSED. The repository-wide
repair pass implemented the narrow, backward-compatible embedding mechanism without
changing the trusted native default:

- `Environment::with_fs_read_roots(...)`;
- `Environment::with_fs_write_roots(...)`;
- `Environment::with_tcp_connect_allowlist(...)`;
- `Environment::with_tcp_listen_allowlist(...)`;
- `None` for any dimension remains unrestricted;
- lexical child environments share the same session policy;
- filesystem canonicalization/enforcement stays in `my-lisp-host`, not the semantic core;
- connect and listen remain independent policy dimensions.

Executable evidence: `crates/my-lisp-host/tests/capability_scoping.rs`, workspace
CI #1038 (tests/build + zero-warning clippy).

**Not yet ratified as a public CLI contract:** the originally proposed
`--allow-fs-read`, `--allow-fs-write`, `--allow-tcp-connect`, and
`--allow-tcp-listen` flag surface has not been added. The confirmed claim is the
programmatic per-session embedding boundary, not that every CLI entry point is now
sandboxed.

**Historical process decision, unchanged:** unrestricted process execution is the
trusted native Lisp-machine profile. Exact `process` allowlists remain available for
restricted embeddings/TCP-oracle sessions.

---

## 1. Current implementation

`crates/my-lisp/src/environment.rs` carries policy data only. The semantic core still
does not perform filesystem or network operations and `Environment::root()` does not
install host capabilities.

```text
Environment session policy
├── process_allowlist
├── fs_read_roots
├── fs_write_roots
├── tcp_connect_allowlist
└── tcp_listen_allowlist

None          → unrestricted trusted profile
Some([])      → deny all for that dimension
Some(entries) → explicit allow scope
```

`crates/my-lisp-host/src/lib.rs` owns enforcement:

- `read-file`, `read-file-bytes`, `read-dir`, and `load` use the read roots;
- `write-file` and `write-file-bytes` use the write roots;
- `tcp-connect` checks the connect allowlist before opening a socket;
- `tcp-listen-raw` checks the listen allowlist before binding;
- an already-created TCP handle is not re-authorized on each read/write/accept/close.

This keeps the boundary:

```text
core                    host
policy data     →       canonicalize / bind / connect / filesystem I/O
(no OS access)          (OS mechanism + enforcement)
```

## 2. Filesystem rule

A filesystem operation is permitted only when the canonical target lies below one
of the configured canonical roots.

For reads, the target must already exist and is canonicalized directly.

For writes:

- an existing target is canonicalized directly;
- for a new file, its existing parent directory is canonicalized and the final file
  name is joined afterward.

This rejects `..`/component escapes and symlink escapes without requiring the core to
touch the filesystem. A write into a parent directory that itself does not exist is
not authorized by this check; `write-file` does not create parent directories anyway.

The regression suite includes a Unix symlink inside an allowed root that points to an
outside file; the read is denied before contents are returned.

## 3. Network rule

Connect and listen are independent because they represent different authority.
Each entry is currently stored as:

```rust
(host_or_bind_address, first_port, last_port)
```

with an inclusive port range. Matching is exact on the supplied host/address string;
DNS identity expansion is deliberately not claimed.

Tests prove deny-all is enforced **before** network access/bind and that listen can be
allowed while a connect outside its independent port range remains denied.

## 4. Named denial

Scoped denial is an `ErrorKind::InvalidForm` host-boundary failure with a stable
English marker:

```text
outside this session's capability scope
```

The message also carries Ukrainian/German explanatory text. This does not create a
new language error category or a new semantic primitive.

## 5. Backward compatibility

No scope configured means the same trusted behavior as before this implementation.
Existing native CLI/REPL code therefore does not silently become sandboxed.

This matters: capability scoping is opt-in embedding policy, not a redefinition of
`read-file`, `write-file`, or TCP semantics.

## 6. Executable adversarial evidence

`capability_scoping.rs` currently attempts to break the boundary through:

1. unrestricted-default compatibility;
2. allowed read vs outside read;
3. allowed new write vs outside write (and verifies denial creates no file);
4. `load` as a potential read-scope bypass;
5. Unix symlink escape;
6. TCP connect deny-all before connection attempt;
7. TCP listen deny-all before bind;
8. independent connect/listen policies.

CI #1038 passed workspace tests/build and `cargo clippy --workspace --all-targets -- -D warnings`.

## 7. Remaining migration gate

The embedding API is confirmed. What remains is a user-facing/operational decision,
not an unimplemented security primitive:

- whether the native CLI should expose the four originally proposed scope flags;
- whether those flags should constrain only the local execution session, TCP/oracle
  sessions, or both;
- how the CLI should encode hostnames/IPv6/port ranges without ambiguous parsing;
- whether unauthenticated TCP/oracle should move from current process-only explicit
  restriction toward a default deny policy for filesystem/network too.

Those choices can now be made on top of tested enforcement rather than designing and
shipping the policy simultaneously.

## 8. Epistemic status

```text
core/host capability separation                 confirmed
process per-session allowlist                    confirmed
filesystem per-session read/write scopes         confirmed (tested boundary)
load obeys filesystem read scope                 confirmed
tcp connect/listen independent scopes            confirmed (tested boundary)
symlink escape prevention                        confirmed on Unix regression
trusted default unchanged                        confirmed
public CLI flags                                 not implemented
complete sandbox against every OS namespace      NOT claimed
```

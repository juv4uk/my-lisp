# Host error portability audit / Аудит переносимості host-помилок

Status: architectural audit. No new error kind is introduced here.

## Why this matters

The host substrate is becoming smaller and more portable, but failure reporting still leaks platform-specific text. For example, TCP mechanisms currently map operating-system failures to `LanguageError { kind: InvalidForm, message, span }` and append the OS error string to `message`.

That means two hosts can expose the same mechanism and still produce different human text for the same class of failure.

For agents and cross-repository coordination, that is a weak boundary if consumers start pattern-matching on prose.

## Current evidence

The core error type already separates:

```text
kind
message
span
```

and derives a broader classification from `kind`.

This is useful, but all current TCP transport failures such as connect/bind/accept/read/write/close collapse into `ErrorKind::InvalidForm`. The only operation identity then lives inside the message string.

A concrete consumer already demonstrates the problem: the TCP integration retry helper identifies a retryable connect failure with a string check for `"tcp-connect:"`.

```text
agent/test decision
      ↓
message text inspection
      ↓
platform-specific diagnostic tail
```

The prefix is currently stable by convention, but this is not yet a machine-readable failure identity.

## Do not solve this by exploding ErrorKind

Adding `TcpConnectFailed`, `TcpBindFailed`, `TcpReadFailed`, and similar variants directly to the language's contractual `ErrorKind` would couple the core language contract to one host technology.

That would move in the wrong direction for FPGA/WASM/alternate hosts.

The desirable separation is:

```text
language failure category
        +
host mechanism identity
        +
mechanism failure identity
        +
optional human/OS detail
```

The exact representation is not ratified by this audit.

## Candidate shape

A future host failure could conceptually carry data equivalent to:

```text
(host-failure
  (operation tcp-connect)
  (reason connect-failed)
  (detail "...platform diagnostic..."))
```

or the same information in structured runtime fields.

The important property is not syntax. It is that an agent can decide from stable data such as:

```text
operation = tcp-connect
reason = connect-failed
```

without parsing Windows/Linux/Rust prose.

## Evidence gate before changing the error model

1. Keep current `ErrorKind` semantics unchanged unless there is independent reason to version the language contract.
2. Define the smallest host-failure identity needed by at least two real mechanisms or consumers.
3. Preserve the OS diagnostic only as optional human detail, never as the semantic discriminator.
4. Replace message-string branching in tests/agents with structured branching.
5. Prove that two deliberately different underlying OS errors can still expose the same stable mechanism-level failure identity when they mean the same thing.
6. Keep workspace tests/build/clippy green before widening the pattern to filesystem/process/time.

## Immediate conclusion

Do not rewrite the whole error system now.

The next justified experiment is narrow: give TCP connect/bind failures a stable machine-readable mechanism identity without changing public Lisp semantics, then migrate the existing retry helper away from prose inspection.

If that experiment remains small and useful, generalize it into the portable host substrate contract. If it requires broad core churn, stop and reassess.

## Principle

> Agents should branch on meaning, not on operating-system prose.

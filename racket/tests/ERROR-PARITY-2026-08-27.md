# Racket error-fixture parity — 2026-08-27

Status: EXECUTED, divergences remain visible as failing conformance checks.

Command:

```sh
timeout 60 nice -n 10 guix shell racket -- raco test racket/tests/conformance.rkt
```

The fixture file currently contains 34 `error` cases (the task description's
32 was stale). All 34 are now read and evaluated by the Racket port. Seventeen
match the expected contract error class; seventeen diverge:

- Reader/numeric grammar (4): `.` is a host read error instead of
  `UnknownSymbol`; `1e100001`, `1e-100001`, and `(def 1e100001 5)` return
  values instead of `NumericOverflow`.
- Core validation (4): `(lambda (x))`, `(quote a b)`, and
  `(defmacro 5 (x) x)` return values; `(cons 1)` leaks a Racket contract
  exception instead of the my-lisp `Arity` error.
- Missing Racket-port capabilities (9): TCP (2), process (1), byte-file IO
  (2), JSON parse (1), typed-buffer construction (2), and typed-buffer map
  (1) report `UnknownSymbol` rather than their contract error classes.

The run reports `34/234 test failures`: 17 pre-existing value-fixture failures
plus these 17 newly measured error-fixture divergences. This task adds the
measurement gate; it does not claim the Racket port is conformant.

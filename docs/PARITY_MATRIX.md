# Evaluator Parity Matrix

## Goal

Ensure that the native evaluator and the metacircular evaluator agree on language meaning before reducing native semantic ownership.

## Core witnesses

| Area | Native path | Meta evaluator witness | Status |
| --- | --- | --- | --- |
| quote | eval special form | quoted data | verify |
| atom | primitive dispatch | my-apply-primitive | verify |
| cons | primitive dispatch | my-apply-primitive | verify |
| car/cdr | primitive dispatch | my-apply-primitive | verify |
| lambda | closure creation | my-make-closure | verify |
| lexical capture | Environment | captured env data | verify |
| recursion | native closure binding | recursive-closure | verify |
| mutual recursion | native environment | recursive-group-closure | verify |
| macros | macro substrate | macro values | verify |

## Exit condition

A semantic feature can move from native ownership only after:

1. A My-Lisp witness exists.
2. Native and metacircular evaluations produce the same result.
3. Regression tests preserve the contract.

## Next implementation target

Move from documentation parity to executable parity tests.

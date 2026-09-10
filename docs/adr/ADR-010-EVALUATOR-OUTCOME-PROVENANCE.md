# ADR-010: Evaluator Outcome Provenance
# ADR-010: Походження результату обчислення

**Status:** Accepted / Прийнято  
**Date:** 2026-09-10  
**Scope:** `lib/meta-eval.my` internal sequencing and error propagation

## 1. Decision / Рішення

The metacircular evaluator distinguishes an **internal evaluator failure** from an ordinary Lisp value by carrying internal outcomes through a private envelope until the current evaluation step has finished.

Conceptually:

```text
success(value)
failure(error-value)
```

The public `my-eval` / `my-apply` interface remains unchanged: callers still observe ordinary values or the existing data-level `(error kind detail)` representation.

The internal envelope is tagged by private native closure identities. It is deliberately **not** recognized by a public symbol such as `ok`, `fail`, or `error` and not by structural list shape.

## 2. Why provenance is necessary / Навіщо потрібне походження

Before this decision, `my-eval-list` evaluated every argument and simply stored each result as Lisp data. Because evaluator failures are represented publicly as ordinary `(error ...)` data, a failure in an earlier argument could be treated as a successful value while later arguments continued to execute.

Executable witness:

```lisp
((lambda (x y) y)
  missing-first
  ((lambda (z) z)))
```

The reference evaluator stops at `missing-first` with `UnknownSymbol`. The old meta evaluator continued to the second argument, produced `Arity`, bound both error-shaped values as arguments, and returned the later one.

The contract-relevant rule is therefore:

```text
operator first
then arguments left-to-right
stop at the first evaluator failure
```

## 3. Why structural `(error ...)` detection is rejected / Чому не перевіряємо форму `(error ...)`

This is valid user data:

```lisp
(quote (error ordinary data))
```

and so is:

```lisp
(quote (fail ordinary data))
```

A structural predicate such as `my-error?` or a public `(fail value)` tag would confuse legal program data with evaluator control flow.

The regression witnesses therefore require both forms to pass through an application as ordinary values. The evaluator may only short-circuit values that carry the private internal outcome identity created by the evaluator itself.

## 4. Representation / Представлення

`lib/meta-eval.my` uses two private native closure values as outcome identities. Native closures have identity semantics in the reference runtime (`eq` compares closure identity), are atoms, and cannot be manufactured by interpreted source through `quote` or `cons`.

Internal result-aware paths include:

```text
my-eval-result
my-eval-application-result
my-eval-list-result
my-eval-body-result
my-eval-cond-result
my-apply-result
```

Compatibility wrappers (`my-eval`, `my-apply`, `my-eval-list`, `my-eval-body`, `my-eval-cond`) unwrap the outcome and retain the existing external surface.

This representation is mechanism, not new language syntax or a new Canon primitive.

## 5. Observable guarantees / Спостережувані гарантії

For the audited meta-evaluator scope:

1. Operator evaluation happens before ordinary arguments.
2. Ordinary arguments evaluate left-to-right.
3. The first evaluator failure stops later argument evaluation.
4. Sequential closure bodies stop at the first evaluator failure.
5. `cond` test failures propagate instead of being treated as truthy error data.
6. Macro expansion failures remain failures rather than becoming expansion code.
7. Quoted/user-constructed `error`- or `fail`-shaped lists remain ordinary data.

## 6. Non-goals / Не цілі

This ADR does not:

- add exceptions to the language;
- add a Result datatype to user-visible my-lisp;
- change Rust `LanguageError` representation;
- change compiler/CML semantics;
- claim that every possible error detail already has native/meta parity;
- make the human Markdown evidence projection authoritative over `knowledge/meta-eval-evidence.wsm`.

## 7. Acceptance evidence / Доказ прийняття

The decision is accepted only with executable witnesses for both sides of the boundary:

```text
first argument UnknownSymbol vs later Arity -> first error wins
operator UnknownSymbol vs argument failure  -> operator error wins
quoted (error ordinary data)                 -> ordinary value
quoted (fail ordinary data)                  -> ordinary value
```

Evidence lives in `crates/my-lisp/tests/meta_eval_evidence.rs` and `crates/my-lisp/tests/meta_eval_error_provenance.rs`. Once these witnesses pass with the full workspace regression gate, `function-application-order` may be `confirmed` in `knowledge/meta-eval-evidence.wsm`.
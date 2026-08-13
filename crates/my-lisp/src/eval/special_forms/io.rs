//! `print`/`princ`/`write-to-string`, the `read`/`eval`/`read-all`/`load`
//! read-eval-loop primitives, and the one place in the crate that touches
//! real stdin (`read_stdin_line`, behind `(read)` with no arguments).

use super::core::{exact_arity, quoted};
use crate::eval::{closures, evaluate};
use crate::{Environment, ErrorKind, Expr, LanguageError, Span, Value};
use std::rc::Rc;

/// `print` evaluates its one argument and appends its `Display` text to the
/// session-wide output transcript (`Environment::print`) rather than writing
/// to stdout/stderr directly — the crate stays capability-free, and it's the
/// host (`my-lisp-cli`, `my-lisp-wasm`) that decides where `EvalResult.output`
/// actually goes. Returns the evaluated value, so `(print x)` composes like
/// Common Lisp's `print` instead of being a dead end in an expression.
pub(crate) fn evaluate_print(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("print", arguments, 1, span)?;
    let value = evaluate(&arguments[0], environment)?;
    environment.print(value.to_string());
    Ok(value)
}

/// `princ`, the `display`/`princ` half of the classic Lisp print-function
/// pair `print` is the other half of (see `Value::to_princ_string`):
/// strings come out raw, no surrounding quotes or escapes — for output
/// meant for a person or for reassembling as literal text, never meant to
/// be `read` back as the same value.
pub(crate) fn evaluate_princ(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("princ", arguments, 1, span)?;
    let value = evaluate(&arguments[0], environment)?;
    environment.print(value.to_princ_string());
    Ok(value)
}

/// Returns the same canonical, read-back-safe representation used by `print`
/// without touching the output transcript. This is the minimal bridge needed
/// to compose structured Lisp data with `write-file` and `tcp-write`.
pub(crate) fn evaluate_write_to_string(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("write-to-string", arguments, 1, span)?;
    let value = evaluate(&arguments[0], environment)?;
    Ok(Value::String(Rc::from(value.to_string())))
}

/// `read` is McCarthy's original reader primitive: it turns text into one
/// s-expression of *data*, the same way `'expr` does, without evaluating it —
/// `(eval (read "(+ 1 2)"))` is the read/eval loop written out by hand, in
/// the language itself. `(read "...")` (one argument) stays capability-free,
/// same as the rest of the crate — it parses the given string. `(read)`
/// (zero arguments) is the deliberate, explicit exception: it blocks on one
/// line of real stdin via `read_stdin_line`, which is `#[cfg]`-gated to a
/// clear `InvalidForm` error instead of a panic on `wasm32` — the browser
/// REPL (`crates/my-lisp-wasm`) has no console to block on.
pub(crate) fn evaluate_read(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    if arguments.len() > 1 {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "read expects zero or one arguments · read ochikuie nul abo odyn arhument · read erwartet null oder ein Argument",
            span,
        ));
    }
    let source = if let Some(argument) = arguments.first() {
        // `Value` has a custom `Drop` impl (iterative, for stack-safe deep-list
        // drop), which forbids partially moving a field out of a match on it by
        // value — hence matching on a reference and cloning the cheap `Rc<str>`.
        let evaluated = evaluate(argument, environment)?;
        match &evaluated {
            Value::String(text) => text.to_string(),
            _ => {
                return Err(LanguageError::new(
                    ErrorKind::Type,
                    "read expects a string · read ochikuie riadok · read erwartet eine Zeichenkette",
                    argument.span,
                ))
            }
        }
    } else {
        read_stdin_line(span)?
    };
    let expressions = crate::parse(&source).map_err(|mut error| {
        error.span = span;
        error
    })?;
    match <[Expr; 1]>::try_from(expressions) {
        Ok([expression]) => Ok(quoted(&expression)),
        Err(expressions) => Err(LanguageError::new(
            ErrorKind::InvalidForm,
            format!(
                "read expects exactly one expression, found {} · read ochikuie rivno odyn vyraz, znaideno {} · read erwartet genau einen Ausdruck, gefunden {}",
                expressions.len(), expressions.len(), expressions.len()
            ),
            span,
        )),
    }
}

/// Blocks on one line of real stdin. This is the one place in the crate that
/// touches an actual host I/O stream — see `evaluate_read`'s doc comment for
/// why this exception exists and how it's scoped away from `wasm32`.
// Reliable when `my-lisp-cli` runs a *file* (verified: `(eval (read))` in a
// file, piped stdin data, evaluates correctly end to end). Inside the
// interactive REPL, this competes with rustyline for the same stdin — with
// piped/redirected (non-TTY) input, rustyline's own line reading can buffer
// ahead of what it hands back, so a later `(read)` call sees less than a
// real terminal session would. A genuine TTY reads line-by-line in raw mode
// without that over-buffering, so typed-at-a-terminal REPL use is expected
// to behave; piped REPL input is the documented edge case, not a silent gap.
#[cfg(not(target_arch = "wasm32"))]
fn read_stdin_line(span: Span) -> Result<String, LanguageError> {
    use std::io::BufRead;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("read: failed to read from stdin · read: ne vdalos prochytaty stdin · read: Lesen von stdin fehlgeschlagen: {error}"),
            span,
        )
    })?;
    Ok(line.trim_end_matches(['\n', '\r']).to_string())
}

#[cfg(target_arch = "wasm32")]
fn read_stdin_line(span: Span) -> Result<String, LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "read: interactive stdin is not available in this build · read: interaktyvnyi stdin nedostupnyi u tsii zbirtsi · read: interaktives stdin ist in diesem Build nicht verfügbar",
        span,
    ))
}

/// `eval` closes the read/eval loop McCarthy's Lisp is built around:
/// evaluates its argument to get a *datum* (typically from `read` or
/// `quote`), then evaluates that datum as code. Reuses `closures::value_to_expr`,
/// the same data->code conversion macro expansion already relies on, rather
/// than duplicating the cons-cell walk. `Closure`/`Macro` values are
/// self-evaluating (returned unchanged) since there's no source syntax for
/// them to convert back into.
pub(crate) fn evaluate_eval(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("eval", arguments, 1, span)?;
    let datum = evaluate(&arguments[0], environment)?;
    if matches!(datum, Value::Closure(_) | Value::Macro(_)) {
        return Ok(datum);
    }
    let expression = closures::value_to_expr(datum, span)?;
    evaluate(&expression, environment)
}

/// Step 4 of `lib/clips-import.my`: reading a *real* `.clp` file off disk
/// rather than a caller-supplied quoted literal. `load` already reads a
/// file, but evaluates every top-level form it finds — exactly wrong for
/// CLIPS source, whose `defrule`/`=>` forms aren't meaningful my-lisp code
/// to *run*, only to read as data. `read-all` parses text into every
/// top-level form as data, the multi-form counterpart to `read` (which
/// errors unless the string holds exactly one form).
pub(crate) fn evaluate_read_all(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("read-all", arguments, 1, span)?;
    let evaluated = evaluate(&arguments[0], environment)?;
    let Value::String(ref text) = evaluated else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "read-all expects a string · read-all ochikuie riadok · read-all erwartet eine Zeichenkette",
            span,
        ));
    };
    let expressions = crate::parse(text).map_err(|mut error| {
        error.span = span;
        error
    })?;
    Ok(Value::list(expressions.iter().map(quoted)))
}

pub(crate) fn evaluate_load(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("load", arguments, 1, span)?;
    let evaluated = evaluate(&arguments[0], environment)?;
    let Value::String(ref path) = evaluated else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "load expects a string path · load ochikuie riadok-shliakh · load erwartet einen String-Pfad",
            span,
        ));
    };

    let source = super::file_io::read_file(path, span)?;
    let expressions = crate::parse(&source).map_err(|mut error| {
        error.span = span;
        error
    })?;

    let mut last_value = Value::Nil;
    for expr in expressions {
        last_value = evaluate(&expr, environment)?;
    }

    Ok(last_value)
}

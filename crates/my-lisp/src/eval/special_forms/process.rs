//! `process-run` (PLAN.md item 21's follow-up) — deliberately narrow, not a
//! general shell-out primitive: never goes through a shell, and the calling
//! session must have opted into exactly the program's name via
//! `Environment::with_process_allowlist`.

use super::core::exact_arity;
use crate::eval::evaluate;
use crate::{Environment, ErrorKind, Exactness, Expr, LanguageError, Span, Value};
use std::rc::Rc;

/// `(process-run program args)` runs `program` with `args` (a list of
/// strings) and returns `(list exit-code stdout stderr)`.
/// `std::process::Command::new(program).args(args)` never goes through a
/// shell (no `sh -c`, no string interpolation, no injection surface via
/// `;`/`&&`/backticks in an argument), and the default session
/// (`Environment::root()`) always fails this named, never silently — see
/// `Environment::with_process_allowlist`'s own comment for why: combined
/// with `tcp-accept`'s inbound networking, an unrestricted `process-run`
/// would let a remote peer reach arbitrary command execution through a
/// my-lisp program.
pub(crate) fn evaluate_process_run(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("process-run", arguments, 2, span)?;
    let program_value = evaluate(&arguments[0], environment)?;
    let Value::String(ref program) = program_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "process-run expects a string program name · process-run ochikuie riadok-imia prohramy · process-run erwartet einen String-Programmnamen",
            arguments[0].span,
        ));
    };
    if !environment.is_process_allowed(program) {
        return Err(LanguageError::new(
            ErrorKind::InvalidForm,
            format!("process-run: {program} is not on this session's allowlist · process-run: {program} nemaie v allowlist tsiiei sesii · process-run: {program} steht nicht auf der Allowlist dieser Sitzung"),
            span,
        ));
    }
    let args_value = evaluate(&arguments[1], environment)?;
    let args = expect_string_list(&args_value, arguments[1].span)?;
    let output = process_run(program, &args, span)?;
    Ok(Value::list([
        Value::Number(output.status.code().unwrap_or(-1) as f64, Exactness::Exact),
        Value::String(Rc::from(String::from_utf8_lossy(&output.stdout).as_ref())),
        Value::String(Rc::from(String::from_utf8_lossy(&output.stderr).as_ref())),
    ]))
}

fn expect_string_list(value: &Value, span: Span) -> Result<Vec<String>, LanguageError> {
    let mut items = Vec::new();
    let mut current = value;
    loop {
        match current {
            Value::Nil => return Ok(items),
            Value::Pair(head, tail) => {
                let Value::String(ref text) = **head else {
                    return Err(LanguageError::new(
                        ErrorKind::Type,
                        "process-run expects a list of strings for its second argument · process-run ochikuie spysok riadkiv druhym arhumentom · process-run erwartet eine Liste von Zeichenketten als zweites Argument",
                        span,
                    ));
                };
                items.push(text.to_string());
                current = tail;
            }
            _ => {
                return Err(LanguageError::new(
                    ErrorKind::Type,
                    "process-run expects a proper list of strings for its second argument · process-run ochikuie pravylnyi spysok riadkiv druhym arhumentom · process-run erwartet eine echte Liste von Zeichenketten als zweites Argument",
                    span,
                ))
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn process_run(program: &str, args: &[String], span: Span) -> Result<std::process::Output, LanguageError> {
    std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|error| {
            LanguageError::new(
                ErrorKind::InvalidForm,
                format!("process-run: failed to run {program}: {error}"),
                span,
            )
        })
}

#[cfg(target_arch = "wasm32")]
fn process_run(_program: &str, _args: &[String], span: Span) -> Result<std::process::Output, LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "process-run: process execution is not available in this build",
        span,
    ))
}

use my_lisp::{eval_expr, exact_arity, Environment, ErrorKind, Exactness, Expr, LanguageError, Span, Value};
use std::rc::Rc;

fn bytes_to_value(bytes: &[u8]) -> Value {
    Value::list(
        bytes
            .iter()
            .map(|byte| Value::Number(*byte as f64, Exactness::Exact)),
    )
}

/// `(process-run-raw program args)` runs one host process and returns the
/// transport observation without interpreting stdout/stderr as text:
///
///   (process-result exit-code-or-() stdout-bytes stderr-bytes)
///
/// Process selection and allowlisting remain host capability policy. Text
/// decoding belongs to the language layer (`lib/utf8.my`).
pub(super) fn evaluate_process_run_raw(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("process-run-raw", arguments, 2, span)?;
    let program_value = eval_expr(&arguments[0], environment)?;
    let Value::String(ref program) = program_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "process-run-raw expects a string program name · process-run-raw ochikuie riadok-imia prohramy · process-run-raw erwartet einen String-Programmnamen",
            arguments[0].span,
        ));
    };
    if !environment.is_process_allowed(program) {
        return Err(LanguageError::new(
            ErrorKind::InvalidForm,
            format!("process-run-raw: {program} is not on this session's allowlist · process-run-raw: {program} nemaie v allowlist tsiiei sesii · process-run-raw: {program} steht nicht auf der Allowlist dieser Sitzung"),
            span,
        ));
    }

    let args_value = eval_expr(&arguments[1], environment)?;
    let args = super::expect_string_list(&args_value, arguments[1].span)?;
    let output = super::process_run(program, &args, span)?;

    let exit_code = output
        .status
        .code()
        .map(|code| Value::Number(code as f64, Exactness::Exact))
        .unwrap_or(Value::Nil);

    Ok(Value::list([
        Value::Symbol(Rc::from("process-result")),
        exit_code,
        bytes_to_value(&output.stdout),
        bytes_to_value(&output.stderr),
    ]))
}

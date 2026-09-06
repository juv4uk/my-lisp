use super::core::exact_arity;
use crate::eval::evaluate;
use crate::{Environment, ErrorKind, Exactness, Expr, LanguageError, Span, Value};
use std::rc::Rc;

/// Minimal runtime bridge from an exact Unicode scalar value to the language's
/// immutable string representation. Unicode/UTF-8 decoding policy stays in
/// Lisp; this function only materializes one already-interpreted scalar.
pub(crate) fn evaluate_codepoint_to_string(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("codepoint->string", arguments, 1, span)?;
    let value = evaluate(&arguments[0], environment)?;
    let scalar = exact_scalar_value(&value, span)?;
    let character = char::from_u32(scalar).ok_or_else(|| {
        LanguageError::new(
            ErrorKind::Type,
            "codepoint->string expects a Unicode scalar value · codepoint->string ochikuie skaliarne znachennia Unicode · codepoint->string erwartet einen Unicode-Skalarwert",
            span,
        )
    })?;
    Ok(Value::String(Rc::from(character.to_string().as_str())))
}

fn exact_scalar_value(value: &Value, span: Span) -> Result<u32, LanguageError> {
    let integer = match value {
        Value::Number(number, Exactness::Exact)
            if number.is_finite() && number.fract() == 0.0 =>
        {
            if *number < 0.0 || *number > 0x10ffff as f64 {
                return Err(invalid_scalar(span));
            }
            *number as u32
        }
        Value::Rational(rational) if rational.is_integer() => {
            let number = rational.as_precise_i64().ok_or_else(|| invalid_scalar(span))?;
            u32::try_from(number).map_err(|_| invalid_scalar(span))?
        }
        _ => return Err(invalid_scalar(span)),
    };

    if integer > 0x10ffff || (0xd800..=0xdfff).contains(&integer) {
        return Err(invalid_scalar(span));
    }
    Ok(integer)
}

fn invalid_scalar(span: Span) -> LanguageError {
    LanguageError::new(
        ErrorKind::Type,
        "codepoint->string expects an exact Unicode scalar integer (0..0x10FFFF excluding surrogates) · codepoint->string ochikuie tochne tsile skaliarne znachennia Unicode · codepoint->string erwartet eine exakte Unicode-Skalarzahl",
        span,
    )
}

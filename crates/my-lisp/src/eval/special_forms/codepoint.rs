use super::core::exact_arity;
use crate::eval::evaluate;
use crate::{Environment, ErrorKind, Exactness, Expr, LanguageError, Span, Value};
use std::rc::Rc;

pub(crate) fn codepoint_to_string_values(
    arguments: &[Value],
    span: Span,
) -> Result<Value, LanguageError> {
    if arguments.len() != 1 {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "codepoint->string expects exactly 1 argument(s)",
            span,
        ));
    }
    let scalar = exact_scalar_value(&arguments[0], span)?;
    let character = char::from_u32(scalar).ok_or_else(|| {
        LanguageError::new(
            ErrorKind::Type,
            "codepoint->string expects a Unicode scalar value · codepoint->string ochikuie skaliarne znachennia Unicode · codepoint->string erwartet einen Unicode-Skalarwert",
            span,
        )
    })?;
    Ok(Value::String(Rc::from(character.to_string().as_str())))
}

pub(crate) fn string_to_codepoint_values(
    arguments: &[Value],
    span: Span,
) -> Result<Value, LanguageError> {
    if arguments.len() != 1 {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "string->codepoint expects exactly 1 argument(s)",
            span,
        ));
    }
    let Value::String(text) = &arguments[0] else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "string->codepoint expects a one-character string · string->codepoint ochikuie riadok z odnoho symvolu · string->codepoint erwartet eine Zeichenkette mit genau einem Zeichen",
            span,
        ));
    };

    let mut characters = text.chars();
    let Some(character) = characters.next() else {
        return Err(invalid_character_string(span));
    };
    if characters.next().is_some() {
        return Err(invalid_character_string(span));
    }

    Ok(Value::Number(character as u32 as f64, Exactness::Exact))
}

// Transitional Expr wrappers retained until eval/mod.rs forgets these names.
pub(crate) fn evaluate_codepoint_to_string(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("codepoint->string", arguments, 1, span)?;
    let values = [evaluate(&arguments[0], environment)?];
    codepoint_to_string_values(&values, span)
}

pub(crate) fn evaluate_string_to_codepoint(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string->codepoint", arguments, 1, span)?;
    let values = [evaluate(&arguments[0], environment)?];
    string_to_codepoint_values(&values, span)
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

fn invalid_character_string(span: Span) -> LanguageError {
    LanguageError::new(
        ErrorKind::Type,
        "string->codepoint expects exactly one Unicode scalar character · string->codepoint ochikuie rivno odyn skaliarnyi symvol Unicode · string->codepoint erwartet genau ein Unicode-Skalarzeichen",
        span,
    )
}

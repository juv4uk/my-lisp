//! String/symbol mechanisms that genuinely need the Rust substrate.
//!
//! Public callable identity does not belong here.  The value-level helpers in
//! this module are deliberately independent of evaluator syntax dispatch so
//! they can be installed as ordinary first-class `Value::Builtin` values.
//! Transitional Expr wrappers remain only until the evaluator-name migration
//! is complete.

use super::core::exact_arity;
use crate::eval::evaluate;
use crate::{Environment, ErrorKind, Expr, LanguageError, Span, Value};
use std::rc::Rc;

fn exact_value_arity(
    name: &'static str,
    arguments: &[Value],
    expected: usize,
    span: Span,
) -> Result<(), LanguageError> {
    if arguments.len() != expected {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            format!("{name} expects exactly {expected} argument(s)"),
            span,
        ));
    }
    Ok(())
}

/// Return the half-open character-indexed slice of a string.
///
/// Indices count Unicode scalar values, matching `string-first` and
/// `string-rest`, rather than UTF-8 bytes. Bounds are clamped to the string
/// length; an inverted or empty range returns the empty string.
pub(crate) fn evaluate_string_slice(
    arguments: &[Value],
    span: Span,
) -> Result<Value, LanguageError> {
    if arguments.len() != 3 {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "string-slice expects a string, start, and end · string-slice ochikuie riadok, pochatok i kinets · string-slice erwartet Zeichenkette, Anfang und Ende",
            span,
        ));
    }
    let Value::String(text) = &arguments[0] else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "string-slice expects a string as its first argument · string-slice ochikuie riadok pershym arhumentom · string-slice erwartet eine Zeichenkette als erstes Argument",
            span,
        ));
    };
    let start = slice_index(&arguments[1], span)?;
    let end = slice_index(&arguments[2], span)?;
    let result = crate::string_slice_text(text, start, end);
    Ok(Value::String(Rc::from(result.as_str())))
}

fn slice_index(value: &Value, span: Span) -> Result<usize, LanguageError> {
    let integer = match value {
        Value::Number(number, crate::Exactness::Exact)
            if number.is_finite() && number.fract() == 0.0 =>
        {
            if *number < 0.0 {
                return Err(LanguageError::new(
                    ErrorKind::Type,
                    "string-slice indices must be non-negative exact integers · indeksy string-slice maiut buty nevidiemni tochnymy tsilymy · string-slice-Indizes müssen nichtnegative exakte Ganzzahlen sein",
                    span,
                ));
            }
            (*number as u128).try_into().map_err(|_| {
                LanguageError::new(
                    ErrorKind::NumericOverflow,
                    "string-slice index is too large · indeks string-slice zavelykyi · string-slice-Index ist zu groß",
                    span,
                )
            })?
        }
        Value::Rational(rational) if rational.is_integer() => {
            let number = rational.as_precise_i64().ok_or_else(|| {
                LanguageError::new(
                    ErrorKind::NumericOverflow,
                    "string-slice index is too large · indeks string-slice zavelykyi · string-slice-Index ist zu groß",
                    span,
                )
            })?;
            usize::try_from(number).map_err(|_| {
                LanguageError::new(
                    ErrorKind::Type,
                    "string-slice indices must be non-negative exact integers · indeksy string-slice maiut buty nevidiemni tochnymy tsilymy · string-slice-Indizes müssen nichtnegative exakte Ganzzahlen sein",
                    span,
                )
            })?
        }
        _ => {
            return Err(LanguageError::new(
                ErrorKind::Type,
                "string-slice indices must be non-negative exact integers · indeksy string-slice maiut buty nevidiemni tochnymy tsilymy · string-slice-Indizes müssen nichtnegative exakte Ganzzahlen sein",
                span,
            ));
        }
    };
    Ok(integer)
}

pub(crate) fn string_append_values(
    arguments: &[Value],
    span: Span,
) -> Result<Value, LanguageError> {
    exact_value_arity("string-append", arguments, 2, span)?;
    let Value::String(left) = &arguments[0] else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "string-append expects two strings · string-append ochikuie dva riadky · string-append erwartet zwei Zeichenketten",
            span,
        ));
    };
    let Value::String(right) = &arguments[1] else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "string-append expects two strings · string-append ochikuie dva riadky · string-append erwartet zwei Zeichenketten",
            span,
        ));
    };
    Ok(Value::String(Rc::from(format!("{left}{right}").as_str())))
}

pub(crate) fn string_less_than_values(
    arguments: &[Value],
    span: Span,
) -> Result<Value, LanguageError> {
    exact_value_arity("string<?", arguments, 2, span)?;
    let Value::String(left) = &arguments[0] else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "string<? expects two strings · string<? ochikuie dva riadky · string<? erwartet zwei Zeichenketten",
            span,
        ));
    };
    let Value::String(right) = &arguments[1] else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "string<? expects two strings · string<? ochikuie dva riadky · string<? erwartet zwei Zeichenketten",
            span,
        ));
    };
    Ok(Value::truth(left.as_ref() < right.as_ref()))
}

pub(crate) fn string_predicate_values(
    arguments: &[Value],
    span: Span,
) -> Result<Value, LanguageError> {
    exact_value_arity("string?", arguments, 1, span)?;
    Ok(Value::truth(matches!(arguments[0], Value::String(_))))
}

pub(crate) fn symbol_to_string_values(
    arguments: &[Value],
    span: Span,
) -> Result<Value, LanguageError> {
    exact_value_arity("symbol->string", arguments, 1, span)?;
    match &arguments[0] {
        Value::Symbol(symbol) => Ok(Value::String(symbol.clone())),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "symbol->string expects a symbol · symbol->string ochikuie symvol · symbol->string erwartet ein Symbol",
            span,
        )),
    }
}

pub(crate) fn string_to_symbol_values(
    arguments: &[Value],
    span: Span,
) -> Result<Value, LanguageError> {
    exact_value_arity("string->symbol", arguments, 1, span)?;
    match &arguments[0] {
        Value::String(text) => Ok(Value::Symbol(text.clone())),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "string->symbol expects a string · string->symbol ochikuie riadok · string->symbol erwartet eine Zeichenkette",
            span,
        )),
    }
}

pub(crate) fn string_first_values(
    arguments: &[Value],
    span: Span,
) -> Result<Value, LanguageError> {
    exact_value_arity("string-first", arguments, 1, span)?;
    match &arguments[0] {
        Value::String(text) => match text.chars().next() {
            Some(character) => Ok(Value::String(Rc::from(character.to_string().as_str()))),
            None => Err(LanguageError::new(
                ErrorKind::Type,
                "string-first expects a non-empty string · string-first ochikuie neporozhnii riadok · string-first erwartet eine nicht leere Zeichenkette",
                span,
            )),
        },
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "string-first expects a string · string-first ochikuie riadok · string-first erwartet eine Zeichenkette",
            span,
        )),
    }
}

pub(crate) fn string_rest_values(
    arguments: &[Value],
    span: Span,
) -> Result<Value, LanguageError> {
    exact_value_arity("string-rest", arguments, 1, span)?;
    match &arguments[0] {
        Value::String(text) => {
            let mut characters = text.chars();
            if characters.next().is_none() {
                return Err(LanguageError::new(
                    ErrorKind::Type,
                    "string-rest expects a non-empty string · string-rest ochikuie neporozhnii riadok · string-rest erwartet eine nicht leere Zeichenkette",
                    span,
                ));
            }
            Ok(Value::String(Rc::from(characters.as_str())))
        }
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "string-rest expects a string · string-rest ochikuie riadok · string-rest erwartet eine Zeichenkette",
            span,
        )),
    }
}

// Transitional Expr wrappers.  Delete these once `eval/mod.rs` no longer
// dispatches these names specially.
pub(crate) fn evaluate_string_append(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string-append", arguments, 2, span)?;
    let values = [
        evaluate(&arguments[0], environment)?,
        evaluate(&arguments[1], environment)?,
    ];
    string_append_values(&values, span)
}

pub(crate) fn evaluate_string_less_than(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string<?", arguments, 2, span)?;
    let values = [
        evaluate(&arguments[0], environment)?,
        evaluate(&arguments[1], environment)?,
    ];
    string_less_than_values(&values, span)
}

pub(crate) fn evaluate_string_predicate(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string?", arguments, 1, span)?;
    let values = [evaluate(&arguments[0], environment)?];
    string_predicate_values(&values, span)
}

pub(crate) fn evaluate_symbol_to_string(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("symbol->string", arguments, 1, span)?;
    let values = [evaluate(&arguments[0], environment)?];
    symbol_to_string_values(&values, span)
}

pub(crate) fn evaluate_string_to_symbol(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string->symbol", arguments, 1, span)?;
    let values = [evaluate(&arguments[0], environment)?];
    string_to_symbol_values(&values, span)
}

pub(crate) fn evaluate_string_first(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string-first", arguments, 1, span)?;
    let values = [evaluate(&arguments[0], environment)?];
    string_first_values(&values, span)
}

pub(crate) fn evaluate_string_rest(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string-rest", arguments, 1, span)?;
    let values = [evaluate(&arguments[0], environment)?];
    string_rest_values(&values, span)
}

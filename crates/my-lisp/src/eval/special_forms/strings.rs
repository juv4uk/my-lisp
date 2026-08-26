//! String/symbol primitives that genuinely need a Rust primitive
//! (`string-append`, `string<?`, `symbol->string`, `string->symbol`) or
//! were added deliberately for `lib/clips-import.my`'s introspection needs
//! (`string?`, `string-first`, `string-rest`).

use super::core::exact_arity;
use crate::eval::evaluate;
use crate::{Environment, ErrorKind, Expr, LanguageError, Span, Value};
use std::rc::Rc;

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
    if start >= end {
        return Ok(Value::String(Rc::from("")));
    }
    let result: String = text.chars().skip(start).take(end - start).collect();
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

/// String concatenation (PLAN.md item 14) — genuinely needs a Rust
/// primitive, unlike `string-length`/`string-contains?` (both now in
/// `lib/core.my`, expressible via `string-first`/`string-rest`/`eq`
/// alone): `Value::String` wraps an immutable `Rc<str>`, and no
/// existing primitive combines two strings into a new one.
pub(crate) fn evaluate_string_append(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string-append", arguments, 2, span)?;
    let left_value = evaluate(&arguments[0], environment)?;
    let Value::String(ref left) = left_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "string-append expects two strings · string-append ochikuie dva riadky · string-append erwartet zwei Zeichenketten",
            span,
        ));
    };
    let right_value = evaluate(&arguments[1], environment)?;
    let Value::String(ref right) = right_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "string-append expects two strings · string-append ochikuie dva riadky · string-append erwartet zwei Zeichenketten",
            span,
        ));
    };
    Ok(Value::String(Rc::from(format!("{left}{right}").as_str())))
}

/// Lexicographic string ordering (PLAN.md item 15) — the one new Rust
/// primitive the persistent-map design actually needs: Rust's `Ord` for
/// `&str` gives this for free, but nothing in the language could derive
/// "is one string before another" from `string-first`/`string-rest`/`eq`
/// alone (those only ever test *equality* one character at a time, never
/// ordering).
pub(crate) fn evaluate_string_less_than(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string<?", arguments, 2, span)?;
    let left_value = evaluate(&arguments[0], environment)?;
    let Value::String(ref left) = left_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "string<? expects two strings · string<? ochikuie dva riadky · string<? erwartet zwei Zeichenketten",
            span,
        ));
    };
    let right_value = evaluate(&arguments[1], environment)?;
    let Value::String(ref right) = right_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "string<? expects two strings · string<? ochikuie dva riadky · string<? erwartet zwei Zeichenketten",
            span,
        ));
    };
    Ok(Value::Bool(left.as_ref() < right.as_ref()))
}

/// The minimal symbol/string introspection this project held off on for a
/// long time (per CLAUDE.md's "don't grow the Rust surface" principle) —
/// added deliberately when `lib/clips-import.my`'s Step 2 hit a real wall:
/// converting CLIPS's `?x` variable syntax into `(var x)` needs to peel
/// the leading `?` off a symbol's name, and there was no way to inspect a
/// symbol's characters from within my-lisp itself.
pub(crate) fn evaluate_string_predicate(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string?", arguments, 1, span)?;
    Ok(Value::Bool(matches!(
        evaluate(&arguments[0], environment)?,
        Value::String(_)
    )))
}

pub(crate) fn evaluate_symbol_to_string(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("symbol->string", arguments, 1, span)?;
    match evaluate(&arguments[0], environment)? {
        Value::Symbol(ref symbol) => Ok(Value::String(symbol.clone())),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "symbol->string expects a symbol · symbol->string ochikuie symvol · symbol->string erwartet ein Symbol",
            span,
        )),
    }
}

pub(crate) fn evaluate_string_to_symbol(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string->symbol", arguments, 1, span)?;
    match evaluate(&arguments[0], environment)? {
        Value::String(ref text) => Ok(Value::Symbol(text.clone())),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "string->symbol expects a string · string->symbol ochikuie riadok · string->symbol erwartet eine Zeichenkette",
            span,
        )),
    }
}

/// The first character, as a one-character string — the string analogue
/// of `car`. Errors on an empty string, same as `car` on an empty list.
pub(crate) fn evaluate_string_first(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string-first", arguments, 1, span)?;
    match evaluate(&arguments[0], environment)? {
        Value::String(ref text) => match text.chars().next() {
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

/// All but the first character — the string analogue of `cdr`. Errors on
/// an empty string rather than silently returning one, the same way `car`
/// errors on an empty list instead of returning `()`.
pub(crate) fn evaluate_string_rest(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string-rest", arguments, 1, span)?;
    match evaluate(&arguments[0], environment)? {
        Value::String(ref text) => {
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

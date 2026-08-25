//! Exact/inexact numeric handling for `+`, `-`, `*`, and `/`.
//! Obrobka tochnykh/netochnykh chysel dlia `+`, `-`, `*` ta `/`.
//! Verarbeitung exakter/inexakter Zahlen für `+`, `-`, `*` und `/`.


use crate::{Environment, ErrorKind, Exactness, LanguageError, Rational, Span, Value};


// `Rational` wraps a heap-allocated `BigRational` (arbitrary precision), so
// it isn't `Copy` — neither is `Numeric` anymore. Both accessor methods
// below take `&self` and clone on the way out where an owned `Rational` is
// needed, rather than moving out of borrowed slice/vec elements.
// `Rational` ohortaie heap-allocated `BigRational` (dovilna tochnist), tozh
// ne `Copy` — tak samo y `Numeric`. Obydva metody-aktsesory nyzhche berut
// `&self` i klonuiut na vykhodi tam, de potriben vlasnyi `Rational`, zamist
// peremishchennia z pozychenykh elementiv slice/vec.
// `Rational` umschließt ein heap-allokiertes `BigRational` (beliebige
// Genauigkeit), daher ist es nicht `Copy` — `Numeric` auch nicht mehr.
// Beide Zugriffsmethoden unten nehmen `&self` und klonen beim Herausgeben,
// wo ein eigener `Rational` gebraucht wird, statt aus geliehenen
// Slice-/Vec-Elementen herauszubewegen.
#[derive(Clone)]
enum Numeric {
    Exact(Rational),
    Inexact(f64),
}

impl Numeric {
    fn as_f64(&self) -> f64 {
        match self {
            Self::Exact(value) => value.as_f64(),
            Self::Inexact(value) => *value,
        }
    }

    fn to_exact(&self) -> Rational {
        match self {
            Self::Exact(value) => value.clone(),
            Self::Inexact(_) => unreachable!("inexact operands handled before exact arithmetic"),
        }
    }
}

fn numeric_value(value: Value, span: Span) -> Result<Numeric, LanguageError> {
    // Matches on `&value`, not `value`, and clones the `Rational` out:
    // `Value` has a custom `Drop` impl (iterative, for stack-safe deep-list
    // drop — see `value.rs`), which forbids partially moving a field out of
    // a match on it by value.
    // Matchyt na `&value`, ne `value`, i klonuie `Rational`: `Value` maie
    // vlasnyi `Drop` (iteratyvnyi, dlia stack-safe drop hlybokykh spyskiv —
    // dyv. `value.rs`), yakyi zaboroniaie chastkovo peremishchuvaty pole z
    // `match` za znachenniam.
    // Matcht auf `&value`, nicht `value`, und klont das `Rational` heraus:
    // `Value` hat einen eigenen `Drop`-Impl (iterativ, für stack-sicheres
    // Droppen tiefer Listen — siehe `value.rs`), der ein teilweises
    // Herausbewegen eines Feldes aus einem `match` nach Wert verbietet.
    match &value {
        Value::Rational(rational) => Ok(Numeric::Exact(rational.clone())),
        // Reads the tag the reader/arithmetic already set (PLAN.md item 10,
        // Path A) instead of re-guessing exactness from `fract() == 0.0` —
        // an exact `Value::Number` is always integral by construction (see
        // `exact_value` below), so converting straight to `i64` is safe.
        // Chytaie teh, yakyi uzhe vstanovyv reader/aryfmetyka (PLAN.md, punkt
        // 10, shliakh A), zamist toho shchob zanovo vhaduvaty exactness cherez
        // `fract() == 0.0` — tochnyi `Value::Number` zavzhdy tsilyi za
        // pobudovoiu (dyv. `exact_value` nyzhche), tozh priama konversiia v
        // `i64` bezpechna.
        Value::Number(number, Exactness::Exact) => Ok(Numeric::Exact(Rational::integer(*number as i64))),
        Value::Number(number, Exactness::Inexact) => Ok(Numeric::Inexact(*number)),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "arithmetic expects numbers · aryfmetyka ochikuie chysla · Arithmetik erwartet Zahlen",
            span,
        )),
    }
}

pub(crate) fn exact_value(value: Rational) -> Value {
    match value.as_precise_i64() {
        Some(n) => Value::Number(n as f64, Exactness::Exact),
        None => Value::Rational(value),
    }
}

fn arithmetic_overflow(span: Span) -> LanguageError {
    LanguageError::new(
        ErrorKind::NumericOverflow,
        "exact arithmetic overflow · perepovnennia tochnoi aryfmetyky · Überlauf der exakten Arithmetik",
        span,
    )
}

/// Enforces an *opt-in* numeric resource limit (`Environment::with_numeric_bit_limit`)
/// — a no-op when this session never configured one, which is every
/// `conformance.my` fixture and the Rust reference implementation by
/// default (see S1's own open note on arbitrary precision). Checked after
/// computing an exact result, never used to fall back to an inexact
/// approximation — that would violate S1, not satisfy it.
/// Zastosovuie *optsiinu* chyslovu mezhu resursu (`Environment::with_numeric_bit_limit`)
/// — nichoho ne robyt, yakshcho tsia sesiia yii ne nalashtuvala, shcho ye typovym dlia
/// kozhnoi fikstury `conformance.my` y Rust-realizatsii (dyv. vlasnu
/// vidkrytu prymitku S1 pro dovilnu tochnist). Pereviriaietsia pislia
/// obchyslennia tochnoho rezultatu, nikoly ne vykorystovuietsia, shchob
/// vidkotytys do netochnoho nablyzhennia — tse porushylo b S1, ne
/// zadovolnylo b yoho.
fn check_numeric_limit(environment: &Environment, result: &Rational, span: Span) -> Result<(), LanguageError> {
    if let Some(limit) = environment.numeric_bit_limit() {
        if result.bit_length() > limit {
            return Err(LanguageError::new(
                ErrorKind::NumericOverflow,
                "exact arithmetic result exceeds the configured bit-length limit · tochnyi rezultat aryfmetyky perevyshchuie nalashtovanu mezhu v bitakh · exaktes Arithmetikergebnis überschreitet die konfigurierte Bitlängengrenze",
                span,
            ));
        }
    }
    Ok(())
}




fn compare<T: PartialOrd>(operator: &str, left: T, right: T) -> bool {
    match operator {
        "<" => left < right,
        ">" => left > right,
        "=" => left == right,
        _ => unreachable!("known comparison operator"),
    }
}

fn division_error(span: Span) -> LanguageError {
    LanguageError::new(
        ErrorKind::DivisionByZero,
        "division by zero · dilennia na nul · Division durch null",
        span,
    )
}

// ── contract 2.1: value-level entry points (first-class builtins) ──
// The expr-handlers above evaluate arguments then delegate here; the
// builtin closures in eval/builtins.rs call these directly with
// pre-evaluated values. Single compute path, two front doors.

pub(super) fn arithmetic_on_values(
    operator: &str,
    values: &[Value],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    if operator == "-" && values.is_empty() {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "- expects at least one argument · - ochikuie shchonaimenshe odyn arhument · - erwartet mindestens ein Argument",
            span,
        ));
    }
    // ── fast path: all exact integers that fit in i64 ──
    // Avoids BigRational allocation + gcd normalization for the
    // most common case in real workloads (WSM-24 evidence).
    if values.iter().all(|v| matches!(
        v, Value::Number(f, Exactness::Exact)
        if *f == (*f as i64) as f64 && f.abs() <= 9_000_000_000_000.0
    )) {
        let ints: Vec<i64> = values.iter().map(|v| {
            if let Value::Number(f, Exactness::Exact) = v { *f as i64 } else { unreachable!() }
        }).collect();
        let result: Option<i64> = match operator {
            "+" => ints.iter().try_fold(0i64, |acc, &x| acc.checked_add(x)),
            "-" => match ints.len() {
                1 => Some(-ints[0]),
                _ => ints[1..].iter().try_fold(ints[0], |acc, &x| acc.checked_sub(x)),
            },
            "*" => ints.iter().try_fold(1i64, |acc, &x| acc.checked_mul(x)),
            _ => None,
        };
        if let Some(result) = result {
            let exact = Rational::integer(result);
            check_numeric_limit(environment, &exact, span)?;
            return Ok(Value::Number(result as f64, Exactness::Exact));
        }
        // overflow: fall through to bignum path below
    }

    let numerics = values
        .iter()
        .map(|value| numeric_value(value.clone(), span))
        .collect::<Result<Vec<_>, _>>()?;

    if numerics.iter().any(|value| matches!(value, Numeric::Inexact(_))) {
        let floats = numerics.iter().map(Numeric::as_f64).collect::<Vec<_>>();
        let mut result = floats[0];
        for &operand in &floats[1..] {
            result = match operator {
                "+" => result + operand,
                "-" => result - operand,
                "*" => result * operand,
                _ => unreachable!("known arithmetic operator"),
            };
        }
        return Ok(Value::Number(result, Exactness::Inexact));
    }

    let exact = numerics.iter().map(Numeric::to_exact).collect::<Vec<_>>();
    let result = match operator {
        "+" => exact
            .into_iter()
            .try_fold(Rational::integer(0), Rational::checked_add),
        "*" => exact
            .into_iter()
            .try_fold(Rational::integer(1), Rational::checked_mul),
        "-" if exact.len() == 1 => exact[0].clone().checked_neg(),
        "-" => exact[1..]
            .iter()
            .try_fold(exact[0].clone(), |result, value| result.checked_sub(value.clone())),
        _ => unreachable!("known arithmetic operator"),
    }
    .ok_or_else(|| arithmetic_overflow(span))?;
    check_numeric_limit(environment, &result, span)?;
    Ok(exact_value(result))
}

pub(super) fn division_on_values(
    values: &[Value],
    argument_count: usize,
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    if values.is_empty() {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "/ expects at least one argument · / ochikuie shchonaimenshe odyn arhument · / erwartet mindestens ein Argument",
            span,
        ));
    }
    let numerics = values
        .iter()
        .map(|value| numeric_value(value.clone(), span))
        .collect::<Result<Vec<_>, _>>()?;

    let single = argument_count == 1;

    if numerics.iter().any(|value| matches!(value, Numeric::Inexact(_))) {
        let floats = numerics.iter().map(Numeric::as_f64).collect::<Vec<_>>();
        let mut result = floats[0];
        if single {
            if result == 0.0 {
                return Err(division_error(span));
            }
            result = 1.0 / result;
        } else {
            for &divisor in &floats[1..] {
                if divisor == 0.0 {
                    return Err(division_error(span));
                }
                result /= divisor;
            }
        }
        return Ok(Value::Number(result, Exactness::Inexact));
    }

    let exact = numerics.iter().map(Numeric::to_exact).collect::<Vec<_>>();
    let mut result = exact[0].clone();
    if single {
        result = Rational::integer(1)
            .checked_div(result)
            .ok_or_else(|| division_error(span))?;
    } else {
        for divisor in exact.into_iter().skip(1) {
            result = result
                .checked_div(divisor)
                .ok_or_else(|| division_error(span))?;
        }
    }
    check_numeric_limit(environment, &result, span)?;
    Ok(exact_value(result))
}

pub(super) fn comparison_on_values(
    operator: &str,
    values: &[Value],
    span: Span,
) -> Result<Value, LanguageError> {
    if values.is_empty() {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            format!("{operator} expects at least one argument · {operator} ochikuie shchonaimenshe odyn arhument · {operator} erwartet mindestens ein Argument"),
            span,
        ));
    }
    let numerics = values
        .iter()
        .map(|value| numeric_value(value.clone(), span))
        .collect::<Result<Vec<_>, _>>()?;

    let holds = if numerics.iter().any(|value| matches!(value, Numeric::Inexact(_))) {
        numerics
            .windows(2)
            .all(|pair| compare(operator, pair[0].as_f64(), pair[1].as_f64()))
    } else {
        numerics
            .windows(2)
            .all(|pair| compare(operator, pair[0].to_exact(), pair[1].to_exact()))
    };
    Ok(Value::Bool(holds))
}

pub(super) fn order_pair(
    operator: &str,
    left: &Value,
    right: &Value,
    span: Span,
) -> Result<bool, LanguageError> {
    match comparison_on_values(operator, &[left.clone(), right.clone()], span)? {
        Value::Bool(holds) => Ok(holds),
        _ => unreachable!("comparison_on_values returns Bool"),
    }
}

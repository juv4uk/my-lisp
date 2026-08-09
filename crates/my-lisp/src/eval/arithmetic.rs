//! Exact/inexact numeric handling for `+`, `-`, `*`, and `/`.
//! Обробка точних/неточних чисел для `+`, `-`, `*` та `/`.
//! Verarbeitung exakter/inexakter Zahlen für `+`, `-`, `*` und `/`.

use super::evaluate;
use crate::{Environment, ErrorKind, Expr, LanguageError, Rational, Span, Value};

pub(super) fn evaluate_arithmetic(
    operator: &str,
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    if operator == "-" && arguments.is_empty() {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "- expects at least one argument · - очікує щонайменше один аргумент · - erwartet mindestens ein Argument",
            span,
        ));
    }
    let values = arguments
        .iter()
        .map(|argument| numeric_value(evaluate(argument, environment)?, argument.span))
        .collect::<Result<Vec<_>, _>>()?;

    // Exact integers and rationals stay exact. One inexact operand deliberately makes the result inexact.
    // Точні цілі та раціональні лишаються точними. Один неточний операнд навмисно робить результат неточним.
    // Exakte Ganz- und rationale Zahlen bleiben exakt. Ein unexakter Operand macht das Ergebnis bewusst unexakt.
    if values
        .iter()
        .any(|value| matches!(value, Numeric::Inexact(_)))
    {
        let values = values
            .iter()
            .map(|value| value.as_f64())
            .collect::<Vec<_>>();
        let result = match operator {
            "+" => values.iter().sum(),
            "*" => values.iter().product(),
            "-" if values.len() == 1 => -values[0],
            "-" => values[1..]
                .iter()
                .fold(values[0], |result, value| result - value),
            _ => unreachable!("known arithmetic operator"),
        };
        return Ok(Value::Number(result));
    }

    let exact = values
        .iter()
        .map(Numeric::into_exact)
        .collect::<Vec<_>>();
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

// `Rational` wraps a heap-allocated `BigRational` (arbitrary precision), so
// it isn't `Copy` — neither is `Numeric` anymore. Both accessor methods
// below take `&self` and clone on the way out where an owned `Rational` is
// needed, rather than moving out of borrowed slice/vec elements.
// `Rational` огортає heap-allocated `BigRational` (довільна точність), тож
// не `Copy` — так само й `Numeric`. Обидва методи-акцесори нижче беруть
// `&self` і клонують на виході там, де потрібен власний `Rational`, замість
// переміщення з позичених елементів slice/vec.
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

    fn into_exact(&self) -> Rational {
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
    // Матчить на `&value`, не `value`, і клонує `Rational`: `Value` має
    // власний `Drop` (ітеративний, для stack-safe drop глибоких списків —
    // див. `value.rs`), який забороняє частково переміщувати поле з
    // `match` за значенням.
    // Matcht auf `&value`, nicht `value`, und klont das `Rational` heraus:
    // `Value` hat einen eigenen `Drop`-Impl (iterativ, für stack-sicheres
    // Droppen tiefer Listen — siehe `value.rs`), der ein teilweises
    // Herausbewegen eines Feldes aus einem `match` nach Wert verbietet.
    match &value {
        Value::Rational(rational) => Ok(Numeric::Exact(rational.clone())),
        Value::Number(number)
            if number.fract() == 0.0 && *number >= i64::MIN as f64 && *number <= i64::MAX as f64 =>
        {
            Ok(Numeric::Exact(Rational::integer(*number as i64)))
        }
        Value::Number(number) => Ok(Numeric::Inexact(*number)),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "arithmetic expects numbers · арифметика очікує числа · Arithmetik erwartet Zahlen",
            span,
        )),
    }
}

fn exact_value(value: Rational) -> Value {
    match value.as_precise_i64() {
        Some(n) => Value::Number(n as f64),
        None => Value::Rational(value),
    }
}

fn arithmetic_overflow(span: Span) -> LanguageError {
    LanguageError::new(
        ErrorKind::InvalidForm,
        "exact arithmetic overflow · переповнення точної арифметики · Überlauf der exakten Arithmetik",
        span,
    )
}

/// Enforces an *opt-in* numeric resource limit (`Environment::with_numeric_bit_limit`)
/// — a no-op when this session never configured one, which is every
/// `conformance.my` fixture and the Rust reference implementation by
/// default (see S1's own open note on arbitrary precision). Checked after
/// computing an exact result, never used to fall back to an inexact
/// approximation — that would violate S1, not satisfy it.
/// Застосовує *опційну* числову межу ресурсу (`Environment::with_numeric_bit_limit`)
/// — нічого не робить, якщо ця сесія її не налаштувала, що є типовим для
/// кожної фікстури `conformance.my` й Rust-реалізації (див. власну
/// відкриту примітку S1 про довільну точність). Перевіряється після
/// обчислення точного результату, ніколи не використовується, щоб
/// відкотитись до неточного наближення — це порушило б S1, не
/// задовольнило б його.
fn check_numeric_limit(environment: &Environment, result: &Rational, span: Span) -> Result<(), LanguageError> {
    if let Some(limit) = environment.numeric_bit_limit() {
        if result.bit_length() > limit {
            return Err(LanguageError::new(
                ErrorKind::NumericOverflow,
                "exact arithmetic result exceeds the configured bit-length limit · точний результат арифметики перевищує налаштовану межу в бітах · exaktes Arithmetikergebnis überschreitet die konfigurierte Bitlängengrenze",
                span,
            ));
        }
    }
    Ok(())
}

pub(super) fn evaluate_division(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    if arguments.is_empty() {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "/ expects at least one argument · / очікує щонайменше один аргумент · / erwartet mindestens ein Argument",
            span,
        ));
    }
    let mut values = arguments.iter().map(|argument| {
        let value = evaluate(argument, environment)?;
        // Matches on `&value`: see the comment on the same pattern in
        // `numeric_value` above.
        match &value {
            Value::Rational(rational) => Ok(rational.clone()),
            Value::Number(number) if number.fract() == 0.0 && *number >= i64::MIN as f64 && *number <= i64::MAX as f64 => {
                Ok(Rational::integer(*number as i64))
            }
            _ => Err(LanguageError::new(
                ErrorKind::Type,
                "/ expects exact integers or rational numbers · / очікує точні цілі або раціональні числа · / erwartet exakte Ganz- oder rationale Zahlen",
                argument.span,
            )),
        }
    });
    // The empty-arguments case is rejected above, but the iterator is re-derived here
    // rather than trusting that earlier check, so a future reorder cannot turn this into a panic.
    // Порожній список аргументів відхиляється вище, але ітератор тут перевіряється
    // окремо, тож майбутнє перевпорядкування коду не перетвориться на паніку.
    // Der Fall leerer Argumente wird oben abgelehnt, aber der Iterator wird hier erneut
    // geprüft, sodass eine spätere Umordnung dies nicht in einen Panic verwandeln kann.
    let Some(first) = values.next() else {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "/ expects at least one argument · / очікує щонайменше один аргумент · / erwartet mindestens ein Argument",
            span,
        ));
    };
    let mut result = first?;
    if arguments.len() == 1 {
        result = Rational::integer(1)
            .checked_div(result)
            .ok_or_else(|| division_error(span))?;
    } else {
        for divisor in values {
            result = result
                .checked_div(divisor?)
                .ok_or_else(|| division_error(span))?;
        }
    }
    check_numeric_limit(environment, &result, span)?;
    Ok(exact_value(result))
}

/// `<`, `>`, `=`, `<=`, `>=` follow the same exact/inexact promotion rule as
/// `+`/`-`/`*`: if every operand is exact, comparison is exact (`Rational`'s
/// `Ord`, no float involved); one inexact operand makes the whole comparison
/// inexact. Chained like `(< 1 2 3)`: true iff each operand compares against
/// the next in order, same as Scheme/Racket's variadic comparisons.
/// `<`, `>`, `=`, `<=`, `>=` дотримуються того самого правила exact/inexact,
/// що й `+`/`-`/`*`: якщо всі операнди точні, порівняння точне (`Ord` для
/// `Rational`, без float); один неточний операнд робить усе порівняння
/// неточним. Ланцюгове, як `(< 1 2 3)`: істина, якщо кожен операнд
/// порівнюється з наступним по порядку — як варіативні порівняння в
/// Scheme/Racket.
/// `<`, `>`, `=`, `<=`, `>=` folgen derselben exakt/inexakt-Promotionsregel
/// wie `+`/`-`/`*`: sind alle Operanden exakt, ist der Vergleich exakt
/// (`Ord` für `Rational`, kein Float); ein inexakter Operand macht den
/// gesamten Vergleich inexakt. Verkettet wie `(< 1 2 3)`: wahr, wenn jeder
/// Operand im Vergleich zum nächsten in Ordnung ist — wie variadische
/// Vergleiche in Scheme/Racket.
pub(super) fn evaluate_comparison(
    operator: &str,
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    if arguments.is_empty() {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            format!("{operator} expects at least one argument · {operator} очікує щонайменше один аргумент · {operator} erwartet mindestens ein Argument"),
            span,
        ));
    }
    let values = arguments
        .iter()
        .map(|argument| numeric_value(evaluate(argument, environment)?, argument.span))
        .collect::<Result<Vec<_>, _>>()?;

    let holds = if values
        .iter()
        .any(|value| matches!(value, Numeric::Inexact(_)))
    {
        values
            .windows(2)
            .all(|pair| compare(operator, pair[0].as_f64(), pair[1].as_f64()))
    } else {
        values
            .windows(2)
            .all(|pair| compare(operator, pair[0].into_exact(), pair[1].into_exact()))
    };
    Ok(Value::Bool(holds))
}

fn compare<T: PartialOrd>(operator: &str, left: T, right: T) -> bool {
    match operator {
        "<" => left < right,
        ">" => left > right,
        "=" => left == right,
        "<=" => left <= right,
        ">=" => left >= right,
        _ => unreachable!("known comparison operator"),
    }
}

fn division_error(span: Span) -> LanguageError {
    LanguageError::new(
        ErrorKind::InvalidForm,
        "division by zero or rational overflow · ділення на нуль або переповнення дробу · Division durch null oder Bruchüberlauf",
        span,
    )
}

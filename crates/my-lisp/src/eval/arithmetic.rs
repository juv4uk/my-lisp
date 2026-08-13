//! Exact/inexact numeric handling for `+`, `-`, `*`, and `/`.
//! Obrobka tochnykh/netochnykh chysel dlia `+`, `-`, `*` ta `/`.
//! Verarbeitung exakter/inexakter Zahlen für `+`, `-`, `*` und `/`.

use super::evaluate;
use crate::{Environment, ErrorKind, Exactness, Expr, LanguageError, Rational, Span, Value};

pub(super) fn evaluate_arithmetic(
    operator: &str,
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    if operator == "-" && arguments.is_empty() {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "- expects at least one argument · - ochikuie shchonaimenshe odyn arhument · - erwartet mindestens ein Argument",
            span,
        ));
    }
    let values = arguments
        .iter()
        .map(|argument| numeric_value(evaluate(argument, environment)?, argument.span))
        .collect::<Result<Vec<_>, _>>()?;

    // Exact integers and rationals stay exact. One inexact operand deliberately makes the result inexact.
    // Tochni tsili ta ratsionalni lyshaiutsia tochnymy. Odyn netochnyi operand navmysno robyt rezultat netochnym.
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
        return Ok(Value::Number(result, Exactness::Inexact));
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

fn exact_value(value: Rational) -> Value {
    match value.as_precise_i64() {
        Some(n) => Value::Number(n as f64, Exactness::Exact),
        None => Value::Rational(value),
    }
}

fn arithmetic_overflow(span: Span) -> LanguageError {
    LanguageError::new(
        ErrorKind::InvalidForm,
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

pub(super) fn evaluate_division(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    if arguments.is_empty() {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "/ expects at least one argument · / ochikuie shchonaimenshe odyn arhument · / erwartet mindestens ein Argument",
            span,
        ));
    }
    let mut values = arguments.iter().map(|argument| {
        let value = evaluate(argument, environment)?;
        // Matches on `&value`: see the comment on the same pattern in
        // `numeric_value` above.
        match &value {
            Value::Rational(rational) => Ok(rational.clone()),
            Value::Number(number, Exactness::Exact) => Ok(Rational::integer(*number as i64)),
            _ => Err(LanguageError::new(
                ErrorKind::Type,
                "/ expects exact integers or rational numbers · / ochikuie tochni tsili abo ratsionalni chysla · / erwartet exakte Ganz- oder rationale Zahlen",
                argument.span,
            )),
        }
    });
    // The empty-arguments case is rejected above, but the iterator is re-derived here
    // rather than trusting that earlier check, so a future reorder cannot turn this into a panic.
    // Porozhnii spysok arhumentiv vidkhyliaietsia vyshche, ale iterator tut pereviriaietsia
    // okremo, tozh maibutnie perevporiadkuvannia kodu ne peretvorytsia na paniku.
    // Der Fall leerer Argumente wird oben abgelehnt, aber der Iterator wird hier erneut
    // geprüft, sodass eine spätere Umordnung dies nicht in einen Panic verwandeln kann.
    let Some(first) = values.next() else {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "/ expects at least one argument · / ochikuie shchonaimenshe odyn arhument · / erwartet mindestens ein Argument",
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

/// `<`, `>`, `=` follow the same exact/inexact promotion rule as
/// `+`/`-`/`*`: if every operand is exact, comparison is exact (`Rational`'s
/// `Ord`, no float involved); one inexact operand makes the whole comparison
/// inexact. Chained like `(< 1 2 3)`: true iff each operand compares against
/// the next in order, same as Scheme/Racket's variadic comparisons.
/// `<`, `>`, `=` dotrymuiutsia toho samoho pravyla exact/inexact,
/// shcho y `+`/`-`/`*`: yakshcho vsi operandy tochni, porivniannia tochne (`Ord` dlia
/// `Rational`, bez float); odyn netochnyi operand robyt use porivniannia
/// netochnym. Lantsiuhove, yak `(< 1 2 3)`: istyna, yakshcho kozhen operand
/// porivniuietsia z nastupnym po poriadku — yak variatyvni porivniannia v
/// Scheme/Racket.
/// `<`, `>`, `=` folgen derselben exakt/inexakt-Promotionsregel
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
            format!("{operator} expects at least one argument · {operator} ochikuie shchonaimenshe odyn arhument · {operator} erwartet mindestens ein Argument"),
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
        _ => unreachable!("known comparison operator"),
    }
}

fn division_error(span: Span) -> LanguageError {
    LanguageError::new(
        ErrorKind::InvalidForm,
        "division by zero or rational overflow · dilennia na nul abo perepovnennia drobu · Division durch null oder Bruchüberlauf",
        span,
    )
}

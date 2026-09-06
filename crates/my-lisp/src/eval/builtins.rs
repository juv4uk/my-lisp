//! Contract 2.1 bootstrap: registers primitive operations into the root
//! environment as first-class `Value::Builtin` values (docs/
//! PROPOSAL-FIRST-CLASS-BUILTINS.md). After this runs, the evaluator's
//! symbol match no longer owns these names -- the environment is the
//! single runtime authority; the registry in this file is
//! bootstrap-description only.
//!
//! Batch 1 (2026-08-23): car cdr cons eq atom + - * / < > =.
//! Remaining primitives (print, string ops, sha256, json, ...) stay in
//! the evaluator match until converted -- see PROPOSAL §8.

use crate::environment::Environment;
use crate::eval::arithmetic::{
    arithmetic_on_values, comparison_on_values, division_on_values, exact_value,
};
use crate::eval::special_forms::{car_value, cdr_value, cons_values, eq_values};
use crate::{Exactness, NumericBuffer, Rational, Span, Value};

type Native =
    std::rc::Rc<dyn Fn(&[Value], &Environment, Span) -> Result<Value, crate::LanguageError>>;

fn builtin(name: &'static str, func: Native) -> Value {
    Value::Builtin(std::rc::Rc::new(crate::value::Builtin { name, func }))
}

// Convert whole UTC days since 1970-01-01 to a proleptic Gregorian date.
// This keeps the core dependency-free while making the wall-clock result
// unambiguous and deterministic to decode.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year, m, d)
}

fn utc_now_value(span: Span) -> Result<Value, crate::LanguageError> {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| crate::LanguageError::new(
            crate::ErrorKind::Type,
            "utc-now is unavailable before the Unix epoch · utc-now nedostupnyi do Unix epoch · utc-now ist vor der Unix-Epoche nicht verfügbar",
            span,
        ))?;
    let seconds = duration.as_secs();
    let days = (seconds / 86_400) as i64;
    let day_seconds = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = (day_seconds / 3_600) as i64;
    let minute = ((day_seconds % 3_600) / 60) as i64;
    let second = (day_seconds % 60) as i64;
    let number = |value: i64| exact_value(Rational::integer(value));
    Ok(Value::list([
        Value::Symbol(std::rc::Rc::from("utc")),
        number(year),
        number(month),
        number(day),
        number(hour),
        number(minute),
        number(second),
        number(duration.subsec_nanos() as i64),
    ]))
}

fn internet_time_sync_value(
    host: &str,
    timeout_ms: u64,
    span: Span,
) -> Result<Value, crate::LanguageError> {
    use std::net::{ToSocketAddrs, UdpSocket};
    use std::time::Duration;
    let timeout_ms = timeout_ms.min(5_000);
    let address = (host, 123)
        .to_socket_addrs()
        .map_err(|_| {
            crate::LanguageError::new(
                crate::ErrorKind::Type,
                "internet-time-sync cannot resolve host",
                span,
            )
        })?
        .next()
        .ok_or_else(|| {
            crate::LanguageError::new(
                crate::ErrorKind::Type,
                "internet-time-sync host has no address",
                span,
            )
        })?;
    let socket = UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket.set_read_timeout(Some(Duration::from_millis(timeout_ms)))?;
            socket.set_write_timeout(Some(Duration::from_millis(timeout_ms)))?;
            socket.connect(address)?;
            Ok(socket)
        })
        .map_err(|_| {
            crate::LanguageError::new(
                crate::ErrorKind::Type,
                "internet-time-sync socket unavailable",
                span,
            )
        })?;
    let mut request = [0u8; 48];
    request[0] = 0x23; // LI=0, version=4, client mode=3.
    if socket.send(&request).is_err() {
        return Ok(Value::list([
            Value::Symbol(std::rc::Rc::from("rejected")),
            Value::Symbol(std::rc::Rc::from("send-failed")),
        ]));
    }
    let mut response = [0u8; 512];
    let size = match socket.recv(&mut response) {
        Ok(size) => size,
        Err(_) => {
            return Ok(Value::list([
                Value::Symbol(std::rc::Rc::from("rejected")),
                Value::Symbol(std::rc::Rc::from("receive-failed")),
            ]));
        }
    };
    let reason = if size < 48 {
        Some("short-response")
    } else {
        let mode = response[0] & 0x07;
        let stratum = response[1];
        let seconds = u32::from_be_bytes([response[40], response[41], response[42], response[43]]);
        if !(mode == 4 || mode == 5) || stratum == 0 || stratum > 15 {
            Some("invalid-response")
        } else if seconds < 2_208_988_800 {
            Some("invalid-epoch")
        } else {
            None
        }
    };
    if let Some(reason) = reason {
        return Ok(Value::list([
            Value::Symbol(std::rc::Rc::from("rejected")),
            Value::Symbol(std::rc::Rc::from(reason)),
        ]));
    }
    let seconds = u32::from_be_bytes([response[40], response[41], response[42], response[43]]);
    let fraction = u32::from_be_bytes([response[44], response[45], response[46], response[47]]);
    let unix_seconds = (seconds - 2_208_988_800) as i64;
    let nanosecond = ((fraction as u64 * 1_000_000_000) >> 32) as i64;
    Ok(Value::list([
        Value::Symbol(std::rc::Rc::from("accepted")),
        Value::String(std::rc::Rc::from(host)),
        exact_value(Rational::integer(unix_seconds)),
        exact_value(Rational::integer(nanosecond)),
    ]))
}

/// Registers batch-1 builtins into `environment`. Idempotent per name:
/// later definitions simply shadow earlier ones like any other binding.
pub(crate) fn install(environment: &Environment) {
    macro_rules! define {
        ($env:expr, $name:expr, $f:expr) => {
            $env.define($name, builtin($name, std::rc::Rc::new($f)));
        };
    }

    define!(
        environment,
        "car",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("car", args, 1, span)?;
            car_value(&args[0], span)
        }
    );

    define!(
        environment,
        "cdr",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("cdr", args, 1, span)?;
            cdr_value(&args[0], span)
        }
    );

    define!(
        environment,
        "cons",
        |args: &[Value], env: &Environment, span: Span| {
            exact_args("cons", args, 2, span)?;
            cons_values(args[0].clone(), args[1].clone(), env, span)
        }
    );

    define!(
        environment,
        "eq",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("eq", args, 2, span)?;
            eq_values(args[0].clone(), args[1].clone(), span)
        }
    );

    define!(
        environment,
        "atom",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("atom", args, 1, span)?;
            Ok(Value::truth(args[0].is_atom()))
        }
    );

    define!(
        environment,
        "abs",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("abs", args, 1, span)?;
            Ok(match &args[0] {
                crate::Value::Number(f, e) => {
                    crate::Value::Number(if *f < 0.0 { -*f } else { *f }, *e)
                }
                crate::Value::Rational(r) => {
                    let neg = r.is_negative();
                    if neg {
                        crate::Value::Rational(-r.clone())
                    } else {
                        crate::Value::Rational(r.clone())
                    }
                }
                other => other.clone(),
            })
        }
    );

    define!(
        environment,
        "min-list",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("min-list", args, 1, span)?;
            let mut items = Vec::new();
            let mut cur = args[0].clone();
            while let crate::Value::Pair(h, t) = &cur {
                items.push((**h).clone());
                cur = (**t).clone();
            }
            if items.is_empty() {
                return Ok(crate::Value::Nil);
            }
            let mut best = items[0].clone();
            for item in &items[1..] {
                if super::arithmetic::order_pair("<", item, &best, span)? {
                    best = item.clone();
                }
            }
            Ok(best)
        }
    );

    define!(
        environment,
        "max-list",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("max-list", args, 1, span)?;
            let mut items = Vec::new();
            let mut cur = args[0].clone();
            while let crate::Value::Pair(h, t) = &cur {
                items.push((**h).clone());
                cur = (**t).clone();
            }
            if items.is_empty() {
                return Ok(crate::Value::Nil);
            }
            let mut best = items[0].clone();
            for item in &items[1..] {
                if super::arithmetic::order_pair(">", item, &best, span)? {
                    best = item.clone();
                }
            }
            Ok(best)
        }
    );

    define!(
        environment,
        "min",
        |args: &[Value], _env: &Environment, span: Span| {
            if args.is_empty() {
                return Err(crate::LanguageError::new(
                crate::ErrorKind::Arity,
                "min expects at least one argument · min ochikuie shchonaimenshe odyn arhument · min erwartet mindestens ein Argument",
                span,
            ));
            }
            let mut best = args[0].clone();
            for v in &args[1..] {
                if super::arithmetic::order_pair("<", v, &best, span)? {
                    best = v.clone();
                }
            }
            Ok(best)
        }
    );

    define!(
        environment,
        "max",
        |args: &[Value], _env: &Environment, span: Span| {
            if args.is_empty() {
                return Err(crate::LanguageError::new(
                crate::ErrorKind::Arity,
                "max expects at least one argument · max ochikuie shchonaimenshe odyn arhument · max erwartet mindestens ein Argument",
                span,
            ));
            }
            let mut best = args[0].clone();
            for v in &args[1..] {
                if super::arithmetic::order_pair(">", v, &best, span)? {
                    best = v.clone();
                }
            }
            Ok(best)
        }
    );

    define!(
        environment,
        "make-vector",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("make-vector", args, 1, span)?;
            match &args[0] {
            Value::Number(f, Exactness::Exact) if *f >= 0.0 && f.fract() == 0.0 => {
                Ok(Value::vector(std::iter::repeat_n(Value::Nil, *f as usize)))
            }
            _ => Err(crate::LanguageError::new(
                crate::ErrorKind::Type,
                "make-vector expects an exact non-negative integer · make-vector ochikuie tochnyi nenulevyi tsilyi · make-vector erwartet eine exakte nichtnegative ganze Zahl",
                span,
            )),
        }
        }
    );

    define!(
        environment,
        "vector",
        |args: &[Value], _env: &Environment, _span: Span| {
            Ok(Value::vector(args.iter().cloned()))
        }
    );

    // (mono-ns) — the single host monotonic clock observation primitive.
    // It goes through `exact_value`/`Rational`: an `f64` only represents
    // integers losslessly up to 2^53, which a nanosecond count reaches after
    // ~104 days of process uptime. `exact_value` falls back to a `Rational`
    // past that point instead of silently rounding, so `(mono-ns)` stays exact
    // for the life of the process. Millisecond views are derived in Lisp.
    define!(
        environment,
        "mono-ns",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("mono-ns", args, 0, span)?;
            static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
            let elapsed = START.get_or_init(std::time::Instant::now).elapsed();
            Ok(exact_value(Rational::integer(elapsed.as_nanos() as i64)))
        }
    );

    // (utc-now) -> (utc year month day hour minute second nanosecond).
    // The calendar is UTC and the final field preserves the clock reading
    // to nanosecond resolution; this is wall-clock observation, not a
    // monotonic timer and must not replace logical FS revisions.
    // (utc-now) -> (utc рік місяць день година хвилина секунда наносекунда).
    // Календар UTC, останнє поле зберігає показ до наносекунди; це спостереження
    // годинника, а не монотонний таймер і не заміна логічних revision FS.
    define!(
        environment,
        "utc-now",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("utc-now", args, 0, span)?;
            utc_now_value(span)
        }
    );

    // (internet-time-sync host timeout-ms) performs one bounded NTP query.
    // It returns data, never changes the operating-system clock, and caps the
    // timeout at five seconds. Internet time is an external observation.
    define!(
        environment,
        "internet-time-sync",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("internet-time-sync", args, 2, span)?;
            let host = match &args[0] {
                Value::String(value) => value.as_ref(),
                _ => {
                    return Err(crate::LanguageError::new(
                        crate::ErrorKind::Type,
                        "internet-time-sync expects host string",
                        span,
                    ))
                }
            };
            let timeout = match &args[1] {
                Value::Number(value, Exactness::Exact) if *value >= 0.0 && value.fract() == 0.0 => {
                    *value as u64
                }
                _ => {
                    return Err(crate::LanguageError::new(
                        crate::ErrorKind::Type,
                        "internet-time-sync expects exact timeout milliseconds",
                        span,
                    ))
                }
            };
            internet_time_sync_value(host, timeout, span)
        }
    );

    // (timezone-detect) observes the host's explicit timezone declaration.
    // It does not guess from coordinates and does not mutate the host.
    define!(
        environment,
        "timezone-detect",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("timezone-detect", args, 0, span)?;
            if let Ok(value) = std::env::var("TZ") {
                if !value.is_empty() {
                    return Ok(Value::list([
                        Value::Symbol(std::rc::Rc::from("detected")),
                        Value::String(std::rc::Rc::from(value)),
                        Value::Symbol(std::rc::Rc::from("TZ")),
                    ]));
                }
            }
            if let Ok(value) = std::fs::read_to_string("/etc/timezone") {
                let value = value.trim();
                if !value.is_empty() {
                    return Ok(Value::list([
                        Value::Symbol(std::rc::Rc::from("detected")),
                        Value::String(std::rc::Rc::from(value)),
                        Value::Symbol(std::rc::Rc::from("etc-timezone")),
                    ]));
                }
            }
            Ok(Value::list([
                Value::Symbol(std::rc::Rc::from("unknown")),
                Value::Symbol(std::rc::Rc::from("host-declaration-unavailable")),
            ]))
        }
    );

    define!(
        environment,
        "vector-length",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("vector-length", args, 1, span)?;
            match &args[0] {
                Value::Vector(vec) => {
                    Ok(Value::Number(vec.borrow().len() as f64, Exactness::Exact))
                }
                _ => Err(crate::LanguageError::new(
                    crate::ErrorKind::Type,
                    "vector-length expects a vector",
                    span,
                )),
            }
        }
    );

    define!(
        environment,
        "vector-ref",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("vector-ref", args, 2, span)?;
            let index = match &args[1] {
                Value::Number(f, Exactness::Exact)
                    if *f >= 0.0 && f.fract() == 0.0 && *f <= usize::MAX as f64 =>
                {
                    *f as usize
                }
                _ => {
                    return Err(crate::LanguageError::new(
                        crate::ErrorKind::Type,
                        "vector-ref expects an exact non-negative integer index",
                        span,
                    ))
                }
            };
            match &args[0] {
                Value::Vector(vec) => vec.borrow().get(index).cloned().ok_or_else(|| {
                    crate::LanguageError::new(
                        crate::ErrorKind::InvalidForm,
                        format!(
                            "vector-ref index {index} out of bounds for length {}",
                            vec.borrow().len()
                        ),
                        span,
                    )
                }),
                _ => Err(crate::LanguageError::new(
                    crate::ErrorKind::Type,
                    "vector-ref expects a vector",
                    span,
                )),
            }
        }
    );

    define!(
        environment,
        "vector-set!",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("vector-set!", args, 3, span)?;
            let index = match &args[1] {
                Value::Number(f, Exactness::Exact)
                    if *f >= 0.0 && f.fract() == 0.0 && *f <= usize::MAX as f64 =>
                {
                    *f as usize
                }
                _ => {
                    return Err(crate::LanguageError::new(
                        crate::ErrorKind::Type,
                        "vector-set! expects an exact non-negative integer index",
                        span,
                    ))
                }
            };
            match &args[0] {
                Value::Vector(vec) => {
                    let mut vec = vec.borrow_mut();
                    if index >= vec.len() {
                        let len = vec.len();
                        return Err(crate::LanguageError::new(
                            crate::ErrorKind::InvalidForm,
                            format!("vector-set! index {index} out of bounds for length {len}"),
                            span,
                        ));
                    }
                    vec[index] = args[2].clone();
                    Ok(Value::Nil)
                }
                _ => Err(crate::LanguageError::new(
                    crate::ErrorKind::Type,
                    "vector-set! expects a vector",
                    span,
                )),
            }
        }
    );

    define!(
        environment,
        "i32-buffer",
        |args: &[Value], _env: &Environment, span: Span| {
            let mut values = Vec::with_capacity(args.len());
            for value in args {
                let integer = match value {
                    Value::Number(number, Exactness::Exact) if number.fract() == 0.0 => {
                        *number as i64
                    }
                    Value::Rational(rational) if rational.is_integer() => {
                        rational.as_precise_i64().ok_or_else(|| {
                            crate::LanguageError::new(
                                crate::ErrorKind::NumericOverflow,
                                "i32-buffer element is outside the signed 32-bit range",
                                span,
                            )
                        })?
                    }
                    _ => {
                        return Err(crate::LanguageError::new(
                            crate::ErrorKind::Type,
                            "i32-buffer expects exact integer elements",
                            span,
                        ))
                    }
                };
                values.push(i32::try_from(integer).map_err(|_| {
                    crate::LanguageError::new(
                        crate::ErrorKind::NumericOverflow,
                        "i32-buffer element is outside the signed 32-bit range",
                        span,
                    )
                })?);
            }
            Ok(Value::NumericBuffer(NumericBuffer::I32(values.into())))
        }
    );

    define!(
        environment,
        "f32-buffer",
        |args: &[Value], _env: &Environment, span: Span| {
            let mut values = Vec::with_capacity(args.len());
            for value in args {
                let number = match value {
                    Value::Number(number, _) => *number,
                    Value::Rational(rational) => rational.as_f64(),
                    _ => {
                        return Err(crate::LanguageError::new(
                            crate::ErrorKind::Type,
                            "f32-buffer expects numeric elements",
                            span,
                        ))
                    }
                };
                let narrowed = number as f32;
                if !number.is_finite() || !narrowed.is_finite() {
                    return Err(crate::LanguageError::new(
                        crate::ErrorKind::NumericOverflow,
                        "f32-buffer element is outside the finite binary32 domain",
                        span,
                    ));
                }
                values.push(narrowed);
            }
            Ok(Value::NumericBuffer(NumericBuffer::F32(values.into())))
        }
    );

    define!(
        environment,
        "string-slice",
        |args: &[Value], _env: &Environment, span: Span| {
            super::special_forms::evaluate_string_slice(args, span)
        }
    );

    define!(
        environment,
        "numeric-buffer?",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("numeric-buffer?", args, 1, span)?;
            Ok(if matches!(args[0], Value::NumericBuffer(_)) {
                Value::truth(true)
            } else {
                Value::Nil
            })
        }
    );

    define!(
        environment,
        "numeric-buffer-type",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("numeric-buffer-type", args, 1, span)?;
            let name = match &args[0] {
                Value::NumericBuffer(NumericBuffer::I32(_)) => "i32",
                Value::NumericBuffer(NumericBuffer::F32(_)) => "f32",
                _ => {
                    return Err(crate::LanguageError::new(
                        crate::ErrorKind::Type,
                        "numeric-buffer-type expects a numeric buffer",
                        span,
                    ))
                }
            };
            Ok(Value::Symbol(name.into()))
        }
    );

    define!(
        environment,
        "numeric-buffer-length",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("numeric-buffer-length", args, 1, span)?;
            let length = match &args[0] {
                Value::NumericBuffer(NumericBuffer::I32(values)) => values.len(),
                Value::NumericBuffer(NumericBuffer::F32(values)) => values.len(),
                _ => {
                    return Err(crate::LanguageError::new(
                        crate::ErrorKind::Type,
                        "numeric-buffer-length expects a numeric buffer",
                        span,
                    ))
                }
            };
            Ok(Value::Number(length as f64, Exactness::Exact))
        }
    );

    define!(
        environment,
        "numeric-buffer-ref",
        |args: &[Value], _env: &Environment, span: Span| {
            exact_args("numeric-buffer-ref", args, 2, span)?;
            let index = match args[1] {
                Value::Number(number, Exactness::Exact)
                    if number >= 0.0 && number.fract() == 0.0 && number <= usize::MAX as f64 =>
                {
                    number as usize
                }
                _ => {
                    return Err(crate::LanguageError::new(
                        crate::ErrorKind::Type,
                        "numeric-buffer-ref expects an exact non-negative integer index",
                        span,
                    ))
                }
            };
            match &args[0] {
                Value::NumericBuffer(NumericBuffer::I32(values)) => values
                    .get(index)
                    .map(|value| Value::Number(f64::from(*value), Exactness::Exact)),
                Value::NumericBuffer(NumericBuffer::F32(values)) => values
                    .get(index)
                    .map(|value| Value::Number(f64::from(*value), Exactness::Inexact)),
                _ => {
                    return Err(crate::LanguageError::new(
                        crate::ErrorKind::Type,
                        "numeric-buffer-ref expects a numeric buffer",
                        span,
                    ))
                }
            }
            .ok_or_else(|| {
                crate::LanguageError::new(
                    crate::ErrorKind::InvalidForm,
                    "numeric-buffer-ref index is out of bounds",
                    span,
                )
            })
        }
    );

    define!(
        environment,
        "numeric-buffer-map",
        |args: &[Value], env: &Environment, span: Span| {
            exact_args("numeric-buffer-map", args, 2, span)?;
            match &args[1] {
                Value::NumericBuffer(NumericBuffer::I32(input)) => {
                    let mut output = Vec::with_capacity(input.len());
                    for element in input.iter() {
                        let result = super::invoke_value(
                            &args[0],
                            &[Value::Number(f64::from(*element), Exactness::Exact)],
                            env,
                            span,
                        )?;
                        let integer = match &result {
                            Value::Number(number, Exactness::Exact) if number.fract() == 0.0 => {
                                *number as i64
                            }
                            Value::Rational(rational) if rational.is_integer() => {
                                rational.as_precise_i64().ok_or_else(|| {
                                    crate::LanguageError::new(
                                crate::ErrorKind::NumericOverflow,
                                "numeric-buffer-map i32 result is outside the signed 32-bit range",
                                span,
                            )
                                })?
                            }
                            _ => {
                                return Err(crate::LanguageError::new(
                                    crate::ErrorKind::Type,
                                    "numeric-buffer-map over i32 requires exact integer results",
                                    span,
                                ))
                            }
                        };
                        output.push(i32::try_from(integer).map_err(|_| {
                            crate::LanguageError::new(
                                crate::ErrorKind::NumericOverflow,
                                "numeric-buffer-map i32 result is outside the signed 32-bit range",
                                span,
                            )
                        })?);
                    }
                    Ok(Value::NumericBuffer(NumericBuffer::I32(output.into())))
                }
                Value::NumericBuffer(NumericBuffer::F32(input)) => {
                    let mut output = Vec::with_capacity(input.len());
                    for bits in input.iter() {
                        let result = super::invoke_value(
                            &args[0],
                            &[Value::Number(f64::from(*bits), Exactness::Inexact)],
                            env,
                            span,
                        )?;
                        let number = match &result {
                            Value::Number(number, _) => *number,
                            Value::Rational(rational) => rational.as_f64(),
                            _ => {
                                return Err(crate::LanguageError::new(
                                    crate::ErrorKind::Type,
                                    "numeric-buffer-map over f32 requires numeric results",
                                    span,
                                ))
                            }
                        };
                        let narrowed = number as f32;
                        if !number.is_finite() || !narrowed.is_finite() {
                            return Err(crate::LanguageError::new(
                            crate::ErrorKind::NumericOverflow,
                            "numeric-buffer-map f32 result is outside the finite binary32 domain",
                            span,
                        ));
                        }
                        output.push(narrowed);
                    }
                    Ok(Value::NumericBuffer(NumericBuffer::F32(output.into())))
                }
                _ => Err(crate::LanguageError::new(
                    crate::ErrorKind::Type,
                    "numeric-buffer-map expects a numeric buffer",
                    span,
                )),
            }
        }
    );

    for op in ["+", "-", "*"] {
        define!(
            environment,
            op,
            move |args: &[Value], env: &Environment, span: Span| {
                arithmetic_on_values(op, args, env, span)
            }
        );
    }

    define!(
        environment,
        "/",
        move |args: &[Value], env: &Environment, span: Span| {
            division_on_values(args, args.len(), env, span)
        }
    );

    define!(
        environment,
        "env",
        |args: &[Value], env: &Environment, span: Span| {
            exact_args("env", args, 0, span)?;
            let _ = span;
            let mut items = Vec::new();
            for (name, value) in env.snapshot() {
                // skip the marker boolean t's own entry noise? keep everything:
                items.push(crate::Value::Pair(
                    std::rc::Rc::new(crate::Value::String(name)),
                    std::rc::Rc::new(value),
                ));
            }
            let mut list = crate::Value::Nil;
            for item in items.into_iter().rev() {
                list = crate::Value::Pair(std::rc::Rc::new(item), std::rc::Rc::new(list));
            }
            Ok(list)
        }
    );

    for op in ["<", ">", "="] {
        define!(
            environment,
            op,
            move |args: &[Value], _env: &Environment, span: Span| {
                comparison_on_values(op, args, span)
            }
        );
    }
}

fn exact_args(
    name: &'static str,
    args: &[Value],
    expected: usize,
    span: Span,
) -> Result<(), crate::LanguageError> {
    if args.len() != expected {
        return Err(crate::LanguageError::new(
            crate::ErrorKind::Arity,
            format!("{name} expects exactly {expected} argument(s)"),
            span,
        ));
    }
    Ok(())
}
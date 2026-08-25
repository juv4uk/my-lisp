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

type Native = std::rc::Rc<dyn Fn(&[Value], &Environment, Span) -> Result<Value, crate::LanguageError>>;

fn builtin(name: &'static str, func: Native) -> Value {
    Value::Builtin(std::rc::Rc::new(crate::value::Builtin { name, func }))
}

/// Registers batch-1 builtins into `environment`. Idempotent per name:
/// later definitions simply shadow earlier ones like any other binding.
pub(crate) fn install(environment: &Environment) {

    macro_rules! define {
        ($env:expr, $name:expr, $f:expr) => {
            $env.define($name, builtin($name, std::rc::Rc::new($f)));
        };
    }

    define!(environment, "car", |args: &[Value], _env: &Environment, span: Span| {
        exact_args("car", args, 1, span)?;
        car_value(&args[0], span)
    });

    define!(environment, "cdr", |args: &[Value], _env: &Environment, span: Span| {
        exact_args("cdr", args, 1, span)?;
        cdr_value(&args[0], span)
    });

    define!(environment, "cons", |args: &[Value], env: &Environment, span: Span| {
        exact_args("cons", args, 2, span)?;
        cons_values(args[0].clone(), args[1].clone(), env, span)
    });

    define!(environment, "eq", |args: &[Value], _env: &Environment, span: Span| {
        exact_args("eq", args, 2, span)?;
        eq_values(args[0].clone(), args[1].clone(), span)
    });

    define!(environment, "atom", |args: &[Value], _env: &Environment, span: Span| {
        exact_args("atom", args, 1, span)?;
        Ok(Value::Bool(args[0].is_atom()))
    });


    define!(environment, "abs", |args: &[Value], _env: &Environment, span: Span| {
        exact_args("abs", args, 1, span)?;
        Ok(match &args[0] {
            crate::Value::Number(f, e) => crate::Value::Number(if *f < 0.0 { -*f } else { *f }, *e),
            crate::Value::Rational(r) => {
                let neg = r.is_negative();
                if neg { crate::Value::Rational(-r.clone()) } else { crate::Value::Rational(r.clone()) }
            }
            other => other.clone(),
        })
    });

    define!(environment, "min-list", |args: &[Value], _env: &Environment, span: Span| {
        exact_args("min-list", args, 1, span)?;
        let mut items = Vec::new();
        let mut cur = args[0].clone();
        while let crate::Value::Pair(h, t) = &cur {
            items.push((**h).clone());
            cur = (**t).clone();
        }
        if items.is_empty() { return Ok(crate::Value::Nil); }
        let mut best = items[0].clone();
        for item in &items[1..] {
            if super::arithmetic::order_pair("<", item, &best, span)? { best = item.clone(); }
        }
        Ok(best)
    });

    define!(environment, "max-list", |args: &[Value], _env: &Environment, span: Span| {
        exact_args("max-list", args, 1, span)?;
        let mut items = Vec::new();
        let mut cur = args[0].clone();
        while let crate::Value::Pair(h, t) = &cur {
            items.push((**h).clone());
            cur = (**t).clone();
        }
        if items.is_empty() { return Ok(crate::Value::Nil); }
        let mut best = items[0].clone();
        for item in &items[1..] {
            if super::arithmetic::order_pair(">", item, &best, span)? { best = item.clone(); }
        }
        Ok(best)
    });

    define!(environment, "min", |args: &[Value], _env: &Environment, span: Span| {
        if args.is_empty() {
            return Err(crate::LanguageError::new(
                crate::ErrorKind::Arity,
                "min expects at least one argument · min ochikuie shchonaimenshe odyn arhument · min erwartet mindestens ein Argument",
                span,
            ));
        }
        let mut best = args[0].clone();
        for v in &args[1..] {
            if super::arithmetic::order_pair("<", v, &best, span)? { best = v.clone(); }
        }
        Ok(best)
    });

    define!(environment, "max", |args: &[Value], _env: &Environment, span: Span| {
        if args.is_empty() {
            return Err(crate::LanguageError::new(
                crate::ErrorKind::Arity,
                "max expects at least one argument · max ochikuie shchonaimenshe odyn arhument · max erwartet mindestens ein Argument",
                span,
            ));
        }
        let mut best = args[0].clone();
        for v in &args[1..] {
            if super::arithmetic::order_pair(">", v, &best, span)? { best = v.clone(); }
        }
        Ok(best)
    });

    define!(environment, "make-vector", |args: &[Value], _env: &Environment, span: Span| {
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
    });

    define!(environment, "vector", |args: &[Value], _env: &Environment, _span: Span| {
        Ok(Value::vector(args.iter().cloned()))
    });

    // (mono-ms) — monotonie milliseconds since first call in this process.
    // Wall-clock-independent, so diffs measure true elapsed time of a block:
    //   (define t0 (mono-ms)) <block> (- (mono-ms) t0)
    // Library-before-core doctrine: this is the minimal host primitive that
    // CANNOT be expressed in the language itself; everything else (lap
    // timers, `timed` wrappers) stays library-level.
    define!(environment, "mono-ms", |args: &[Value], _env: &Environment, span: Span| {
        exact_args("mono-ms", args, 0, span)?;
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let elapsed = START.get_or_init(std::time::Instant::now).elapsed();
        Ok(Value::Number(elapsed.as_millis() as f64, Exactness::Exact))
    });

    // (mono-ns) — same doctrine as `mono-ms`, at nanosecond resolution.
    // Goes through `exact_value`/`Rational` rather than `mono-ms`'s direct
    // `as f64` cast: an `f64` only represents integers losslessly up to
    // 2^53, which `mono-ms` never reaches at millisecond resolution
    // (~285000 years), but a nanosecond count reaches it after ~104 days of
    // process uptime — a real risk for a long-running agent process, not a
    // hypothetical one. `exact_value` falls back to a `Rational` past that
    // point instead of silently rounding, so `(mono-ns)` stays exact for
    // the life of the process.
    define!(environment, "mono-ns", |args: &[Value], _env: &Environment, span: Span| {
        exact_args("mono-ns", args, 0, span)?;
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let elapsed = START.get_or_init(std::time::Instant::now).elapsed();
        Ok(exact_value(Rational::integer(elapsed.as_nanos() as i64)))
    });

    define!(environment, "vector-length", |args: &[Value], _env: &Environment, span: Span| {
        exact_args("vector-length", args, 1, span)?;
        match &args[0] {
            Value::Vector(vec) => Ok(Value::Number(vec.borrow().len() as f64, Exactness::Exact)),
            _ => Err(crate::LanguageError::new(
                crate::ErrorKind::Type,
                "vector-length expects a vector",
                span,
            )),
        }
    });

    define!(environment, "vector-ref", |args: &[Value], _env: &Environment, span: Span| {
        exact_args("vector-ref", args, 2, span)?;
        let index = match &args[1] {
            Value::Number(f, Exactness::Exact) if *f >= 0.0 && f.fract() == 0.0 && *f <= usize::MAX as f64 => *f as usize,
            _ => return Err(crate::LanguageError::new(
                crate::ErrorKind::Type,
                "vector-ref expects an exact non-negative integer index",
                span,
            )),
        };
        match &args[0] {
            Value::Vector(vec) => vec.borrow().get(index).cloned().ok_or_else(|| {
                crate::LanguageError::new(
                    crate::ErrorKind::InvalidForm,
                    format!("vector-ref index {index} out of bounds for length {}", vec.borrow().len()),
                    span,
                )
            }),
            _ => Err(crate::LanguageError::new(
                crate::ErrorKind::Type,
                "vector-ref expects a vector",
                span,
            )),
        }
    });

    define!(environment, "vector-set!", |args: &[Value], _env: &Environment, span: Span| {
        exact_args("vector-set!", args, 3, span)?;
        let index = match &args[1] {
            Value::Number(f, Exactness::Exact) if *f >= 0.0 && f.fract() == 0.0 && *f <= usize::MAX as f64 => *f as usize,
            _ => return Err(crate::LanguageError::new(
                crate::ErrorKind::Type,
                "vector-set! expects an exact non-negative integer index",
                span,
            )),
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
    });

    define!(environment, "i32-buffer", |args: &[Value], _env: &Environment, span: Span| {
        let mut values = Vec::with_capacity(args.len());
        for value in args {
            let integer = match value {
                Value::Number(number, Exactness::Exact) if number.fract() == 0.0 => *number as i64,
                Value::Rational(rational) if rational.is_integer() => rational
                    .as_precise_i64()
                    .ok_or_else(|| {
                        crate::LanguageError::new(
                            crate::ErrorKind::NumericOverflow,
                            "i32-buffer element is outside the signed 32-bit range",
                            span,
                        )
                    })?,
                _ => return Err(crate::LanguageError::new(
                    crate::ErrorKind::Type,
                    "i32-buffer expects exact integer elements",
                    span,
                )),
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
    });

    define!(environment, "f32-buffer", |args: &[Value], _env: &Environment, span: Span| {
        let mut values = Vec::with_capacity(args.len());
        for value in args {
            let number = match value {
                Value::Number(number, _) => *number,
                Value::Rational(rational) => rational.as_f64(),
                _ => return Err(crate::LanguageError::new(
                    crate::ErrorKind::Type,
                    "f32-buffer expects numeric elements",
                    span,
                )),
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
    });

    define!(environment, "string-slice", |args: &[Value], _env: &Environment, span: Span| {
        super::special_forms::evaluate_string_slice(args, span)
    });

    define!(environment, "numeric-buffer?", |args: &[Value], _env: &Environment, span: Span| {
        exact_args("numeric-buffer?", args, 1, span)?;
        Ok(if matches!(args[0], Value::NumericBuffer(_)) {
            Value::Bool(true)
        } else {
            Value::Nil
        })
    });

    define!(environment, "numeric-buffer-type", |args: &[Value], _env: &Environment, span: Span| {
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
    });

    define!(environment, "numeric-buffer-length", |args: &[Value], _env: &Environment, span: Span| {
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
    });

    define!(environment, "numeric-buffer-ref", |args: &[Value], _env: &Environment, span: Span| {
        exact_args("numeric-buffer-ref", args, 2, span)?;
        let index = match args[1] {
            Value::Number(number, Exactness::Exact)
                if number >= 0.0
                    && number.fract() == 0.0
                    && number <= usize::MAX as f64 =>
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
    });

    define!(environment, "numeric-buffer-map", |args: &[Value], env: &Environment, span: Span| {
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
                        Value::Rational(rational) if rational.is_integer() => rational
                            .as_precise_i64()
                            .ok_or_else(|| crate::LanguageError::new(
                                crate::ErrorKind::NumericOverflow,
                                "numeric-buffer-map i32 result is outside the signed 32-bit range",
                                span,
                            ))?,
                        _ => return Err(crate::LanguageError::new(
                            crate::ErrorKind::Type,
                            "numeric-buffer-map over i32 requires exact integer results",
                            span,
                        )),
                    };
                    output.push(i32::try_from(integer).map_err(|_| crate::LanguageError::new(
                        crate::ErrorKind::NumericOverflow,
                        "numeric-buffer-map i32 result is outside the signed 32-bit range",
                        span,
                    ))?);
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
                        _ => return Err(crate::LanguageError::new(
                            crate::ErrorKind::Type,
                            "numeric-buffer-map over f32 requires numeric results",
                            span,
                        )),
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
    });

    for op in ["+", "-", "*"] {
        define!(environment, op, move |args: &[Value], env: &Environment, span: Span| {
            arithmetic_on_values(op, args, env, span)
        });
    }

    define!(environment, "/", move |args: &[Value], env: &Environment, span: Span| {
        division_on_values(args, args.len(), env, span)
    });

    define!(environment, "env", |args: &[Value], env: &Environment, span: Span| {
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
    });

    for op in ["<", ">", "="] {
        define!(environment, op, move |args: &[Value], _env: &Environment, span: Span| {
            comparison_on_values(op, args, span)
        });
    }
}

fn exact_args(name: &'static str, args: &[Value], expected: usize, span: Span) -> Result<(), crate::LanguageError> {
    if args.len() != expected {
        return Err(crate::LanguageError::new(
            crate::ErrorKind::Arity,
            format!("{name} expects exactly {expected} argument(s)"),
            span,
        ));
    }
    Ok(())
}

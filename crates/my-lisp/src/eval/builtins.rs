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
    arithmetic_on_values, comparison_on_values, division_on_values,
};
use crate::eval::special_forms::{car_value, cdr_value, cons_values, eq_values};
use crate::{Span, Value};

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
        loop {
            match &cur {
                crate::Value::Pair(h, t) => { items.push((**h).clone()); cur = (**t).clone(); }
                _ => break,
            }
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
        loop {
            match &cur {
                crate::Value::Pair(h, t) => { items.push((**h).clone()); cur = (**t).clone(); }
                _ => break,
            }
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

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

    for op in ["+", "-", "*"] {
        define!(environment, op, move |args: &[Value], env: &Environment, span: Span| {
            arithmetic_on_values(op, args, env, span)
        });
    }

    define!(environment, "/", move |args: &[Value], env: &Environment, span: Span| {
        division_on_values(args, args.len(), env, span)
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

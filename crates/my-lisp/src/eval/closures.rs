//! `lambda` construction and applying closures/macros to arguments.
//! Побудова `lambda` та застосування замикань/макросів до аргументів.
//! Bau von `lambda` und Anwendung von Closures/Makros auf Argumente.

use super::{evaluate, special_forms::quoted, EvalStep};
use crate::{Closure, Environment, ErrorKind, Expr, ExprKind, LanguageError, Span, Value};
use std::{collections::HashSet, rc::Rc};

/// Parses a lambda-list, which comes in three shapes shared across the Lisp
/// family (not one dialect's `&rest` keyword): `(a b)` — exactly two fixed
/// parameters, no rest; `(a b . rest)` — a dotted list, `rest` bound to
/// every argument past `a`/`b`; and a bare symbol `args` — zero fixed
/// parameters, every argument bound to `args`. The third shape reads as an
/// ordinary `ExprKind::Symbol`, the second as nested `ExprKind::Pair` (the
/// same dotted-pair reader support added earlier for data literals),
/// exactly the shapes `parser.rs` already produces — no new parser syntax.
/// Розбирає lambda-list, що має три форми, спільні для родини Lisp (не
/// ключове слово `&rest` одного діалекту): `(a b)` — точно два фіксовані
/// параметри, без rest; `(a b . rest)` — dotted-список, `rest` зв'язується
/// з усіма аргументами понад `a`/`b`; голий символ `args` — нуль фіксованих
/// параметрів, кожен аргумент зв'язується з `args`. Третя форма читається як
/// звичайний `ExprKind::Symbol`, друга — як вкладений `ExprKind::Pair` (та
/// сама підтримка dotted-pair reader'а, додана раніше для літералів даних) —
/// саме ті форми, які `parser.rs` уже й так виробляє, без нового синтаксису.
fn parse_lambda_list(expr: &Expr) -> Result<(Vec<Rc<str>>, Option<Rc<str>>), LanguageError> {
    match &expr.kind {
        ExprKind::Symbol(name) => Ok((Vec::new(), Some(name.clone()))),
        ExprKind::List(parameter_forms) => {
            let mut parameters = Vec::with_capacity(parameter_forms.len());
            let mut unique = HashSet::new();
            for parameter in parameter_forms.iter() {
                let ExprKind::Symbol(name) = &parameter.kind else {
                    return Err(LanguageError::new(
                        ErrorKind::InvalidForm,
                        "lambda parameter must be a symbol · параметр lambda має бути символом · lambda-Parameter muss ein Symbol sein",
                        parameter.span,
                    ));
                };
                if !unique.insert(name.clone()) {
                    return Err(LanguageError::new(
                        ErrorKind::InvalidForm,
                        format!("duplicate lambda parameter · повторний параметр lambda · doppelter lambda-Parameter: {name}"),
                        parameter.span,
                    ));
                }
                parameters.push(name.clone());
            }
            Ok((parameters, None))
        }
        ExprKind::Pair(_, _) => {
            let mut parameters = Vec::new();
            let mut unique = HashSet::new();
            let mut current: &Expr = expr;
            let rest = loop {
                match &current.kind {
                    ExprKind::Pair(head, tail) => {
                        let ExprKind::Symbol(name) = &head.kind else {
                            return Err(LanguageError::new(
                                ErrorKind::InvalidForm,
                                "lambda parameter must be a symbol · параметр lambda має бути символом · lambda-Parameter muss ein Symbol sein",
                                head.span,
                            ));
                        };
                        if !unique.insert(name.clone()) {
                            return Err(LanguageError::new(
                                ErrorKind::InvalidForm,
                                format!("duplicate lambda parameter · повторний параметр lambda · doppelter lambda-Parameter: {name}"),
                                head.span,
                            ));
                        }
                        parameters.push(name.clone());
                        current = tail;
                    }
                    ExprKind::Symbol(name) => {
                        if !unique.insert(name.clone()) {
                            return Err(LanguageError::new(
                                ErrorKind::InvalidForm,
                                format!("duplicate lambda parameter · повторний параметр lambda · doppelter lambda-Parameter: {name}"),
                                current.span,
                            ));
                        }
                        break name.clone();
                    }
                    _ => {
                        return Err(LanguageError::new(
                            ErrorKind::InvalidForm,
                            "rest parameter must be a symbol · rest-параметр має бути символом · Rest-Parameter muss ein Symbol sein",
                            current.span,
                        ))
                    }
                }
            };
            Ok((parameters, Some(rest)))
        }
        _ => Err(LanguageError::new(
            ErrorKind::InvalidForm,
            "lambda parameters must be a list, dotted list, or symbol · параметри lambda мають бути списком, dotted-списком або символом · lambda-Parameter müssen eine Liste, Dotted-Liste oder ein Symbol sein",
            expr.span,
        )),
    }
}

pub(super) fn create_lambda(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    if arguments.len() < 2 {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "lambda expects parameters and a body · lambda очікує параметри й тіло · lambda erwartet Parameter und einen Rumpf",
            span,
        ));
    }
    let (parameters, rest) = parse_lambda_list(&arguments[0])?;
    Ok(Value::Closure(Rc::new(Closure {
        parameters,
        rest,
        body: arguments[1..].into(),
        environment: environment.clone(),
    })))
}

/// Shared by `apply`/`apply_macro`: exact arity when there's no rest
/// parameter, "at least this many" when there is.
/// Спільне для `apply`/`apply_macro`: точна арність, якщо нема rest-
/// параметра, "щонайменше стільки" — якщо є.
fn check_arity(
    label: &str,
    fixed: usize,
    has_rest: bool,
    received: usize,
    span: Span,
) -> Result<(), LanguageError> {
    let arity_ok = if has_rest { received >= fixed } else { received == fixed };
    if arity_ok {
        return Ok(());
    }
    let expected = if has_rest {
        format!("at least / щонайменше / mindestens {fixed}")
    } else {
        fixed.to_string()
    };
    Err(LanguageError::new(
        ErrorKind::Arity,
        format!(
            "{label}: expected / очікувалося / erwartet {expected}; received / отримано / erhalten {received}"
        ),
        span,
    ))
}

pub(super) fn apply(
    function: Value,
    arguments: &[Expr],
    calling_environment: &Environment,
    span: Span,
) -> Result<EvalStep, LanguageError> {
    let Value::Closure(ref closure) = function else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "expression is not callable · вираз не можна викликати · Ausdruck ist nicht aufrufbar",
            span,
        ));
    };
    check_arity("lambda", closure.parameters.len(), closure.rest.is_some(), arguments.len(), span)?;

    // Arguments belong to the caller; parameters belong to the captured lexical frame.
    // Аргументи належать виклику, а параметри — захопленому лексичному фрейму.
    // Argumente gehören zum Aufrufer, Parameter zum erfassten lexikalischen Frame.
    let local_environment = closure.environment.child();
    for (parameter, argument) in closure.parameters.iter().zip(arguments.iter()) {
        let value = evaluate(argument, calling_environment)?;
        local_environment.define(parameter.clone(), value);
    }
    if let Some(rest_name) = &closure.rest {
        let mut rest_values = Vec::with_capacity(arguments.len() - closure.parameters.len());
        for argument in &arguments[closure.parameters.len()..] {
            rest_values.push(evaluate(argument, calling_environment)?);
        }
        local_environment.define(rest_name.clone(), Value::list(rest_values));
    }
    let last = last_body_expression(&closure.body, &local_environment, span)?;
    // Tail positions become data for the evaluator loop instead of recursive Rust calls.
    // Хвостові позиції стають даними для циклу evaluator, а не рекурсивними викликами Rust.
    // Tail-Positionen werden zu Daten für die Evaluator-Schleife statt zu rekursiven Rust-Aufrufen.
    Ok(EvalStep::TailCall {
            expression: last.clone(),
            environment: local_environment,
    })
}

/// Runs every body expression except the last for its side effects, then returns
/// the last one for the caller to evaluate in tail position. `create_lambda` always
/// builds a non-empty body, but this returns a `LanguageError` instead of panicking
/// so a future invariant change degrades gracefully rather than crashing the process.
/// Виконує всі вирази тіла, крім останнього, заради побічних ефектів, і повертає
/// останній, щоб виклик обчислив його в хвостовій позиції. `create_lambda` завжди
/// будує непорожнє тіло, але тут повертається `LanguageError`, а не паніка, щоб
/// майбутня зміна інваріанту деградувала плавно, а не аварійно завершувала процес.
/// Führt alle Rumpf-Ausdrücke außer dem letzten wegen ihrer Seiteneffekte aus und
/// gibt den letzten für die Auswertung in Tail-Position zurück. `create_lambda` baut
/// stets einen nicht leeren Rumpf, dennoch wird hier ein `LanguageError` statt eines
/// Panics zurückgegeben, damit eine künftige Invariantenänderung sanft statt abstürzend degradiert.
fn last_body_expression<'a>(
    body: &'a [Expr],
    environment: &Environment,
    span: Span,
) -> Result<&'a Expr, LanguageError> {
    let Some((last, leading)) = body.split_last() else {
        return Err(LanguageError::new(
            ErrorKind::InvalidForm,
            "lambda body must not be empty · тіло lambda не може бути порожнім · lambda-Rumpf darf nicht leer sein",
            span,
        ));
    };
    for expression in leading {
        evaluate(expression, environment)?;
    }
    Ok(last)
}

pub(super) fn apply_macro(
    closure: Rc<Closure>,
    arguments: &[Expr],
    calling_environment: &Environment,
    span: Span,
) -> Result<EvalStep, LanguageError> {
    check_arity("defmacro", closure.parameters.len(), closure.rest.is_some(), arguments.len(), span)?;

    let local_environment = closure.environment.child();
    for (parameter, argument) in closure.parameters.iter().zip(arguments.iter()) {
        let value = quoted(argument); // Do NOT evaluate arguments
        local_environment.define(parameter.clone(), value);
    }
    if let Some(rest_name) = &closure.rest {
        let rest_values: Vec<Value> = arguments[closure.parameters.len()..]
            .iter()
            .map(quoted) // Do NOT evaluate arguments
            .collect();
        local_environment.define(rest_name.clone(), Value::list(rest_values));
    }

    let last = last_body_expression(&closure.body, &local_environment, span)?;

    let expanded_value = evaluate(last, &local_environment)?;
    let expanded_expr = value_to_expr(expanded_value, span)?;

    Ok(EvalStep::TailCall {
        expression: expanded_expr,
        environment: calling_environment.clone(),
    })
}

// `pub(super)`, not private: `special_forms::evaluate_eval` reuses this same
// data->code conversion for `eval`, rather than duplicating the cons-cell
// walk that macro expansion already needed.
// `pub(super)`, не приватна: `special_forms::evaluate_eval` перевикористовує
// це саме перетворення дані->код для `eval`, замість дублювання обходу
// cons-комірок, який уже був потрібен для розгортання макросів.
// `pub(super)`, nicht privat: `special_forms::evaluate_eval` nutzt dieselbe
// Daten->Code-Umwandlung für `eval` wieder, statt den Cons-Zellen-Durchlauf
// zu duplizieren, den die Makro-Expansion bereits brauchte.
pub(super) fn value_to_expr(value: Value, span: Span) -> Result<Expr, LanguageError> {
    let kind = match &value {
        Value::Nil => ExprKind::List(Rc::new([])),
        Value::Bool(true) => ExprKind::Symbol("t".into()),
        Value::Bool(false) => ExprKind::List(Rc::new([])),
        Value::Number(number, exactness) => ExprKind::Number(*number, *exactness),
        Value::Rational(rational) => ExprKind::Rational(rational.clone()),
        Value::String(val) => ExprKind::String(val.clone()),
        Value::Symbol(symbol) => ExprKind::Symbol(symbol.clone()),
        Value::Pair(_, _) => {
            let mut items = Vec::new();
            let mut current = value.clone();
            loop {
                match &current {
                    Value::Pair(h, t) => {
                        items.push(value_to_expr((**h).clone(), span)?);
                        current = (**t).clone();
                    }
                    Value::Nil => break,
                    _ => {
                        return Err(LanguageError::new(
                            ErrorKind::InvalidForm,
                            "macros must return proper lists · макроси повинні повертати правильні списки · Makros müssen korrekte Listen zurückgeben",
                            span,
                        ))
                    }
                }
            }
            ExprKind::List(items.into())
        }
        Value::Closure(_) | Value::Macro(_) => {
            return Err(LanguageError::new(
                ErrorKind::InvalidForm,
                "macros cannot return closures or macros · макроси не можуть повертати замикання або макроси · Makros dürfen keine Closures oder Makros zurückgeben",
                span,
            ))
        }
    };
    Ok(Expr { kind, span })
}

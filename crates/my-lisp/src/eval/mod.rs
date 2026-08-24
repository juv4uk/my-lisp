//! Evaluator entry points and the special-form dispatcher.
//! Tochky vkhodu evaluator i dyspetcher spetsialnykh form.
//! Einstiegspunkte des Evaluators und der Sonderformen-Dispatcher.
//!
//! The evaluator is split by concern: this module owns the trampoline loop and
//! dispatch table, `arithmetic` owns exact/inexact number handling, `special_forms`
//! owns the McCarthy primitives plus `def`/`defmacro`/`cond`, and `closures` owns
//! `lambda` construction and function/macro application.
//! Evaluator rozdileno za vidpovidalnistiu: tsei modul volodiie tsyklom trampoline
//! ta tablytseiu dyspetcheryzatsii, `arithmetic` — tochnymy/netochnymy chyslamy,
//! `special_forms` — prymityvamy Makkarti ta `def`/`defmacro`/`cond`, a `closures` —
//! pobudovoiu `lambda` i zastosuvanniam funktsii/makrosiv.
//! Der Evaluator ist nach Zuständigkeit aufgeteilt: dieses Modul besitzt die
//! Trampolin-Schleife und die Dispatch-Tabelle, `arithmetic` die exakte/inexakte
//! Zahlenverarbeitung, `special_forms` die McCarthy-Primitive sowie `def`/`defmacro`/
//! `cond`, und `closures` den Bau von `lambda` und die Anwendung von Funktionen/Makros.

mod arithmetic;
pub(crate) mod builtins;
mod capabilities;
mod closures;
mod special_forms;

// Facade re-export: host adapters (LSP, CLI tooling) consume the canonical
// JSON decoder as a plain function without going through eval.
pub use capabilities::{capability_installed, installed_capabilities, register_capability, unregister_capability};
pub use special_forms::{exact_arity, json::parse_json};

use crate::{parse, Environment, ErrorKind, Expr, ExprKind, LanguageError, Session, Span, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct EvalResult {
    pub value: Value,
    pub output: Vec<String>,
}

pub fn eval_parsed_expressions(
    expressions: &[Expr],
    session: &mut Session,
) -> Result<EvalResult, LanguageError> {
    let mut value = Value::Nil;
    for expression in expressions {
        value = evaluate(expression, &session.environment)?;
    }
    Ok(EvalResult {
        value,
        output: session.environment.output_snapshot(),
    })
}

/// Parsed-source twin of `eval_program_incremental`: output carries only
/// the lines printed during THIS call.
pub fn eval_parsed_expressions_incremental(
    expressions: &[Expr],
    session: &mut Session,
) -> Result<EvalResult, LanguageError> {
    session.environment.output_take_new();
    let mut value = Value::Nil;
    for expression in expressions {
        value = evaluate(expression, &session.environment)?;
    }
    Ok(EvalResult {
        value,
        output: session.environment.output_take_new(),
    })
}

/// Evaluates source string by parsing it and running the resulting expressions.
/// Obchysliuie syrtsevyi riadok cherez parsynh ta vykonannia otrymanykh vyraziv.
/// Wertet den Quelltext durch Parsing und Ausführung der Ausdrücke aus.
pub fn eval_program(source: &str, session: &mut Session) -> Result<EvalResult, LanguageError> {
    let expressions = parse(source)?;
    eval_parsed_expressions(&expressions, session)
}

/// Same as `eval_program`, but `EvalResult::output` carries only the
/// lines printed during THIS call — O(new output) instead of re-cloning
/// the whole session transcript. For hot hosts (REPL loop, LSP, swarm
/// TCP oracle) this removes the quadratic cost of long sessions; full
/// history remains available via `Environment::output_snapshot`.
pub fn eval_program_incremental(
    source: &str,
    session: &mut Session,
) -> Result<EvalResult, LanguageError> {
    session.environment.output_take_new(); // drop anything printed before this call
    let expressions = parse(source)?;
    let mut value = Value::Nil;
    for expression in &expressions {
        value = evaluate(expression, &session.environment)?;
    }
    Ok(EvalResult {
        value,
        output: session.environment.output_take_new(),
    })
}

pub(crate) enum EvalStep {
    Value(Value),
    TailCall {
        expression: Expr,
        environment: Environment,
    },
}

pub(crate) fn invoke_value(
    function: &Value,
    arguments: &[Value],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    match function {
        Value::Builtin(builtin) => (builtin.func)(arguments, environment, span),
        Value::Closure(closure) => closures::apply_values(closure.clone(), arguments, span),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "numeric-buffer-map expects a callable function",
            span,
        )),
    }
}

pub fn evaluate(expression: &Expr, environment: &Environment) -> Result<Value, LanguageError> {
    let (mut owned_expression, mut owned_environment) =
        match evaluate_step(expression, environment)? {
            EvalStep::Value(value) => return Ok(value),
            EvalStep::TailCall {
                expression,
                environment,
            } => (expression, environment),
        };

    loop {
        match evaluate_step(&owned_expression, &owned_environment)? {
            EvalStep::Value(value) => return Ok(value),
            EvalStep::TailCall {
                expression: next,
                environment: next_environment,
            } => {
                owned_expression = next;
                owned_environment = next_environment;
            }
        }
    }
}

pub(crate) fn evaluate_step(
    expression: &Expr,
    environment: &Environment,
) -> Result<EvalStep, LanguageError> {
    match &expression.kind {
        ExprKind::Number(number, exactness) => Ok(EvalStep::Value(Value::Number(*number, *exactness))),
        ExprKind::Rational(rational) => Ok(EvalStep::Value(Value::Rational(rational.clone()))),
        ExprKind::NumericBuffer(buffer) => Ok(EvalStep::Value(Value::NumericBuffer(buffer.clone()))),
        ExprKind::String(value) => Ok(EvalStep::Value(Value::String(value.clone()))),
        ExprKind::Symbol(symbol) => environment.get(symbol).map(EvalStep::Value).ok_or_else(|| {
            LanguageError::new(
                ErrorKind::UnknownSymbol,
                format!("unknown symbol · nevidomyi symvol · unbekanntes Symbol: {symbol}"),
                expression.span,
            )
        }),
        ExprKind::List(items) if items.is_empty() => Ok(EvalStep::Value(Value::Nil)),
        ExprKind::List(items) => evaluate_list(items, environment, expression.span),
        ExprKind::Pair(_, _) => Err(LanguageError::new(
            ErrorKind::InvalidForm,
            "a dotted pair is not executable code · dotted-para ne ye vykonuvanym kodom · ein Dotted Pair ist kein ausführbarer Code",
            expression.span,
        )),
    }
}

fn evaluate_list(
    items: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<EvalStep, LanguageError> {
    let arguments = &items[1..];
    // Special forms stay explicit because they control which arguments are evaluated.
    // Spetsialni formy lyshaiutsia yavnymy, bo vony keruiut obchyslenniam arhumentiv.
    // Sonderformen bleiben explizit, weil sie die Auswertung ihrer Argumente steuern.
    match items[0].kind.as_symbol() {
        Some("quote") => {
            special_forms::exact_arity("quote", arguments, 1, span)?;
            let value = special_forms::quoted(&arguments[0])?;
            Ok(EvalStep::Value(value))
        }
        Some("lambda") => closures::create_lambda(arguments, environment, span).map(EvalStep::Value),
        Some("def") => {
            special_forms::evaluate_definition(arguments, environment, span).map(EvalStep::Value)
        }
        Some("defmacro") => {
            special_forms::evaluate_defmacro(arguments, environment, span).map(EvalStep::Value)
        }
        Some("cond") => special_forms::evaluate_cond(arguments, environment, span),
        Some("print") => {
            special_forms::evaluate_print(arguments, environment, span).map(EvalStep::Value)
        }
        Some("princ") => {
            special_forms::evaluate_princ(arguments, environment, span).map(EvalStep::Value)
        }
        Some("write-to-string") => {
            special_forms::evaluate_write_to_string(arguments, environment, span).map(EvalStep::Value)
        }
        Some("read") => {
            special_forms::evaluate_read(arguments, environment, span).map(EvalStep::Value)
        }
        Some("eval") => {
            special_forms::evaluate_eval(arguments, environment, span).map(EvalStep::Value)
        }
        Some("string-append") => {
            special_forms::evaluate_string_append(arguments, environment, span).map(EvalStep::Value)
        }
        Some("string<?") => {
            special_forms::evaluate_string_less_than(arguments, environment, span).map(EvalStep::Value)
        }
        Some("read-all") => {
            special_forms::evaluate_read_all(arguments, environment, span).map(EvalStep::Value)
        }
        Some("string?") => {
            special_forms::evaluate_string_predicate(arguments, environment, span).map(EvalStep::Value)
        }
        Some("symbol->string") => {
            special_forms::evaluate_symbol_to_string(arguments, environment, span).map(EvalStep::Value)
        }
        Some("string->symbol") => {
            special_forms::evaluate_string_to_symbol(arguments, environment, span).map(EvalStep::Value)
        }
        Some("string-first") => {
            special_forms::evaluate_string_first(arguments, environment, span).map(EvalStep::Value)
        }
        Some("string-rest") => {
            special_forms::evaluate_string_rest(arguments, environment, span).map(EvalStep::Value)
        }
        Some("sha256-hex") => {
            special_forms::evaluate_sha256_hex(arguments, environment, span).map(EvalStep::Value)
        }
        Some("json-parse") => {
            special_forms::json::evaluate_json_parse(arguments, environment, span).map(EvalStep::Value)
        }
        // NOTE: abs/min/max/min-list/max-list are first-class builtins
        // (eval/builtins.rs), NOT special forms — resolving them through the
        // environment keeps the ratified lexical-shadowing contract intact:
        // user `(def min ...)` must win over the builtin.
        // (eval/builtins.rs), NOT special forms — resolving them through the
        // environment keeps the ratified lexical-shadowing contract intact:
        // user `(def min ...)` must win over the builtin.
        // The same applies to the vector family (make-vector, vector,
        // vector-length, vector-ref, vector-set!) — registered in
        // eval/builtins.rs below.

        // Binding the operator symbol in the pattern avoids re-deriving it with
        // an `.expect()`, so a future refactor of `as_symbol` cannot turn this into a panic.
        // Zakhoplennia symvola operatora priamo v paterni unykaie povtornoho `.expect()`,
        // tozh maibutnia zmina `as_symbol` ne zmozhe peretvoryty tse na paniku.
        // Das Binden des Operator-Symbols im Pattern vermeidet ein erneutes `.expect()`,
        // sodass eine spätere Änderung an `as_symbol` dies nicht zu einem Panic machen kann.
        _ => {
            // Host-capability fallback (see eval/capabilities.rs): the
            // canonical core registers nothing here, so an unregistered
            // name still falls through to ordinary application and fails
            // `UnknownSymbol` as always.
            if let Some(name) = items[0].kind.as_symbol() {
                if let Some(result) =
                    capabilities::dispatch_capability(name, arguments, environment, span)
                {
                    return result;
                }
            }
            let function = evaluate(&items[0], environment)?;
            match &function {
                // contract 2.1: builtins are ordinary callable values --
                // arguments arrive pre-evaluated.
                Value::Builtin(builtin) => {
                    let mut values = Vec::with_capacity(arguments.len());
                    for argument in arguments {
                        values.push(evaluate(argument, environment)?);
                    }
                    (builtin.func)(&values, environment, span).map(EvalStep::Value)
                }
                Value::Macro(closure) => {
                    closures::apply_macro(closure.clone(), arguments, environment, span)
                }
                _ => closures::apply(function, arguments, environment, span),
            }
        }
    }
}

trait ExprKindExt {
    fn as_symbol(&self) -> Option<&str>;
}

impl ExprKindExt for ExprKind {
    fn as_symbol(&self) -> Option<&str> {
        match self {
            ExprKind::Symbol(symbol) => Some(symbol),
            _ => None,
        }
    }
}

#[cfg(test)]
mod single_pass_eval_tests {
    use super::*;

    #[test]
    fn single_pass_eval_parsed_expressions_evaluates_preparsed_ast() {
        let source = "(def x (/ 1 3)) (cons x (quote ()))";
        let forms = parse(source).expect("parsing should succeed");
        let mut session = Session::default();
        let result = eval_parsed_expressions(&forms, &mut session)
            .expect("eval_parsed_expressions should succeed");
        assert_eq!(result.value.to_string(), "(1/3)");
    }

    #[test]
    fn macros_expand_and_evaluate_correctly() {
        // `list` moved to lib/core.my (2026-08-09) — this test deliberately
        // doesn't load it, to keep exercising defmacro/macro-expansion in
        // isolation from the bootstrap library, so `cons`/quote build the
        // expansion by hand instead.
        let source = r#"
            (defmacro unless (condition body)
                (cons (quote cond)
                    (cons (cons condition (cons (quote ()) (quote ())))
                    (cons (cons (quote t) (cons body (quote ()))) (quote ())))))
            (unless () (quote success))
        "#;
        let mut session = Session::default();
        let result = eval_program(source, &mut session).expect("eval should succeed");
        assert_eq!(result.value.to_string(), "success");
    }

    #[test]
    fn macro_expansion_preserves_exact_rationals() {
        let source = r#"
            (defmacro half-of-third ()
                (/ 1 6))
            (half-of-third)
        "#;
        let mut session = Session::default();
        let result = eval_program(source, &mut session).expect("eval should succeed");
        assert_eq!(result.value.to_string(), "1/6");
    }
}

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
mod closures;
mod special_forms;

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

/// Evaluates source string by parsing it and running the resulting expressions.
/// Obchysliuie syrtsevyi riadok cherez parsynh ta vykonannia otrymanykh vyraziv.
/// Wertet den Quelltext durch Parsing und Ausführung der Ausdrücke aus.
pub fn eval_program(source: &str, session: &mut Session) -> Result<EvalResult, LanguageError> {
    let expressions = parse(source)?;
    eval_parsed_expressions(&expressions, session)
}

pub(crate) enum EvalStep {
    Value(Value),
    TailCall {
        expression: Expr,
        environment: Environment,
    },
}

pub(crate) fn evaluate(expression: &Expr, environment: &Environment) -> Result<Value, LanguageError> {
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
            Ok(EvalStep::Value(special_forms::quoted(&arguments[0])))
        }
        Some("lambda") => closures::create_lambda(arguments, environment, span).map(EvalStep::Value),
        Some("def") => {
            special_forms::evaluate_definition(arguments, environment, span).map(EvalStep::Value)
        }
        Some("defmacro") => {
            special_forms::evaluate_defmacro(arguments, environment, span).map(EvalStep::Value)
        }
        Some("cond") => special_forms::evaluate_cond(arguments, environment, span),
        Some("atom") => {
            special_forms::exact_arity("atom", arguments, 1, span)?;
            Ok(EvalStep::Value(Value::Bool(
                evaluate(&arguments[0], environment)?.is_atom(),
            )))
        }
        Some("eq") => special_forms::evaluate_eq(arguments, environment, span).map(EvalStep::Value),
        Some("car") => special_forms::evaluate_car(arguments, environment, span).map(EvalStep::Value),
        Some("cdr") => special_forms::evaluate_cdr(arguments, environment, span).map(EvalStep::Value),
        Some("cons") => {
            special_forms::evaluate_cons(arguments, environment, span).map(EvalStep::Value)
        }
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
        Some("load") => {
            special_forms::evaluate_load(arguments, environment, span).map(EvalStep::Value)
        }
        Some("read-file") => {
            special_forms::evaluate_read_file(arguments, environment, span).map(EvalStep::Value)
        }
        Some("write-file") => {
            special_forms::evaluate_write_file(arguments, environment, span).map(EvalStep::Value)
        }
        Some("read-file-bytes") => {
            special_forms::evaluate_read_file_bytes(arguments, environment, span).map(EvalStep::Value)
        }
        Some("write-file-bytes") => {
            special_forms::evaluate_write_file_bytes(arguments, environment, span).map(EvalStep::Value)
        }
        Some("string-append") => {
            special_forms::evaluate_string_append(arguments, environment, span).map(EvalStep::Value)
        }
        Some("string<?") => {
            special_forms::evaluate_string_less_than(arguments, environment, span).map(EvalStep::Value)
        }
        Some("tcp-connect") => {
            special_forms::evaluate_tcp_connect(arguments, environment, span).map(EvalStep::Value)
        }
        Some("tcp-listen") => {
            special_forms::evaluate_tcp_listen(arguments, environment, span).map(EvalStep::Value)
        }
        Some("tcp-accept") => {
            special_forms::evaluate_tcp_accept(arguments, environment, span).map(EvalStep::Value)
        }
        Some("tcp-read") => {
            special_forms::evaluate_tcp_read(arguments, environment, span).map(EvalStep::Value)
        }
        Some("tcp-write") => {
            special_forms::evaluate_tcp_write(arguments, environment, span).map(EvalStep::Value)
        }
        Some("tcp-close") => {
            special_forms::evaluate_tcp_close(arguments, environment, span).map(EvalStep::Value)
        }
        Some("process-run") => {
            special_forms::evaluate_process_run(arguments, environment, span).map(EvalStep::Value)
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
        Some("/") => arithmetic::evaluate_division(arguments, environment, span).map(EvalStep::Value),
        // Binding the operator symbol in the pattern avoids re-deriving it with
        // an `.expect()`, so a future refactor of `as_symbol` cannot turn this into a panic.
        // Zakhoplennia symvola operatora priamo v paterni unykaie povtornoho `.expect()`,
        // tozh maibutnia zmina `as_symbol` ne zmozhe peretvoryty tse na paniku.
        // Das Binden des Operator-Symbols im Pattern vermeidet ein erneutes `.expect()`,
        // sodass eine spätere Änderung an `as_symbol` dies nicht zu einem Panic machen kann.
        Some(operator @ ("+" | "-" | "*")) => {
            arithmetic::evaluate_arithmetic(operator, arguments, environment, span).map(EvalStep::Value)
        }
        Some(operator @ ("<" | ">" | "=")) => {
            arithmetic::evaluate_comparison(operator, arguments, environment, span).map(EvalStep::Value)
        }
        _ => {
            let function = evaluate(&items[0], environment)?;
            match &function {
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
        let source = "(def x (/ 1 3)) (cons x '())";
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
                (cons 'cond
                    (cons (cons condition (cons '() '()))
                    (cons (cons 't (cons body '())) '()))))
            (unless () 'success)
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

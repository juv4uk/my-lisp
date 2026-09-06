//! Evaluator entry points and the special-form dispatcher.
//! Tochky vkhodu evaluator i dyspetcher spetsialnykh form.
//! Einstiegspunkte des Evaluators und der Sonderformen-Dispatcher.
//!
//! The evaluator is split by concern: this module owns the trampoline loop and
//! dispatch table, `arithmetic` owns exact/inexact number handling, `special_forms`
//! owns the McCarthy primitives plus `def`/`defmacro`/`cond`, and `closures` owns
//! `lambda` construction and function/macro application.
pub(crate) use special_forms::digest::sha256 as digest_sha256;

mod arithmetic;
pub(crate) mod builtins;
mod canon;
mod capabilities;
mod closures;
mod special_forms;

pub use capabilities::{
    capability_installed, installed_capabilities, register_capability, unregister_capability,
};
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

pub fn eval_program(source: &str, session: &mut Session) -> Result<EvalResult, LanguageError> {
    let expressions = parse(source)?;
    eval_parsed_expressions(&expressions, session)
}

pub fn eval_program_incremental(
    source: &str,
    session: &mut Session,
) -> Result<EvalResult, LanguageError> {
    session.environment.output_take_new();
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
        ExprKind::Symbol(symbol) => environment
            .get(symbol)
            // Canonical fallback is semantic, not lexical: resolve the
            // spelling directly to CANON.  Rebinding `car` can no longer
            // mutate what `перше` or `ādi` mean.
            .or_else(|| canon::value_for_surface(symbol))
            .map(EvalStep::Value)
            .ok_or_else(|| {
                LanguageError::new(
                    ErrorKind::UnknownSymbol,
                    format!("unknown symbol · nevidomyi symvol · unbekanntes Symbol: {symbol}"),
                    expression.span,
                )
            }),
        // Canon 0 is a value in its own right.  `Value::Nil` is only its
        // current Rust representation, not its public canonical name.
        ExprKind::List(items) if items.is_empty() => Ok(EvalStep::Value(
            canon::ground_value(canon::CanonicalIdentity::EmptyList)
                .expect("Canon 0 must always materialize"),
        )),
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
    match items[0].kind.as_symbol() {
        Some(name @ ("quote" | "як-є" | "svarūpa")) => {
            special_forms::exact_arity(name, arguments, 1, span)?;
            let value = special_forms::quoted(&arguments[0])?;
            Ok(EvalStep::Value(value))
        }
        Some("lambda") => {
            closures::create_lambda(arguments, environment, span).map(EvalStep::Value)
        }
        Some("def") => {
            special_forms::evaluate_definition(arguments, environment, span).map(EvalStep::Value)
        }
        Some("defmacro") => {
            special_forms::evaluate_defmacro(arguments, environment, span).map(EvalStep::Value)
        }
        Some("cond" | "за-умовою" | "anukrama") => {
            special_forms::evaluate_cond(arguments, environment, span)
        }
        Some("print") => {
            special_forms::evaluate_print(arguments, environment, span).map(EvalStep::Value)
        }
        Some("princ") => {
            special_forms::evaluate_princ(arguments, environment, span).map(EvalStep::Value)
        }
        Some("write-to-string") => {
            special_forms::evaluate_write_to_string(arguments, environment, span)
                .map(EvalStep::Value)
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
        Some("string<?") => special_forms::evaluate_string_less_than(arguments, environment, span)
            .map(EvalStep::Value),
        Some("read-all") => {
            special_forms::evaluate_read_all(arguments, environment, span).map(EvalStep::Value)
        }
        Some("string?") => special_forms::evaluate_string_predicate(arguments, environment, span)
            .map(EvalStep::Value),
        Some("symbol->string") => {
            special_forms::evaluate_symbol_to_string(arguments, environment, span)
                .map(EvalStep::Value)
        }
        Some("string->symbol") => {
            special_forms::evaluate_string_to_symbol(arguments, environment, span)
                .map(EvalStep::Value)
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
            special_forms::json::evaluate_json_parse(arguments, environment, span)
                .map(EvalStep::Value)
        }
        _ => {
            if let Some(name) = items[0].kind.as_symbol() {
                if let Some(result) =
                    capabilities::dispatch_capability(name, arguments, environment, span)
                {
                    return result;
                }
            }
            let function = evaluate(&items[0], environment)?;
            match &function {
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

    #[test]
    fn canon_zero_empty_list_evaluates_directly() {
        let mut session = Session::default();
        let result = eval_program("()", &mut session).expect("Canon 0 should evaluate");
        assert_eq!(result.value.to_string(), "()");
    }

    #[test]
    fn ukrainian_canonical_surface_executes_the_core() {
        let source = r#"
            (за-умовою
              ((атом? (як-є кіт))
               (перше
                 (сполучити
                   (як-є груша)
                   (сполучити (як-є слива) ()))))
              (t (як-є помилка)))
        "#;
        let mut session = Session::default();
        let result = eval_program(source, &mut session)
            .expect("Ukrainian canonical surface should evaluate");
        assert_eq!(result.value.to_string(), "груша");
    }

    #[test]
    fn ukrainian_rest_obeys_proper_list_semantics() {
        let mut session = Session::default();
        let result = eval_program("(решта (як-є (яблуко груша слива)))", &mut session)
            .expect("решта should return the structural remainder");
        assert_eq!(result.value.to_string(), "(груша слива)");
    }

    #[test]
    fn ukrainian_double_projection_reads_the_tree() {
        let mut session = Session::default();
        let result = eval_program(
            "(перше (решта (як-є (яблуко груша слива))))",
            &mut session,
        )
        .expect("canonical composition should evaluate");
        assert_eq!(result.value.to_string(), "груша");
    }

    #[test]
    fn sanskrit_canonical_surface_executes_the_same_core() {
        let source = r#"
            (anukrama
              ((aṇu (svarūpa phalam))
               (ādi
                 (saṃyuj
                   (svarūpa prathama)
                   (saṃyuj (svarūpa śeṣaḥ) ()))))
              (t (svarūpa doṣa)))
        "#;
        let mut session = Session::default();
        let result = eval_program(source, &mut session)
            .expect("Sanskrit canonical surface should evaluate");
        assert_eq!(result.value.to_string(), "prathama");
    }

    #[test]
    fn canonical_builtin_spelling_remains_shadowable() {
        let source = "(def перше (lambda (x) (як-є затінено))) (перше 42)";
        let mut session = Session::default();
        let result = eval_program(source, &mut session)
            .expect("canonical builtin surface should preserve lexical shadowing");
        assert_eq!(result.value.to_string(), "затінено");
    }

    #[test]
    fn rebinding_historical_name_does_not_mutate_other_surfaces() {
        let source = r#"
            (def car (lambda (x) (як-є зламано)))
            (перше (сполучити 1 2))
        "#;
        let mut session = Session::default();
        let result = eval_program(source, &mut session)
            .expect("canonical identity must outlive historical shadowing");
        assert_eq!(result.value.to_string(), "1");
    }
}

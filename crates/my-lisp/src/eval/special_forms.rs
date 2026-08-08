//! The McCarthy primitives (`eq`, `car`, `cdr`, `cons`, `cond`, `quote`'s helper),
//! plus `def`, `defmacro`, and `list`.
//! Примітиви Маккарті (`eq`, `car`, `cdr`, `cons`, `cond`, помічник `quote`),
//! а також `def`, `defmacro` і `list`.
//! Die McCarthy-Primitive (`eq`, `car`, `cdr`, `cons`, `cond`, Helfer für `quote`),
//! sowie `def`, `defmacro` und `list`.

use super::{closures, evaluate, evaluate_step, EvalStep};
use crate::{Environment, ErrorKind, Expr, ExprKind, LanguageError, Span, Value};
use std::rc::Rc;

/// `print` evaluates its one argument and appends its `Display` text to the
/// session-wide output transcript (`Environment::print`) rather than writing
/// to stdout/stderr directly — the crate stays capability-free, and it's the
/// host (`my-lisp-cli`, `my-lisp-wasm`) that decides where `EvalResult.output`
/// actually goes. Returns the evaluated value, so `(print x)` composes like
/// Common Lisp's `print` instead of being a dead end in an expression.
/// `print` обчислює свій єдиний аргумент і додає його `Display`-текст до
/// транскрипту виводу, спільного на сесію (`Environment::print`), а не пише
/// напряму в stdout/stderr — крейт лишається без host-можливостей, і саме
/// host (`my-lisp-cli`, `my-lisp-wasm`) вирішує, куди насправді йде
/// `EvalResult.output`. Повертає обчислене значення, тож `(print x)`
/// компонується, як `print` у Common Lisp, а не є глухим кутом виразу.
/// `print` wertet sein einziges Argument aus und hängt dessen `Display`-Text
/// an das sitzungsweite Ausgabetranskript an (`Environment::print`), statt
/// direkt nach stdout/stderr zu schreiben — das Crate bleibt ohne
/// Host-Fähigkeiten, und der Host (`my-lisp-cli`, `my-lisp-wasm`)
/// entscheidet, wohin `EvalResult.output` tatsächlich geht. Gibt den
/// ausgewerteten Wert zurück, sodass sich `(print x)` wie Common Lisps
/// `print` verketten lässt statt eine Sackgasse im Ausdruck zu sein.
pub(super) fn evaluate_print(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("print", arguments, 1, span)?;
    let value = evaluate(&arguments[0], environment)?;
    environment.print(value.to_string());
    Ok(value)
}

/// `read` is McCarthy's original reader primitive: it turns text into one
/// s-expression of *data*, the same way `'expr` does, without evaluating it —
/// `(eval (read "(+ 1 2)"))` is the read/eval loop written out by hand, in
/// the language itself. Taking a string (not stdin) keeps this
/// capability-free like the rest of the crate; interactive input, if it's
/// ever added, is a separate host-boundary primitive, not this one.
/// `read` — оригінальний reader-примітив Маккарті: перетворює текст на одну
/// s-expression *даних*, так само як `'expr`, без обчислення — `(eval (read
/// "(+ 1 2)"))` це вручну виписаний read/eval цикл самою мовою. Читання з
/// рядка (не stdin) лишає це без host-можливостей, як і решту крейта;
/// інтерактивний ввід, якщо колись з'явиться, — окремий host-межовий
/// примітив, не цей.
/// `read` ist McCarthys ursprüngliches Reader-Primitiv: es macht aus Text
/// eine s-Expression aus *Daten*, genau wie `'expr`, ohne sie auszuwerten —
/// `(eval (read "(+ 1 2)"))` ist die Read/Eval-Schleife von Hand
/// ausgeschrieben, in der Sprache selbst. Eine Zeichenkette (nicht stdin)
/// entgegenzunehmen hält dies ohne Host-Fähigkeiten wie den Rest des
/// Crates; interaktive Eingabe, falls je hinzugefügt, ist ein separates
/// Host-Grenz-Primitiv, nicht dieses.
pub(super) fn evaluate_read(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("read", arguments, 1, span)?;
    // `Value` has a custom `Drop` impl (iterative, for stack-safe deep-list
    // drop), which forbids partially moving a field out of a match on it by
    // value — hence matching on a reference and cloning the cheap `Rc<str>`.
    // `Value` має власний `Drop` (ітеративний, для stack-safe drop глибоких
    // списків), який забороняє частково переміщувати поле з `match` за
    // значенням — тому матчимо через посилання й клонуємо дешевий `Rc<str>`.
    // `Value` hat einen eigenen `Drop`-Impl (iterativ, für stack-sicheres
    // Droppen tiefer Listen), der ein teilweises Herausbewegen eines Feldes
    // aus einem `match` nach Wert verbietet — daher wird über eine Referenz
    // gematcht und der billige `Rc<str>` geklont.
    let evaluated = evaluate(&arguments[0], environment)?;
    let source = match &evaluated {
        Value::String(text) => text.clone(),
        _ => {
            return Err(LanguageError::new(
                ErrorKind::Type,
                "read expects a string · read очікує рядок · read erwartet eine Zeichenkette",
                arguments[0].span,
            ))
        }
    };
    let expressions = crate::parse(&source).map_err(|mut error| {
        error.span = span;
        error
    })?;
    match <[Expr; 1]>::try_from(expressions) {
        Ok([expression]) => Ok(quoted(&expression)),
        Err(expressions) => Err(LanguageError::new(
            ErrorKind::InvalidForm,
            format!(
                "read expects exactly one expression, found {} · read очікує рівно один вираз, знайдено {} · read erwartet genau einen Ausdruck, gefunden {}",
                expressions.len(), expressions.len(), expressions.len()
            ),
            span,
        )),
    }
}

/// `eval` closes the read/eval loop McCarthy's Lisp is built around:
/// evaluates its argument to get a *datum* (typically from `read` or
/// `quote`), then evaluates that datum as code. Reuses `closures::value_to_expr`,
/// the same data->code conversion macro expansion already relies on, rather
/// than duplicating the cons-cell walk. `Closure`/`Macro` values are
/// self-evaluating (returned unchanged) since there's no source syntax for
/// them to convert back into.
/// `eval` замикає read/eval цикл, навколо якого побудовано Lisp Маккарті:
/// обчислює свій аргумент, щоб отримати *дані* (зазвичай від `read` чи
/// `quote`), тоді обчислює ці дані як код. Перевикористовує
/// `closures::value_to_expr` — те саме перетворення дані->код, на яке вже
/// спирається розгортання макросів. Значення `Closure`/`Macro`
/// самообчислювані (повертаються без змін), бо для них немає синтаксису
/// початкового коду, у який можна конвертувати назад.
/// `eval` schließt die Read/Eval-Schleife, um die herum McCarthys Lisp
/// aufgebaut ist: wertet sein Argument aus, um ein *Datum* zu erhalten
/// (typischerweise von `read` oder `quote`), und wertet dieses Datum dann
/// als Code aus. Nutzt `closures::value_to_expr` wieder, dieselbe
/// Daten->Code-Umwandlung, auf die sich die Makro-Expansion bereits
/// stützt. `Closure`/`Macro`-Werte sind selbstauswertend (unverändert
/// zurückgegeben), da es keine Quellsyntax gibt, in die sie zurück
/// konvertiert werden könnten.
pub(super) fn evaluate_eval(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("eval", arguments, 1, span)?;
    let datum = evaluate(&arguments[0], environment)?;
    if matches!(datum, Value::Closure(_) | Value::Macro(_)) {
        return Ok(datum);
    }
    let expression = closures::value_to_expr(datum, span)?;
    evaluate(&expression, environment)
}

pub(super) fn evaluate_definition(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("def", arguments, 2, span)?;
    let ExprKind::Symbol(name) = &arguments[0].kind else {
        return Err(LanguageError::new(
            ErrorKind::InvalidForm,
            "def expects a symbol name · def очікує назву-символ · def erwartet einen Symbolnamen",
            arguments[0].span,
        ));
    };
    let value = evaluate(&arguments[1], environment)?;
    // The shared lexical frame makes recursive definitions visible to their closure after binding.
    // Спільний лексичний фрейм робить рекурсивне визначення видимим замиканню після зв’язування.
    // Der gemeinsame lexikalische Frame macht rekursive Definitionen nach der Bindung für ihre Closure sichtbar.
    environment.define(name.clone(), value.clone());
    Ok(value)
}

pub(super) fn evaluate_defmacro(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    if arguments.len() < 2 {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "defmacro expects a name, parameters, and a body · defmacro очікує назву, параметри й тіло · defmacro erwartet einen Namen, Parameter und einen Rumpf",
            span,
        ));
    }
    let ExprKind::Symbol(name) = &arguments[0].kind else {
        return Err(LanguageError::new(
            ErrorKind::InvalidForm,
            "defmacro expects a symbol name · defmacro очікує назву-символ · defmacro erwartet einen Symbolnamen",
            arguments[0].span,
        ));
    };
    let closure_val = closures::create_lambda(&arguments[1..], environment, span)?;
    let Value::Closure(closure) = &closure_val else {
        unreachable!("create_lambda always returns Closure")
    };
    let macro_val = Value::Macro(closure.clone());
    environment.define(name.clone(), macro_val.clone());
    Ok(macro_val)
}

pub(super) fn evaluate_list_func(
    arguments: &[Expr],
    environment: &Environment,
    _span: Span,
) -> Result<Value, LanguageError> {
    let mut values = Vec::with_capacity(arguments.len());
    for argument in arguments {
        values.push(evaluate(argument, environment)?);
    }
    Ok(Value::list(values))
}

pub(super) fn evaluate_eq(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("eq", arguments, 2, span)?;
    let left = evaluate(&arguments[0], environment)?;
    let right = evaluate(&arguments[1], environment)?;
    if !left.is_atom() || !right.is_atom() {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "eq expects two atoms · eq очікує два атоми · eq erwartet zwei Atome",
            span,
        ));
    }
    Ok(Value::Bool(left == right))
}

pub(super) fn evaluate_car(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("car", arguments, 1, span)?;
    match evaluate(&arguments[0], environment)? {
        Value::Pair(ref head, _) => Ok((**head).clone()),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "car expects a non-empty list · car очікує непорожній список · car erwartet eine nicht leere Liste",
            span,
        )),
    }
}

pub(super) fn evaluate_cdr(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("cdr", arguments, 1, span)?;
    match evaluate(&arguments[0], environment)? {
        Value::Pair(_, ref tail) => Ok((**tail).clone()),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "cdr expects a non-empty list · cdr очікує непорожній список · cdr erwartet eine nicht leere Liste",
            span,
        )),
    }
}

pub(super) fn evaluate_cons(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("cons", arguments, 2, span)?;
    let head = evaluate(&arguments[0], environment)?;
    let tail = evaluate(&arguments[1], environment)?;
    Ok(Value::Pair(Rc::new(head), Rc::new(tail)))
}

pub(super) fn evaluate_cond(
    clauses: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<EvalStep, LanguageError> {
    for clause in clauses {
        let ExprKind::List(parts) = &clause.kind else {
            return Err(LanguageError::new(
                ErrorKind::InvalidForm,
                "cond expects list clauses · cond очікує списки-умови · cond erwartet Listenklauseln",
                clause.span,
            ));
        };
        if parts.len() != 2 {
            return Err(LanguageError::new(
                ErrorKind::InvalidForm,
                "cond expects (test expression) clauses · cond очікує умови (перевірка вираз) · cond erwartet Klauseln der Form (Test Ausdruck)",
                clause.span,
            ));
        }
        if evaluate(&parts[0], environment)?.is_truthy() {
            return evaluate_step(&parts[1], environment);
        }
    }
    if clauses.is_empty() {
        // The span is retained for future strict empty-cond diagnostics.
        // Діапазон збережено для майбутньої строгої діагностики порожнього `cond`.
        // Der Bereich bleibt für eine künftige strikte Diagnose eines leeren `cond` erhalten.
        let _ = span;
    }
    Ok(EvalStep::Value(Value::Nil))
}

pub(super) fn exact_arity(
    operator: &str,
    arguments: &[Expr],
    expected: usize,
    span: Span,
) -> Result<(), LanguageError> {
    if arguments.len() == expected {
        return Ok(());
    }
    Err(LanguageError::new(
        ErrorKind::Arity,
        format!(
            "{operator}: expected / очікувалося / erwartet {expected}; received / отримано / erhalten {}",
            arguments.len()
        ),
        span,
    ))
}

pub(super) fn quoted(expression: &Expr) -> Value {
    match &expression.kind {
        ExprKind::Number(number) => Value::Number(*number),
        ExprKind::Rational(rational) => Value::Rational(*rational),
        ExprKind::String(value) => Value::String(value.clone()),
        ExprKind::Symbol(symbol) => Value::Symbol(symbol.clone()),
        ExprKind::List(items) => Value::list(items.iter().map(quoted)),
    }
}

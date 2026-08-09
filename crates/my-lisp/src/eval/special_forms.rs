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

/// `princ`, the `display`/`princ` half of the classic Lisp print-function
/// pair `print` is the other half of (see `Value::to_princ_string`):
/// strings come out raw, no surrounding quotes or escapes — for output
/// meant for a person or for reassembling as literal text, never meant to
/// be `read` back as the same value. `(princ "a")` and `(princ 'a)` print
/// identically (`a`) for exactly this reason: both are "the letter a as
/// text," the distinction `print` cares about (which one `read` would
/// reconstruct) doesn't apply here.
/// `princ`, «display»/«princ»-половина класичної Lisp-пари функцій друку,
/// другу половину якої складає `print` (див. `Value::to_princ_string`):
/// рядки виходять сирими, без лапок і екранування — для виводу, призначеного
/// людині чи повторному складанню як буквальний текст, ніколи не для
/// зчитування назад через `read` тим самим значенням. `(princ "a")` і
/// `(princ 'a)` друкують однаково (`a`) саме тому: обидва — "буква a як
/// текст", розрізнення, важливе для `print` (яке саме значення відновив
/// би `read`), тут не застосовне.
pub(super) fn evaluate_princ(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("princ", arguments, 1, span)?;
    let value = evaluate(&arguments[0], environment)?;
    environment.print(value.to_princ_string());
    Ok(value)
}

/// `read` is McCarthy's original reader primitive: it turns text into one
/// s-expression of *data*, the same way `'expr` does, without evaluating it —
/// `(eval (read "(+ 1 2)"))` is the read/eval loop written out by hand, in
/// the language itself. `(read "...")` (one argument) stays capability-free,
/// same as the rest of the crate — it parses the given string. `(read)`
/// (zero arguments) is the deliberate, explicit exception: it blocks on one
/// line of real stdin via `read_stdin_line`, which is `#[cfg]`-gated to a
/// clear `InvalidForm` error instead of a panic on `wasm32` — the browser
/// REPL (`crates/my-lisp-wasm`) has no console to block on.
/// `read` — оригінальний reader-примітив Маккарті: перетворює текст на одну
/// s-expression *даних*, так само як `'expr`, без обчислення — `(eval (read
/// "(+ 1 2)"))` це вручну виписаний read/eval цикл самою мовою. `(read
/// "...")` (один аргумент) лишається без host-можливостей, як і решта
/// крейта — парсить переданий рядок. `(read)` (без аргументів) — навмисний,
/// явний виняток: блокується на одному рядку справжнього stdin через
/// `read_stdin_line`, який через `#[cfg]` дає чітку помилку `InvalidForm`
/// замість паніки на `wasm32` — у браузерного REPL (`crates/my-lisp-wasm`)
/// немає консолі, на якій можна блокуватись.
/// `read` ist McCarthys ursprüngliches Reader-Primitiv: es macht aus Text
/// eine s-Expression aus *Daten*, genau wie `'expr`, ohne sie auszuwerten —
/// `(eval (read "(+ 1 2)"))` ist die Read/Eval-Schleife von Hand
/// ausgeschrieben, in der Sprache selbst. `(read "...")` (ein Argument)
/// bleibt ohne Host-Fähigkeiten wie der Rest des Crates — es parst die
/// übergebene Zeichenkette. `(read)` (ohne Argumente) ist die bewusste,
/// explizite Ausnahme: es blockiert auf einer Zeile echtem stdin über
/// `read_stdin_line`, das per `#[cfg]` auf `wasm32` einen klaren
/// `InvalidForm`-Fehler statt eines Panics liefert — der Browser-REPL
/// (`crates/my-lisp-wasm`) hat keine Konsole, auf der blockiert werden kann.
pub(super) fn evaluate_read(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    if arguments.len() > 1 {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "read expects zero or one arguments · read очікує нуль або один аргумент · read erwartet null oder ein Argument",
            span,
        ));
    }
    let source = if let Some(argument) = arguments.first() {
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
        let evaluated = evaluate(argument, environment)?;
        match &evaluated {
            Value::String(text) => text.to_string(),
            _ => {
                return Err(LanguageError::new(
                    ErrorKind::Type,
                    "read expects a string · read очікує рядок · read erwartet eine Zeichenkette",
                    argument.span,
                ))
            }
        }
    } else {
        read_stdin_line(span)?
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

/// Blocks on one line of real stdin. This is the one place in the crate that
/// touches an actual host I/O stream — see `evaluate_read`'s doc comment for
/// why this exception exists and how it's scoped away from `wasm32`.
/// Блокується на одному рядку справжнього stdin. Це єдине місце в крейті,
/// що торкається реального host I/O-потоку — чому цей виняток існує і як
/// він відгороджений від `wasm32`, див. doc-коментар `evaluate_read`.
/// Blockiert auf einer Zeile echtem stdin. Dies ist die einzige Stelle im
/// Crate, die einen echten Host-I/O-Stream berührt — warum diese Ausnahme
/// existiert und wie sie von `wasm32` abgegrenzt ist, siehe den
/// Doc-Kommentar von `evaluate_read`.
// Reliable when `my-lisp-cli` runs a *file* (verified: `(eval (read))` in a
// file, piped stdin data, evaluates correctly end to end). Inside the
// interactive REPL, this competes with rustyline for the same stdin — with
// piped/redirected (non-TTY) input, rustyline's own line reading can buffer
// ahead of what it hands back, so a later `(read)` call sees less than a
// real terminal session would. A genuine TTY reads line-by-line in raw mode
// without that over-buffering, so typed-at-a-terminal REPL use is expected
// to behave; piped REPL input is the documented edge case, not a silent gap.
// Надійно, коли `my-lisp-cli` виконує *файл* (перевірено: `(eval (read))` у
// файлі з переданим через pipe stdin коректно обчислюється наскрізно).
// Усередині інтерактивного REPL це конкурує з rustyline за той самий
// stdin — з переданим через pipe/перенаправленим (не-TTY) вводом власне
// читання рядків rustyline може буферизувати наперед більше, ніж віддає
// назад, тож пізніший виклик `(read)` бачить менше, ніж у справжній
// термінальній сесії. Справжній TTY читає рядок за рядком у raw-режимі
// без такого зайвого буферування, тож використання REPL з набором тексту
// в терміналі має працювати; REPL з переданим через pipe вводом —
// задокументований крайовий випадок, не тихо прихована прогалина.
#[cfg(not(target_arch = "wasm32"))]
fn read_stdin_line(span: Span) -> Result<String, LanguageError> {
    use std::io::BufRead;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("read: failed to read from stdin · read: не вдалось прочитати stdin · read: Lesen von stdin fehlgeschlagen: {error}"),
            span,
        )
    })?;
    Ok(line.trim_end_matches(['\n', '\r']).to_string())
}

#[cfg(target_arch = "wasm32")]
fn read_stdin_line(span: Span) -> Result<String, LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "read: interactive stdin is not available in this build · read: інтерактивний stdin недоступний у цій збірці · read: interaktives stdin ist in diesem Build nicht verfügbar",
        span,
    ))
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

/// Step 4 of `lib/clips-import.my`: reading a *real* `.clp` file off disk
/// rather than a caller-supplied quoted literal. `load` already reads a
/// file, but evaluates every top-level form it finds — exactly wrong for
/// CLIPS source, whose `defrule`/`=>` forms aren't meaningful my-lisp code
/// to *run*, only to read as data (see this file's own header comment).
/// `read-file` returns the raw text; `read-all` (below) parses text into
/// every top-level form as data, the multi-form counterpart to `read`
/// (which errors unless the string holds exactly one form). Deliberately
/// a new, separate host-capability boundary — not bundled into `load` or
/// `read` — since it changes what those two already-trusted forms can do.
/// Крок 4 `lib/clips-import.my`: читання *справжнього* `.clp`-файлу з
/// диска, а не наданого викликачем quoted-літералу. `load` уже читає
/// файл, але виконує кожну знайдену форму верхнього рівня — саме
/// неправильно для CLIPS-коду, чиї форми `defrule`/`=>` не є осмисленим
/// my-lisp кодом для *виконання*, лише для читання як дані. `read-file`
/// повертає сирий текст; `read-all` (нижче) парсить текст у кожну форму
/// верхнього рівня як дані — багатоформний відповідник `read` (який падає,
/// якщо рядок містить не рівно одну форму). Свідомо нова, окрема
/// host-capability межа — не вбудована в `load` чи `read` — бо змінює те,
/// що ці дві вже довірені форми можуть робити.
/// Schritt 4 von `lib/clips-import.my`: eine *echte* `.clp`-Datei von der
/// Festplatte lesen statt eines vom Aufrufer bereitgestellten
/// Quote-Literals. `load` liest bereits eine Datei, wertet aber jede
/// gefundene Form der obersten Ebene aus — genau falsch für CLIPS-Quellcode,
/// dessen `defrule`/`=>`-Formen kein sinnvoller my-lisp-Code zum
/// *Ausführen* sind, sondern nur zum Lesen als Daten. `read-file` gibt den
/// rohen Text zurück; `read-all` (unten) parst Text in jede Form der
/// obersten Ebene als Daten — das Mehrform-Gegenstück zu `read` (das
/// fehlschlägt, sofern der String nicht genau eine Form enthält). Bewusst
/// eine neue, separate Host-Capability-Grenze — nicht in `load` oder
/// `read` eingebaut — da sie ändert, was diese beiden bereits vertrauten
/// Formen können.
pub(super) fn evaluate_read_file(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("read-file", arguments, 1, span)?;
    let evaluated = evaluate(&arguments[0], environment)?;
    let Value::String(ref path) = evaluated else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "read-file expects a string path · read-file очікує рядок-шлях · read-file erwartet einen String-Pfad",
            span,
        ));
    };
    let contents = read_file(path, span)?;
    Ok(Value::String(Rc::from(contents.as_str())))
}

pub(super) fn evaluate_read_all(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("read-all", arguments, 1, span)?;
    let evaluated = evaluate(&arguments[0], environment)?;
    let Value::String(ref text) = evaluated else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "read-all expects a string · read-all очікує рядок · read-all erwartet eine Zeichenkette",
            span,
        ));
    };
    let expressions = crate::parse(text).map_err(|mut error| {
        error.span = span;
        error
    })?;
    Ok(Value::list(expressions.iter().map(quoted)))
}

pub(super) fn evaluate_load(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("load", arguments, 1, span)?;
    let evaluated = evaluate(&arguments[0], environment)?;
    let Value::String(ref path) = evaluated else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "load expects a string path · load очікує рядок-шлях · load erwartet einen String-Pfad",
            span,
        ));
    };
    
    let source = read_file(&path, span)?;
    let expressions = crate::parse(&source).map_err(|mut error| {
        error.span = span;
        error
    })?;
    
    let mut last_value = Value::Nil;
    for expr in expressions {
        last_value = evaluate(&expr, environment)?;
    }
    
    Ok(last_value)
}

#[cfg(not(target_arch = "wasm32"))]
fn read_file(path: &str, span: Span) -> Result<String, LanguageError> {
    std::fs::read_to_string(path).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("load: failed to read file {path}: {error}"),
            span,
        )
    })
}

#[cfg(target_arch = "wasm32")]
fn read_file(_path: &str, span: Span) -> Result<String, LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "load: file system access is not available in this build",
        span,
    ))
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
    if environment.try_alloc_cons().is_err() {
        return Err(LanguageError::new(
            ErrorKind::OutOfMemory,
            "cons: resource limit reached · cons: досягнуто межі ресурсу · cons: Ressourcengrenze erreicht",
            span,
        ));
    }
    Ok(Value::Pair(Rc::new(head), Rc::new(tail)))
}

/// The minimal symbol/string introspection this project held off on for a
/// long time (per CLAUDE.md's "don't grow the Rust surface" principle) —
/// added deliberately, not by default, when `lib/clips-import.my`'s Step 2
/// hit a real wall: converting CLIPS's `?x` variable syntax into `(var x)`
/// needs to peel the leading `?` off a symbol's name, and there is no way
/// to inspect a symbol's characters from within my-lisp itself. Three
/// small, general primitives (not one ad-hoc "strip-question-mark"
/// helper) so the capability is reusable, not single-purpose.
/// Мінімальна інтроспекція символів/рядків, яку цей проєкт довго
/// відкладав (за принципом CLAUDE.md "не розширювати поверхню Rust") —
/// додана свідомо, не за замовчуванням, коли Крок 2 `lib/clips-import.my`
/// вперся у реальну стіну: конвертація CLIPS-синтаксису змінних `?x` у
/// `(var x)` потребує відрізати провідний `?` від імені символу, а
/// перевірити символи самої my-lisp неможливо. Три невеликі, загальні
/// примітиви (не один ad-hoc хелпер "strip-question-mark"), щоб
/// можливість була придатна для повторного використання, не одноразовою.
/// Die minimale Symbol-/String-Introspektion, die dieses Projekt lange
/// zurückgehalten hat (nach dem Prinzip von CLAUDE.md, die Rust-Oberfläche
/// nicht wachsen zu lassen) — bewusst hinzugefügt, nicht standardmäßig,
/// als Schritt 2 von `lib/clips-import.my` an eine echte Grenze stieß: die
/// Umwandlung von CLIPS' `?x`-Variablensyntax in `(var x)` erfordert das
/// Abschälen des führenden `?` vom Namen eines Symbols, und es gibt keine
/// Möglichkeit, die Zeichen eines Symbols aus my-lisp selbst zu
/// inspizieren. Drei kleine, allgemeine Primitive (kein einzelner
/// Ad-hoc-Helfer "strip-question-mark"), damit die Fähigkeit
/// wiederverwendbar bleibt, nicht einmalig.
/// `symbol->string` needs a way to answer "is this actually a symbol"
/// before being called on an arbitrary atom — a CLIPS fact's arguments can
/// just as easily be numbers (`(temperature 98)`) as symbols, and there
/// was no existing predicate to tell them apart (see lib/unify.my's own
/// header comment: "no symbol?/numberp? primitive"). One small, general
/// predicate, not a special case baked into `symbol->string` itself.
pub(super) fn evaluate_symbol_predicate(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("symbol?", arguments, 1, span)?;
    Ok(Value::Bool(matches!(
        evaluate(&arguments[0], environment)?,
        Value::Symbol(_)
    )))
}

/// The `string?` counterpart to `symbol?` — found necessary by importing
/// a second genuine external CLIPS file: CLIPS conventionally allows an
/// optional docstring (a bare string literal) right after a `defrule`'s
/// name, before its conditions, and `lib/clips-import.my` needs to
/// recognize and skip it rather than treating it as a stray condition
/// that will never match any fact.
pub(super) fn evaluate_string_predicate(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string?", arguments, 1, span)?;
    Ok(Value::Bool(matches!(
        evaluate(&arguments[0], environment)?,
        Value::String(_)
    )))
}

pub(super) fn evaluate_symbol_to_string(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("symbol->string", arguments, 1, span)?;
    match evaluate(&arguments[0], environment)? {
        Value::Symbol(ref symbol) => Ok(Value::String(symbol.clone())),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "symbol->string expects a symbol · symbol->string очікує символ · symbol->string erwartet ein Symbol",
            span,
        )),
    }
}

pub(super) fn evaluate_string_to_symbol(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string->symbol", arguments, 1, span)?;
    match evaluate(&arguments[0], environment)? {
        Value::String(ref text) => Ok(Value::Symbol(text.clone())),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "string->symbol expects a string · string->symbol очікує рядок · string->symbol erwartet eine Zeichenkette",
            span,
        )),
    }
}

/// The first character, as a one-character string — the string analogue
/// of `car`. Errors on an empty string, same as `car` on an empty list.
pub(super) fn evaluate_string_first(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string-first", arguments, 1, span)?;
    match evaluate(&arguments[0], environment)? {
        Value::String(ref text) => match text.chars().next() {
            Some(character) => Ok(Value::String(Rc::from(character.to_string().as_str()))),
            None => Err(LanguageError::new(
                ErrorKind::Type,
                "string-first expects a non-empty string · string-first очікує непорожній рядок · string-first erwartet eine nicht leere Zeichenkette",
                span,
            )),
        },
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "string-first expects a string · string-first очікує рядок · string-first erwartet eine Zeichenkette",
            span,
        )),
    }
}

/// All but the first character — the string analogue of `cdr`. Errors on
/// an empty string rather than silently returning one, the same way `car`
/// errors on an empty list instead of returning `()`.
pub(super) fn evaluate_string_rest(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string-rest", arguments, 1, span)?;
    match evaluate(&arguments[0], environment)? {
        Value::String(ref text) => {
            let mut characters = text.chars();
            if characters.next().is_none() {
                return Err(LanguageError::new(
                    ErrorKind::Type,
                    "string-rest expects a non-empty string · string-rest очікує непорожній рядок · string-rest erwartet eine nicht leere Zeichenkette",
                    span,
                ));
            }
            Ok(Value::String(Rc::from(characters.as_str())))
        }
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "string-rest expects a string · string-rest очікує рядок · string-rest erwartet eine Zeichenkette",
            span,
        )),
    }
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
        ExprKind::Rational(rational) => Value::Rational(rational.clone()),
        ExprKind::String(value) => Value::String(value.clone()),
        ExprKind::Symbol(symbol) => Value::Symbol(symbol.clone()),
        ExprKind::List(items) => Value::list(items.iter().map(quoted)),
        ExprKind::Pair(head, tail) => Value::Pair(Rc::new(quoted(head)), Rc::new(quoted(tail))),
    }
}

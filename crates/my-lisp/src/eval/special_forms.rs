//! The McCarthy primitives (`eq`, `car`, `cdr`, `cons`, `cond`, `quote`'s helper),
//! plus `def`, `defmacro`, and `list`.
//! Примітиви Маккарті (`eq`, `car`, `cdr`, `cons`, `cond`, помічник `quote`),
//! а також `def`, `defmacro` і `list`.
//! Die McCarthy-Primitive (`eq`, `car`, `cdr`, `cons`, `cond`, Helfer für `quote`),
//! sowie `def`, `defmacro` und `list`.

use super::{closures, evaluate, evaluate_step, EvalStep};
use crate::{Environment, Exactness, ErrorKind, Expr, ExprKind, LanguageError, Span, Value};
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

/// Returns the same canonical, read-back-safe representation used by `print`
/// without touching the output transcript. This is the minimal bridge needed
/// to compose structured Lisp data with `write-file` and `tcp-write`.
/// Повертає канонічне представлення `print`, придатне для `read`, без виводу.
/// Gibt die kanonische, wieder einlesbare `print`-Darstellung ohne Ausgabe zurück.
pub(super) fn evaluate_write_to_string(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("write-to-string", arguments, 1, span)?;
    let value = evaluate(&arguments[0], environment)?;
    Ok(Value::String(Rc::from(value.to_string())))
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

/// The write-side counterpart to `read-file` (PLAN.md item 13) — one
/// primitive that opens and writes in a single step, the same shape
/// `read-file` already uses for opening and reading, rather than a
/// separate stateful file-handle value: the language has no mutable
/// cells or handles to represent one, and none of `read-file`/`load`
/// needed one either. Always creates or truncates-and-overwrites the
/// target file (`std::fs::write`'s own semantics), never appends —
/// append is a separate, not-yet-decided capability, not silently
/// folded into this one.
/// Симетричний до `read-file` бік запису (PLAN.md, пункт 13) — один
/// примітив, що відкриває й записує за один крок, та сама форма, яку
/// вже використовує `read-file` для відкриття й читання, а не окреме
/// stateful-значення файлового дескриптора: мова не має мутабельних
/// комірок чи дескрипторів, щоб його представити, і жодному з
/// `read-file`/`load` він не був потрібен. Завжди створює чи
/// перезаписує (обрізаючи) цільовий файл (власна семантика
/// `std::fs::write`), ніколи не дописує — дописування це окрема, ще не
/// вирішена спроможність, не мовчки згорнута в цю.
pub(super) fn evaluate_write_file(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("write-file", arguments, 2, span)?;
    let path_value = evaluate(&arguments[0], environment)?;
    let Value::String(ref path) = path_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "write-file expects a string path · write-file очікує рядок-шлях · write-file erwartet einen String-Pfad",
            span,
        ));
    };
    let content_value = evaluate(&arguments[1], environment)?;
    let Value::String(ref content) = content_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "write-file expects a string as its second argument · write-file очікує рядок другим аргументом · write-file erwartet eine Zeichenkette als zweites Argument",
            span,
        ));
    };
    write_file(path, content, span)?;
    Ok(content_value)
}

/// `(write-file-bytes path byte-list)` (PLAN.md item 22) — the byte-level
/// counterpart to `write-file`: `byte-list` is a list of fixnums 0-255,
/// written as raw bytes (`std::fs::write(path, &bytes)` over a `Vec<u8>`),
/// never through `&str`. `write-file` can only ever produce valid UTF-8 —
/// no primitive in the language can build a string containing an
/// arbitrary byte (no char-code/integer->char, no bytevector type), so
/// writing a real binary (compiled machine code, any non-UTF-8 format)
/// was impossible before this. Same shape as `write-file` otherwise:
/// always creates-or-truncates, never appends.
/// `(write-file-bytes path byte-list)` (PLAN.md, пункт 22) — байтовий
/// відповідник `write-file`: `byte-list` — список fixnum 0-255, пишеться
/// як сирі байти (`std::fs::write(path, &bytes)` над `Vec<u8>`), ніколи
/// через `&str`. `write-file` завжди дає лише коректний UTF-8 — жоден
/// примітив мови не міг побудувати рядок із довільним байтом (немає
/// char-code/integer->char, немає типу bytevector), тож записати
/// справжній бінарник (скомпільований машинний код, будь-який
/// не-UTF-8-формат) до цього було неможливо. Форма та сама, що й у
/// `write-file`: завжди створює чи перезаписує, ніколи не дописує.
pub(super) fn evaluate_write_file_bytes(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("write-file-bytes", arguments, 2, span)?;
    let path_value = evaluate(&arguments[0], environment)?;
    let Value::String(ref path) = path_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "write-file-bytes expects a string path · write-file-bytes очікує рядок-шлях · write-file-bytes erwartet einen String-Pfad",
            span,
        ));
    };
    let bytes_value = evaluate(&arguments[1], environment)?;
    let bytes = expect_byte_list(&bytes_value, arguments[1].span)?;
    write_file_bytes(path, &bytes, span)?;
    Ok(bytes_value)
}

/// `(read-file-bytes path)` (PLAN.md item 22) — the byte-level counterpart
/// to `read-file`: returns the file's raw bytes as a list of fixnums
/// 0-255, not a UTF-8-decoded string, which would fail outright — or
/// worse, silently corrupt — on a non-UTF-8 file.
/// `(read-file-bytes path)` (PLAN.md, пункт 22) — байтовий відповідник
/// `read-file`: повертає сирі байти файлу як список fixnum 0-255, не
/// UTF-8-декодований рядок, який би або відверто провалився, або —
/// гірше — мовчки спотворив дані на не-UTF-8-файлі.
pub(super) fn evaluate_read_file_bytes(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("read-file-bytes", arguments, 1, span)?;
    let evaluated = evaluate(&arguments[0], environment)?;
    let Value::String(ref path) = evaluated else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "read-file-bytes expects a string path · read-file-bytes очікує рядок-шлях · read-file-bytes erwartet einen String-Pfad",
            span,
        ));
    };
    let bytes = read_file_bytes(path, span)?;
    Ok(Value::list(
        bytes
            .into_iter()
            .map(|byte| Value::Number(byte as f64, Exactness::Exact)),
    ))
}

fn expect_byte_list(value: &Value, span: Span) -> Result<Vec<u8>, LanguageError> {
    let mut bytes = Vec::new();
    let mut current = value;
    loop {
        match current {
            Value::Nil => return Ok(bytes),
            Value::Pair(head, tail) => {
                let Value::Number(number, _) = **head else {
                    return Err(LanguageError::new(
                        ErrorKind::Type,
                        "write-file-bytes expects a list of integers 0-255 · write-file-bytes очікує список цілих чисел 0-255 · write-file-bytes erwartet eine Liste von Ganzzahlen 0-255",
                        span,
                    ));
                };
                if number.fract() != 0.0 || !(0.0..=255.0).contains(&number) {
                    return Err(LanguageError::new(
                        ErrorKind::Type,
                        "write-file-bytes expects each element to be an integer between 0 and 255 · write-file-bytes очікує, щоб кожен елемент був цілим числом від 0 до 255 · write-file-bytes erwartet, dass jedes Element eine Ganzzahl zwischen 0 und 255 ist",
                        span,
                    ));
                }
                bytes.push(number as u8);
                current = tail;
            }
            _ => {
                return Err(LanguageError::new(
                    ErrorKind::Type,
                    "write-file-bytes expects a proper list of integers 0-255 · write-file-bytes очікує правильний список цілих чисел 0-255 · write-file-bytes erwartet eine echte Liste von Ganzzahlen 0-255",
                    span,
                ))
            }
        }
    }
}

/// String concatenation (PLAN.md item 14) — genuinely needs a Rust
/// primitive, unlike `string-length`/`string-contains?` (both now in
/// `lib/core.my`, expressible via `string-first`/`string-rest`/`eq`
/// alone): `Value::String` wraps an immutable `Rc<str>`, and no
/// existing primitive combines two strings into a new one — the item
/// 20 audit test ("already expressible acceptably?") comes back no
/// here, not yes.
/// Конкатенація рядків (PLAN.md, пункт 14) — справді потребує
/// Rust-примітива, на відміну від `string-length`/`string-contains?`
/// (обидва тепер у `lib/core.my`, виразні через самі
/// `string-first`/`string-rest`/`eq`): `Value::String` огортає
/// незмінний `Rc<str>`, і жоден наявний примітив не об'єднує два
/// рядки в новий — тест аудиту з пункту 20 ("уже виразне прийнятним
/// способом?") тут дає "ні", не "так".
pub(super) fn evaluate_string_append(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string-append", arguments, 2, span)?;
    let left_value = evaluate(&arguments[0], environment)?;
    let Value::String(ref left) = left_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "string-append expects two strings · string-append очікує два рядки · string-append erwartet zwei Zeichenketten",
            span,
        ));
    };
    let right_value = evaluate(&arguments[1], environment)?;
    let Value::String(ref right) = right_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "string-append expects two strings · string-append очікує два рядки · string-append erwartet zwei Zeichenketten",
            span,
        ));
    };
    Ok(Value::String(Rc::from(format!("{left}{right}").as_str())))
}

/// Lexicographic string ordering (PLAN.md item 15) — the one new Rust
/// primitive the persistent-map design actually needs: Rust's `Ord` for
/// `&str` gives this for free, but nothing in the language could derive
/// "is one string before another" from `string-first`/`string-rest`/`eq`
/// alone (those only ever test *equality* one character at a time, never
/// ordering). Everything built on top of this one primitive — the
/// balanced tree itself, insert, lookup — is ordinary my-lisp, per the
/// same item-20 G5 test `string-append` failed and `string-length` passed.
/// Лексикографічне впорядкування рядків (PLAN.md, пункт 15) — єдиний
/// новий Rust-примітив, якого справді потребує дизайн персистентної
/// мапи: `Ord` для `&str` у Rust дає це безкоштовно, але ніщо в мові не
/// могло вивести "який рядок раніше" лише зі `string-first`/`string-rest`/
/// `eq` (вони перевіряють лише *рівність* по символу, ніколи порядок).
/// Усе, побудоване поверх цього одного примітива — саме збалансоване
/// дерево, вставка, пошук — звичайна my-lisp, за тим самим тестом G5 з
/// пункту 20, який `string-append` провалив, а `string-length` пройшла.
pub(super) fn evaluate_string_less_than(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("string<?", arguments, 2, span)?;
    let left_value = evaluate(&arguments[0], environment)?;
    let Value::String(ref left) = left_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "string<? expects two strings · string<? очікує два рядки · string<? erwartet zwei Zeichenketten",
            span,
        ));
    };
    let right_value = evaluate(&arguments[1], environment)?;
    let Value::String(ref right) = right_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "string<? expects two strings · string<? очікує два рядки · string<? erwartet zwei Zeichenketten",
            span,
        ));
    };
    Ok(Value::Bool(left.as_ref() < right.as_ref()))
}

/// `(tcp-connect host port)` (PLAN.md item 21) — the outbound-client half of
/// "talk to other AI systems" (principle 3, extended to LLM APIs/other
/// agents): opens a TCP connection, returns a `Value::TcpConnection` handle.
/// `(tcp-connect "example.com" 443)` — the caller writes an HTTP request
/// itself with `tcp-write`/`string-append` and reads the response with
/// `tcp-read`; no HTTP/TLS logic lives in Rust, only the raw byte pipe (S2:
/// connection failures fail named, `ErrorKind::InvalidForm`, never silently).
/// `(tcp-connect хост порт)` (PLAN.md, пункт 21) — вихідна/клієнтська
/// половина "спілкуватись з іншими AI-системами" (принцип 3, поширений на
/// LLM API/інших агентів): відкриває TCP-з'єднання, повертає handle
/// `Value::TcpConnection`. `(tcp-connect "example.com" 443)` — сам виклик
/// формує HTTP-запит через `tcp-write`/`string-append`, читає відповідь
/// через `tcp-read`; жодної HTTP/TLS-логіки в Rust, лише сирий байтовий
/// канал (S2: помилки з'єднання провалюються названо, `ErrorKind::InvalidForm`,
/// ніколи мовчки).
pub(super) fn evaluate_tcp_connect(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-connect", arguments, 2, span)?;
    let host_value = evaluate(&arguments[0], environment)?;
    let Value::String(ref host) = host_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "tcp-connect expects a string host · tcp-connect очікує рядок-хост · tcp-connect erwartet einen String-Host",
            arguments[0].span,
        ));
    };
    let port = expect_port(&arguments[1], environment)?;
    let stream = tcp_connect(host, port, span)?;
    Ok(Value::TcpConnection(Rc::new(std::cell::RefCell::new(stream))))
}

/// `(tcp-listen port)` — the inbound-server half: binds and starts listening,
/// returns a `Value::TcpListener` handle for `tcp-accept`. Lets my-lisp
/// accept connections from other agents, not just call out to them.
/// `(tcp-listen порт)` — вхідна/серверна половина: біндиться й починає
/// слухати, повертає handle `Value::TcpListener` для `tcp-accept`. Дозволяє
/// my-lisp приймати з'єднання від інших агентів, не лише звертатись до них.
pub(super) fn evaluate_tcp_listen(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-listen", arguments, 1, span)?;
    let port = expect_port(&arguments[0], environment)?;
    let listener = tcp_listen(port, span)?;
    Ok(Value::TcpListener(Rc::new(listener)))
}

/// `(tcp-accept listener)` — blocks until one inbound connection arrives on
/// `listener`, returns it as a `Value::TcpConnection` (the same handle type
/// `tcp-connect` produces — `tcp-read`/`tcp-write`/`tcp-close` don't care
/// which side opened the connection).
/// `(tcp-accept listener)` — блокується, поки не прийде одне вхідне
/// з'єднання на `listener`, повертає його як `Value::TcpConnection` (той
/// самий тип handle, що дає `tcp-connect` — `tcp-read`/`tcp-write`/`tcp-close`
/// не розрізняють, яка сторона відкрила з'єднання).
pub(super) fn evaluate_tcp_accept(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-accept", arguments, 1, span)?;
    let listener_value = evaluate(&arguments[0], environment)?;
    let Value::TcpListener(ref listener) = listener_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "tcp-accept expects a TCP listener · tcp-accept очікує TCP-listener · tcp-accept erwartet einen TCP-Listener",
            arguments[0].span,
        ));
    };
    let stream = tcp_accept(listener, span)?;
    Ok(Value::TcpConnection(Rc::new(std::cell::RefCell::new(stream))))
}

/// `(tcp-read connection)` — one `read()` call, up to 64 KiB, returned as a
/// string; `""` means the peer closed the connection (EOF), not an error.
/// A response larger than one read is drained by calling `tcp-read`
/// recursively and building the result with `string-append` (item 14) —
/// the same recursive-accumulation shape `lib/core.my` already uses
/// everywhere else, not a new idiom invented for sockets.
/// `(tcp-read connection)` — один виклик `read()`, до 64 КіБ, повертається
/// як рядок; `""` означає, що інша сторона закрила з'єднання (EOF), не
/// помилку. Відповідь, більша за один read, витягується рекурсивним
/// викликом `tcp-read` і будується через `string-append` (пункт 14) — та
/// сама форма рекурсивного накопичення, що `lib/core.my` уже використовує
/// всюди, не новий ідіом, вигаданий для сокетів.
pub(super) fn evaluate_tcp_read(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-read", arguments, 1, span)?;
    let connection_value = evaluate(&arguments[0], environment)?;
    let Value::TcpConnection(ref connection) = connection_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "tcp-read expects a TCP connection · tcp-read очікує TCP-з'єднання · tcp-read erwartet eine TCP-Verbindung",
            arguments[0].span,
        ));
    };
    let text = tcp_read(connection, span)?;
    Ok(Value::String(Rc::from(text.as_str())))
}

/// `(tcp-write connection content)` — writes `content`'s UTF-8 bytes,
/// returns `content` unchanged (composes like `print`/`write-file`).
/// `(tcp-write connection content)` — записує UTF-8-байти `content`,
/// повертає `content` без змін (компонується як `print`/`write-file`).
pub(super) fn evaluate_tcp_write(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-write", arguments, 2, span)?;
    let connection_value = evaluate(&arguments[0], environment)?;
    let Value::TcpConnection(ref connection) = connection_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "tcp-write expects a TCP connection · tcp-write очікує TCP-з'єднання · tcp-write erwartet eine TCP-Verbindung",
            arguments[0].span,
        ));
    };
    let content_value = evaluate(&arguments[1], environment)?;
    let Value::String(ref content) = content_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "tcp-write expects a string as its second argument · tcp-write очікує рядок другим аргументом · tcp-write erwartet eine Zeichenkette als zweites Argument",
            arguments[1].span,
        ));
    };
    tcp_write(connection, content, span)?;
    Ok(content_value)
}

/// `(tcp-close connection)` — explicitly shuts down both directions of the
/// connection rather than waiting for the handle to be dropped, so the
/// peer sees the close promptly (matters for HTTP servers reading until
/// EOF). Returns `t`.
/// `(tcp-close connection)` — явно закриває з'єднання в обидва боки, не
/// чекаючи, поки handle буде відкинуто, щоб інша сторона побачила закриття
/// одразу (важливо для HTTP-серверів, що читають до EOF). Повертає `t`.
pub(super) fn evaluate_tcp_close(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-close", arguments, 1, span)?;
    let connection_value = evaluate(&arguments[0], environment)?;
    let Value::TcpConnection(ref connection) = connection_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "tcp-close expects a TCP connection · tcp-close очікує TCP-з'єднання · tcp-close erwartet eine TCP-Verbindung",
            arguments[0].span,
        ));
    };
    tcp_close(connection, span)?;
    Ok(Value::Bool(true))
}

fn expect_port(expr: &Expr, environment: &Environment) -> Result<u16, LanguageError> {
    let value = evaluate(expr, environment)?;
    let Value::Number(port, _) = value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "expected a port number · очікувався номер порту · erwartete eine Portnummer",
            expr.span,
        ));
    };
    if port.fract() != 0.0 || port < 0.0 || port > u16::MAX as f64 {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "port must be an integer between 0 and 65535 · порт має бути цілим числом від 0 до 65535 · Port muss eine Ganzzahl zwischen 0 und 65535 sein",
            expr.span,
        ));
    }
    Ok(port as u16)
}

#[cfg(not(target_arch = "wasm32"))]
fn tcp_connect(host: &str, port: u16, span: Span) -> Result<std::net::TcpStream, LanguageError> {
    std::net::TcpStream::connect((host, port)).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("tcp-connect: failed to connect to {host}:{port}: {error}"),
            span,
        )
    })
}

#[cfg(target_arch = "wasm32")]
fn tcp_connect(_host: &str, _port: u16, span: Span) -> Result<std::net::TcpStream, LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "tcp-connect: networking is not available in this build",
        span,
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn tcp_listen(port: u16, span: Span) -> Result<std::net::TcpListener, LanguageError> {
    std::net::TcpListener::bind(("0.0.0.0", port)).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("tcp-listen: failed to bind port {port}: {error}"),
            span,
        )
    })
}

#[cfg(target_arch = "wasm32")]
fn tcp_listen(_port: u16, span: Span) -> Result<std::net::TcpListener, LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "tcp-listen: networking is not available in this build",
        span,
    ))
}

fn tcp_accept(
    listener: &std::net::TcpListener,
    span: Span,
) -> Result<std::net::TcpStream, LanguageError> {
    listener
        .accept()
        .map(|(stream, _addr)| stream)
        .map_err(|error| {
            LanguageError::new(
                ErrorKind::InvalidForm,
                format!("tcp-accept: failed to accept a connection: {error}"),
                span,
            )
        })
}

fn tcp_read(
    connection: &std::cell::RefCell<std::net::TcpStream>,
    span: Span,
) -> Result<String, LanguageError> {
    use std::io::Read;
    let mut buffer = [0u8; 65536];
    let read = connection
        .borrow_mut()
        .read(&mut buffer)
        .map_err(|error| {
            LanguageError::new(
                ErrorKind::InvalidForm,
                format!("tcp-read: failed to read from the connection: {error}"),
                span,
            )
        })?;
    String::from_utf8(buffer[..read].to_vec()).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("tcp-read: received bytes that aren't valid UTF-8: {error}"),
            span,
        )
    })
}

fn tcp_write(
    connection: &std::cell::RefCell<std::net::TcpStream>,
    content: &str,
    span: Span,
) -> Result<(), LanguageError> {
    use std::io::Write;
    connection
        .borrow_mut()
        .write_all(content.as_bytes())
        .map_err(|error| {
            LanguageError::new(
                ErrorKind::InvalidForm,
                format!("tcp-write: failed to write to the connection: {error}"),
                span,
            )
        })
}

fn tcp_close(
    connection: &std::cell::RefCell<std::net::TcpStream>,
    span: Span,
) -> Result<(), LanguageError> {
    connection
        .borrow()
        .shutdown(std::net::Shutdown::Both)
        .map_err(|error| {
            LanguageError::new(
                ErrorKind::InvalidForm,
                format!("tcp-close: failed to close the connection: {error}"),
                span,
            )
        })
}

/// `(process-run program args)` (PLAN.md item 21's follow-up) — runs
/// `program` with `args` (a list of strings) and returns
/// `(list exit-code stdout stderr)`. Deliberately narrow, not a general
/// shell-out primitive: `std::process::Command::new(program).args(args)`
/// never goes through a shell (no `sh -c`, no string interpolation, no
/// injection surface via `;`/`&&`/backticks in an argument), and the
/// session must have opted into exactly `program`'s name via
/// `Environment::with_process_allowlist` — the default session
/// (`Environment::root()`) always fails this named, never silently. See
/// that method's own comment for why: combined with `tcp-accept`'s
/// inbound networking, an unrestricted `process-run` would let a remote
/// peer reach arbitrary command execution through a my-lisp program.
/// `(process-run program args)` (продовження PLAN.md, пункт 21) —
/// запускає `program` з `args` (список рядків), повертає
/// `(list exit-code stdout stderr)`. Свідомо вузький, не загальний
/// shell-примітив: `std::process::Command::new(program).args(args)`
/// ніколи не йде через shell (без `sh -c`, без інтерполяції рядків, без
/// поверхні для ін'єкції через `;`/`&&`/backtick в аргументі), і сесія
/// має явно дозволити точно ім'я `program` через
/// `Environment::with_process_allowlist` — типова сесія
/// (`Environment::root()`) завжди провалює це названо, ніколи мовчки. Див.
/// власний коментар того методу чому: разом із вхідною мережею
/// `tcp-accept`, необмежений `process-run` дав би віддаленому учаснику
/// шлях до довільного виконання команд через my-lisp-програму.
pub(super) fn evaluate_process_run(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("process-run", arguments, 2, span)?;
    let program_value = evaluate(&arguments[0], environment)?;
    let Value::String(ref program) = program_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "process-run expects a string program name · process-run очікує рядок-ім'я програми · process-run erwartet einen String-Programmnamen",
            arguments[0].span,
        ));
    };
    if !environment.is_process_allowed(program) {
        return Err(LanguageError::new(
            ErrorKind::InvalidForm,
            format!("process-run: {program} is not on this session's allowlist · process-run: {program} немає в allowlist цієї сесії · process-run: {program} steht nicht auf der Allowlist dieser Sitzung"),
            span,
        ));
    }
    let args_value = evaluate(&arguments[1], environment)?;
    let args = expect_string_list(&args_value, arguments[1].span)?;
    let output = process_run(program, &args, span)?;
    Ok(Value::list([
        Value::Number(output.status.code().unwrap_or(-1) as f64, Exactness::Exact),
        Value::String(Rc::from(String::from_utf8_lossy(&output.stdout).as_ref())),
        Value::String(Rc::from(String::from_utf8_lossy(&output.stderr).as_ref())),
    ]))
}

fn expect_string_list(value: &Value, span: Span) -> Result<Vec<String>, LanguageError> {
    let mut items = Vec::new();
    let mut current = value;
    loop {
        match current {
            Value::Nil => return Ok(items),
            Value::Pair(head, tail) => {
                let Value::String(ref text) = **head else {
                    return Err(LanguageError::new(
                        ErrorKind::Type,
                        "process-run expects a list of strings for its second argument · process-run очікує список рядків другим аргументом · process-run erwartet eine Liste von Zeichenketten als zweites Argument",
                        span,
                    ));
                };
                items.push(text.to_string());
                current = tail;
            }
            _ => {
                return Err(LanguageError::new(
                    ErrorKind::Type,
                    "process-run expects a proper list of strings for its second argument · process-run очікує правильний список рядків другим аргументом · process-run erwartet eine echte Liste von Zeichenketten als zweites Argument",
                    span,
                ))
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn process_run(program: &str, args: &[String], span: Span) -> Result<std::process::Output, LanguageError> {
    std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|error| {
            LanguageError::new(
                ErrorKind::InvalidForm,
                format!("process-run: failed to run {program}: {error}"),
                span,
            )
        })
}

#[cfg(target_arch = "wasm32")]
fn process_run(_program: &str, _args: &[String], span: Span) -> Result<std::process::Output, LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "process-run: process execution is not available in this build",
        span,
    ))
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

#[cfg(not(target_arch = "wasm32"))]
fn write_file(path: &str, content: &str, span: Span) -> Result<(), LanguageError> {
    std::fs::write(path, content).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("write-file: failed to write file {path}: {error}"),
            span,
        )
    })
}

#[cfg(target_arch = "wasm32")]
fn write_file(_path: &str, _content: &str, span: Span) -> Result<(), LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "write-file: file system access is not available in this build",
        span,
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn read_file_bytes(path: &str, span: Span) -> Result<Vec<u8>, LanguageError> {
    std::fs::read(path).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("read-file-bytes: failed to read file {path}: {error}"),
            span,
        )
    })
}

#[cfg(target_arch = "wasm32")]
fn read_file_bytes(_path: &str, span: Span) -> Result<Vec<u8>, LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "read-file-bytes: file system access is not available in this build",
        span,
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn write_file_bytes(path: &str, bytes: &[u8], span: Span) -> Result<(), LanguageError> {
    std::fs::write(path, bytes).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("write-file-bytes: failed to write file {path}: {error}"),
            span,
        )
    })
}

#[cfg(target_arch = "wasm32")]
fn write_file_bytes(_path: &str, _bytes: &[u8], span: Span) -> Result<(), LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "write-file-bytes: file system access is not available in this build",
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
        ExprKind::Number(number, exactness) => Value::Number(*number, *exactness),
        ExprKind::Rational(rational) => Value::Rational(rational.clone()),
        ExprKind::String(value) => Value::String(value.clone()),
        ExprKind::Symbol(symbol) => Value::Symbol(symbol.clone()),
        ExprKind::List(items) => Value::list(items.iter().map(quoted)),
        ExprKind::Pair(head, tail) => Value::Pair(Rc::new(quoted(head)), Rc::new(quoted(tail))),
    }
}

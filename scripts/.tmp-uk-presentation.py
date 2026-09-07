from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one anchor, found {count}")
    p.write_text(text.replace(old, new), encoding="utf-8")


presentation = r'''//! Presentation-only rendering for interactive human-facing surfaces.
//!
//! Canonical `Value::Display` and `LanguageError::render` stay unchanged: they
//! are used by conformance, machine protocols and source round-tripping.  This
//! module changes only what an interactive surface shows to a person.

use crate::{ErrorKind, Exactness, LanguageError, NumericBuffer, Value};
use std::collections::HashMap;
use std::sync::OnceLock;

const UK_COVERAGE: &str = include_str!("../../../lib/surface/uk-sa-coverage.wsm");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationLanguage {
    Canonical,
    English,
    Ukrainian,
    Sanskrit,
}

fn uk_names() -> &'static HashMap<&'static str, &'static str> {
    static NAMES: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    NAMES.get_or_init(|| {
        let mut names = HashMap::new();
        for line in UK_COVERAGE.lines() {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            if fields.first() == Some(&"(entry")
                && fields.get(5) == Some(&"stable")
                && fields.get(3) != Some(&"—")
            {
                names.insert(fields[2], fields[3]);
            }
        }
        names
    })
}

fn uk_operation_name(name: &str) -> String {
    match name {
        "PRIM_ATOM" => "атом?".to_string(),
        "PRIM_EQ" => "тотожне?".to_string(),
        "PRIM_CONS" => "сполучити".to_string(),
        "PRIM_CAR" => "перше".to_string(),
        "PRIM_CDR" => "решта".to_string(),
        other => uk_names().get(other).copied().unwrap_or(other).to_string(),
    }
}

fn canonical_inexact(number: f64) -> String {
    if number.fract() == 0.0 && number.is_finite() {
        format!("{number:.1}")
    } else {
        number.to_string()
    }
}

fn uk_decimal(text: String) -> String {
    text.replace('.', ",")
}

fn render_uk(value: &Value) -> String {
    match value {
        Value::Builtin(builtin) => {
            format!("#<вбудована {}>", uk_operation_name(builtin.name))
        }
        Value::Vector(vector) => {
            let items = vector
                .borrow()
                .iter()
                .map(render_uk)
                .collect::<Vec<_>>();
            format!("#({})", items.join(" "))
        }
        Value::NumericBuffer(NumericBuffer::I32(values)) => {
            let items = values.iter().map(i32::to_string).collect::<Vec<_>>();
            format!("#i32({})", items.join(" "))
        }
        Value::NumericBuffer(NumericBuffer::F32(values)) => {
            let items = values
                .iter()
                .map(|number| {
                    let text = if number.fract() == 0.0 {
                        format!("{number:.1}")
                    } else {
                        number.to_string()
                    };
                    uk_decimal(text)
                })
                .collect::<Vec<_>>();
            format!("#f32({})", items.join(" "))
        }
        Value::Nil | Value::Bool(false) => "()".to_string(),
        Value::Bool(true) => "істина".to_string(),
        Value::Number(number, Exactness::Exact) => number.to_string(),
        Value::Number(number, Exactness::Inexact) => uk_decimal(canonical_inexact(*number)),
        Value::Rational(number) => number.to_string(),
        Value::String(text) => {
            let mut escaped = String::with_capacity(text.len() + 2);
            escaped.push('"');
            for ch in text.chars() {
                match ch {
                    '"' => escaped.push_str("\\\""),
                    '\\' => escaped.push_str("\\\\"),
                    '\n' => escaped.push_str("\\n"),
                    '\t' => escaped.push_str("\\t"),
                    other => escaped.push(other),
                }
            }
            escaped.push('"');
            escaped
        }
        Value::Symbol(symbol) if symbol.as_ref() == "t" => "істина".to_string(),
        Value::Symbol(symbol) => symbol.to_string(),
        Value::Pair(_, _) => render_pair_uk(value),
        Value::Closure(_) => "<функція>".to_string(),
        Value::Macro(_) => "<макрос>".to_string(),
        Value::TcpConnection(_) => "<tcp-з'єднання>".to_string(),
        Value::TcpListener(_) => "<tcp-слухач>".to_string(),
    }
}

fn render_pair_uk(value: &Value) -> String {
    let mut output = String::from("(");
    let mut current = value;
    let mut first = true;
    loop {
        match current {
            Value::Pair(head, tail) => {
                if !first {
                    output.push(' ');
                }
                output.push_str(&render_uk(head));
                current = tail;
                first = false;
            }
            Value::Nil => {
                output.push(')');
                return output;
            }
            tail => {
                output.push_str(" . ");
                output.push_str(&render_uk(tail));
                output.push(')');
                return output;
            }
        }
    }
}

pub fn render_value_for_presentation(
    value: &Value,
    language: PresentationLanguage,
) -> String {
    match language {
        PresentationLanguage::Ukrainian => render_uk(value),
        PresentationLanguage::Canonical
        | PresentationLanguage::English
        | PresentationLanguage::Sanskrit => value.to_string(),
    }
}

fn argument_word(count: usize) -> &'static str {
    let tens = count % 100;
    let ones = count % 10;
    if tens == 11 {
        "аргументів"
    } else if ones == 1 {
        "аргумент"
    } else if (2..=4).contains(&ones) && !(12..=14).contains(&tens) {
        "аргументи"
    } else {
        "аргументів"
    }
}

fn translate_expectation(text: &str) -> String {
    if let Some(count) = text
        .strip_prefix("exactly ")
        .and_then(|rest| rest.strip_suffix(" argument(s)"))
        .and_then(|digits| digits.parse::<usize>().ok())
    {
        return format!("рівно {count} {}", argument_word(count));
    }

    match text {
        "a non-empty list" => "непорожній список".to_string(),
        "two atoms" => "два атоми".to_string(),
        "a callable function" => "функцію, яку можна викликати".to_string(),
        "a symbol name" => "ім'я-символ".to_string(),
        "list clauses" => "спискові гілки".to_string(),
        "(test expression) clauses" => "гілки виду (перевірка вираз)".to_string(),
        "zero or one arguments" => "нуль або один аргумент".to_string(),
        "an exact non-negative integer" => "точне невід'ємне ціле число".to_string(),
        "a string" => "текст".to_string(),
        "a string path" => "текстовий шлях".to_string(),
        "a list of strings for its second argument" => {
            "список текстів другим аргументом".to_string()
        }
        other => other
            .replace("non-empty list", "непорожній список")
            .replace("list of strings", "список текстів")
            .replace("string", "текст")
            .replace("integer", "ціле число")
            .replace("number", "число")
            .replace("symbol", "символ")
            .replace("vector", "вектор")
            .replace("argument(s)", "аргументів")
            .replace("arguments", "аргументи"),
    }
}

fn ukrainian_message(message: &str) -> String {
    if message.starts_with("unknown symbol") {
        let symbol = message
            .rsplit_once(": ")
            .map(|(_, symbol)| symbol)
            .unwrap_or("?");
        return format!("Невідомий символ: {symbol}");
    }

    if let Some((operator, rest)) =
        message.split_once(": expected / ochikuvalosia / erwartet ")
    {
        if let Some((expected, received)) =
            rest.split_once("; received / otrymano / erhalten ")
        {
            return format!(
                "{}: очікувалося {expected}; отримано {received}",
                uk_operation_name(operator)
            );
        }
    }

    let english = message.split(" · ").next().unwrap_or(message).trim();

    match english {
        "unexpected closing parenthesis" => "неочікувана закривна дужка".to_string(),
        "unclosed list" => "незакритий список".to_string(),
        "unclosed string" => "незакритий текстовий рядок".to_string(),
        "unclosed numeric buffer" => "незакритий числовий буфер".to_string(),
        "unexpected end of input" | "unexpected eof" => "неочікуваний кінець вводу".to_string(),
        "division by zero" => "Ділення на нуль".to_string(),
        "a dotted pair is not executable code" => "крапкова пара не є виконуваним кодом".to_string(),
        "quoted structure exceeds reader limit" => "цитована структура перевищує межу читача".to_string(),
        _ => {
            if let Some((operator, expectation)) = english.split_once(" expects ") {
                return format!(
                    "{} очікує {}",
                    uk_operation_name(operator),
                    translate_expectation(expectation)
                );
            }
            if let Some(operator) = english.strip_suffix(": resource limit reached") {
                return format!("{}: досягнуто межі ресурсу", uk_operation_name(operator));
            }

            english
                .replace("numeric literal", "числовий літерал")
                .replace("resource limit", "межа ресурсу")
                .replace("invalid form", "некоректна форма")
                .replace("invalid", "некоректний")
                .replace("expected", "очікувалося")
                .replace("received", "отримано")
                .replace("unexpected", "неочікуваний")
        }
    }
}

pub fn present_system_message(message: &str, language: PresentationLanguage) -> String {
    match language {
        PresentationLanguage::Ukrainian => ukrainian_message(message),
        PresentationLanguage::Canonical
        | PresentationLanguage::English
        | PresentationLanguage::Sanskrit => message.to_string(),
    }
}

fn uk_error_line(error: &LanguageError) -> String {
    let message = ukrainian_message(&error.message);
    match error.kind {
        ErrorKind::Parse => format!("Синтаксична помилка: {message}"),
        ErrorKind::UnknownSymbol => message,
        ErrorKind::Arity => format!("Помилка кількості аргументів: {message}"),
        ErrorKind::Type => format!("Помилка типу: {message}"),
        ErrorKind::InvalidForm => format!("Некоректна форма: {message}"),
        ErrorKind::OutOfMemory => format!("Вичерпано ресурс пам'яті: {message}"),
        ErrorKind::NumericOverflow => format!("Перевищено числову межу: {message}"),
        ErrorKind::DivisionByZero if message == "Ділення на нуль" => message,
        ErrorKind::DivisionByZero => format!("Ділення на нуль: {message}"),
    }
}

pub fn render_error_for_presentation(
    error: &LanguageError,
    source: &str,
    language: PresentationLanguage,
) -> String {
    if language != PresentationLanguage::Ukrainian {
        return error.render(source);
    }

    let (line, column) = error.line_col(source);
    let line_text = source.lines().nth(line - 1).unwrap_or("");
    let span_chars = source[error.span.start.min(source.len())..error.span.end.min(source.len())]
        .chars()
        .count()
        .max(1);
    let gutter = format!("{line}");
    let indent = " ".repeat(gutter.len());
    let caret = " ".repeat(column.saturating_sub(1)) + &"^".repeat(span_chars);

    format!(
        "{message}\n{indent} --> рядок {line}, стовпець {column}\n{indent} |\n{gutter} | {line_text}\n{indent} | {caret}",
        message = uk_error_line(error),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{eval_program, parse, Session, Span};

    #[test]
    fn ukrainian_value_presentation_changes_only_the_human_view() {
        let mut session = Session::default();
        let truth = eval_program("(atom 'мама)", &mut session).expect("predicate").value;
        assert_eq!(truth.to_string(), "t", "canonical value must not change");
        assert_eq!(
            render_value_for_presentation(&truth, PresentationLanguage::Ukrainian),
            "істина"
        );
        assert_eq!(
            render_value_for_presentation(&truth, PresentationLanguage::Canonical),
            "t"
        );
    }

    #[test]
    fn ukrainian_presentation_uses_decimal_comma_for_inexact_numbers() {
        let value = Value::Number(12.5, Exactness::Inexact);
        assert_eq!(
            render_value_for_presentation(&value, PresentationLanguage::Ukrainian),
            "12,5"
        );
        assert_eq!(value.to_string(), "12.5");
    }

    #[test]
    fn ukrainian_presentation_localizes_builtin_and_function_markers() {
        let mut session = Session::default();
        let builtin = eval_program("atom", &mut session).expect("atom value").value;
        assert_eq!(
            render_value_for_presentation(&builtin, PresentationLanguage::Ukrainian),
            "#<вбудована атом?>"
        );
        let closure = eval_program("(lambda (x) x)", &mut session).expect("closure").value;
        assert_eq!(
            render_value_for_presentation(&closure, PresentationLanguage::Ukrainian),
            "<функція>"
        );
    }

    #[test]
    fn ukrainian_error_presentation_has_no_mixed_line_label() {
        let source = "(car 5)";
        let mut session = Session::default();
        let error = eval_program(source, &mut session).expect_err("type error");
        let rendered = render_error_for_presentation(
            &error,
            source,
            PresentationLanguage::Ukrainian,
        );
        assert!(rendered.contains("Помилка типу"), "{rendered}");
        assert!(rendered.contains("перше очікує непорожній список"), "{rendered}");
        assert!(rendered.contains("рядок 1, стовпець 1"), "{rendered}");
        assert!(!rendered.contains("line/riadok/Zeile"), "{rendered}");
        assert!(!rendered.contains("car expects"), "{rendered}");
    }

    #[test]
    fn ukrainian_unknown_symbol_and_parser_errors_are_human_facing() {
        let unknown_source = "(cons невідоме ())";
        let mut session = Session::default();
        let unknown = eval_program(unknown_source, &mut session).expect_err("unknown symbol");
        let rendered = render_error_for_presentation(
            &unknown,
            unknown_source,
            PresentationLanguage::Ukrainian,
        );
        assert!(rendered.contains("Невідомий символ: невідоме"), "{rendered}");

        let parse_source = ")";
        let parse_error = parse(parse_source).expect_err("parse error");
        let rendered = render_error_for_presentation(
            &parse_error,
            parse_source,
            PresentationLanguage::Ukrainian,
        );
        assert!(rendered.contains("Синтаксична помилка"), "{rendered}");
        assert!(rendered.contains("неочікувана закривна дужка"), "{rendered}");
    }

    #[test]
    fn ukrainian_exact_arity_names_canonical_primitive_by_surface_name() {
        let source = "(атом?)";
        let mut session = Session::default();
        let error = eval_program(source, &mut session).expect_err("arity");
        let rendered = render_error_for_presentation(
            &error,
            source,
            PresentationLanguage::Ukrainian,
        );
        assert!(rendered.contains("атом?: очікувалося 1; отримано 0"), "{rendered}");
    }

    #[test]
    fn raw_user_strings_and_symbols_are_not_translated() {
        let string = Value::String("Hello".into());
        let symbol = Value::Symbol("London".into());
        assert_eq!(
            render_value_for_presentation(&string, PresentationLanguage::Ukrainian),
            "\"Hello\""
        );
        assert_eq!(
            render_value_for_presentation(&symbol, PresentationLanguage::Ukrainian),
            "London"
        );

        let error = LanguageError::new(ErrorKind::Type, "boom", Span { start: 0, end: 1 });
        let rendered = render_error_for_presentation(
            &error,
            "x",
            PresentationLanguage::Canonical,
        );
        assert!(rendered.starts_with("boom"));
    }
}
'''

Path("crates/my-lisp/src/presentation.rs").write_text(presentation, encoding="utf-8")

replace_once(
    "crates/my-lisp/src/lib.rs",
    "mod parser;\npub mod syntax;\nmod value;",
    "mod parser;\nmod presentation;\npub mod syntax;\nmod value;",
)
replace_once(
    "crates/my-lisp/src/lib.rs",
    "pub use parser::parse;\n",
    "pub use parser::parse;\npub use presentation::{\n    present_system_message, render_error_for_presentation, render_value_for_presentation,\n    PresentationLanguage,\n};\n",
)

replace_once(
    "lib/surface/uk.my",
    "(define атом? atom)\n(define тотожне? eq)\n(define сполучити cons)\n(define перше car)\n(define решта cdr)\n",
    "(define атом? atom)\n(define тотожне? eq)\n(define сполучити cons)\n(define перше car)\n(define решта cdr)\n\n; Українські імена тих самих канонічних значень істини й хиби.\n; Це не нові Boolean-об'єкти: істина є тим самим символом t, а хиба — тим\n; самим Canon 0 `()`. Presentation-layer показує t як `істина` лише в UK REPL.\n(define істина t)\n(define хиба (quote ()))\n",
)

replace_once(
    "crates/my-lisp-cli/src/repl.rs",
    "    eval_parsed_expressions_incremental, eval_program, parse, Environment, ErrorKind, ExprKind,\n    Session,\n",
    "    eval_parsed_expressions_incremental, eval_program, parse, render_error_for_presentation,\n    render_value_for_presentation, Environment, ErrorKind, ExprKind, PresentationLanguage, Session,\n",
)
replace_once(
    "crates/my-lisp-cli/src/repl.rs",
    "    fn title(self) -> &'static str {\n        match self {\n            Self::Core => \"ядро\",\n            Self::English => \"англійська\",\n            Self::Ukrainian => \"українська\",\n            Self::Sanskrit => \"санскрит\",\n        }\n    }\n",
    "    fn title(self) -> &'static str {\n        match self {\n            Self::Core => \"ядро\",\n            Self::English => \"англійська\",\n            Self::Ukrainian => \"українська\",\n            Self::Sanskrit => \"санскрит\",\n        }\n    }\n\n    fn presentation(self) -> PresentationLanguage {\n        match self {\n            Self::Core => PresentationLanguage::Canonical,\n            Self::English => PresentationLanguage::English,\n            Self::Ukrainian => PresentationLanguage::Ukrainian,\n            Self::Sanskrit => PresentationLanguage::Sanskrit,\n        }\n    }\n",
)
replace_once(
    "crates/my-lisp-cli/src/repl.rs",
    "                                println!(\"{}\", result.value);\n",
    "                                println!(\n                                    \"{}\",\n                                    render_value_for_presentation(\n                                        &result.value,\n                                        state.surface.presentation(),\n                                    )\n                                );\n",
)
replace_once(
    "crates/my-lisp-cli/src/repl.rs",
    "                                    eprintln!(\"Error: {}\", e.render(line));\n",
    "                                    eprintln!(\n                                        \"{}\",\n                                        render_error_for_presentation(\n                                            &e,\n                                            line,\n                                            state.surface.presentation(),\n                                        )\n                                    );\n",
)
replace_once(
    "crates/my-lisp-cli/src/repl.rs",
    "                    Err(e) => eprintln!(\"Parse error: {}\", e.render(line)),\n",
    "                    Err(e) => eprintln!(\n                        \"{}\",\n                        render_error_for_presentation(\n                            &e,\n                            line,\n                            state.surface.presentation(),\n                        )\n                    ),\n",
)
replace_once(
    "crates/my-lisp-cli/src/repl.rs",
    "        assert_eq!(value(&mut state, \"(атом? 'мама)\"), \"t\");\n        assert!(state.session.environment.get(\"атом?\").is_some());\n",
    "        assert_eq!(value(&mut state, \"(атом? 'мама)\"), \"t\");\n        let truth = eval_program(\"(атом? 'мама)\", &mut state.session)\n            .expect(\"uk predicate\")\n            .value;\n        assert_eq!(\n            render_value_for_presentation(&truth, state.surface.presentation()),\n            \"істина\"\n        );\n        assert!(state.session.environment.get(\"атом?\").is_some());\n        assert_eq!(value(&mut state, \"істина\"), \"t\");\n        assert_eq!(value(&mut state, \"хиба\"), \"()\");\n",
)

replace_once(
    "crates/my-lisp-wasm/src/lib.rs",
    "use my_lisp::{eval_program, Environment, Session};\n",
    "use my_lisp::{\n    eval_program, present_system_message, render_error_for_presentation,\n    render_value_for_presentation, Environment, PresentationLanguage, Session,\n};\n",
)
replace_once(
    "crates/my-lisp-wasm/src/lib.rs",
    "    fn code(self) -> &'static str {\n        match self {\n            Self::Core => \"core\",\n            Self::English => \"en\",\n            Self::Ukrainian => \"ук\",\n            Self::Sanskrit => \"sa\",\n        }\n    }\n",
    "    fn code(self) -> &'static str {\n        match self {\n            Self::Core => \"core\",\n            Self::English => \"en\",\n            Self::Ukrainian => \"ук\",\n            Self::Sanskrit => \"sa\",\n        }\n    }\n\n    fn presentation(self) -> PresentationLanguage {\n        match self {\n            Self::Core => PresentationLanguage::Canonical,\n            Self::English => PresentationLanguage::English,\n            Self::Ukrainian => PresentationLanguage::Ukrainian,\n            Self::Sanskrit => PresentationLanguage::Sanskrit,\n        }\n    }\n",
)
replace_once(
    "crates/my-lisp-wasm/src/lib.rs",
    "        let (result, forms) =\n            my_lisp_literate::eval_literate(source, source_mode, &mut state.session)\n                .map_err(|e| JsValue::from_str(&e.to_string()))?;\n\n        let evaluation = Evaluation {\n            value: result.value.to_string(),\n",
    "        let (result, forms) =\n            my_lisp_literate::eval_literate(source, source_mode, &mut state.session).map_err(|e| {\n                JsValue::from_str(&render_error_for_presentation(\n                    &e,\n                    source,\n                    state.surface.presentation(),\n                ))\n            })?;\n\n        let evaluation = Evaluation {\n            value: render_value_for_presentation(&result.value, state.surface.presentation()),\n",
)
replace_once(
    "crates/my-lisp-wasm/src/lib.rs",
    "fn diagnose_impl(source: &str, is_literate: bool) -> Vec<Diagnostic> {\n",
    "fn diagnose_impl(source: &str, is_literate: bool) -> Vec<Diagnostic> {\n    diagnose_impl_with_presentation(source, is_literate, PresentationLanguage::Canonical)\n}\n\nfn diagnose_impl_with_presentation(\n    source: &str,\n    is_literate: bool,\n    presentation: PresentationLanguage,\n) -> Vec<Diagnostic> {\n",
)
replace_once(
    "crates/my-lisp-wasm/src/lib.rs",
    "            message: e.to_string(),\n",
    "            message: present_system_message(&e.message, presentation),\n",
)
replace_once(
    "crates/my-lisp-wasm/src/lib.rs",
    "                        message: d.message,\n",
    "                        message: present_system_message(&d.message, presentation),\n",
)
replace_once(
    "crates/my-lisp-wasm/src/lib.rs",
    "pub fn diagnose(source: &str, mode: JsValue) -> JsValue {\n    let mode_str = mode.as_string().unwrap_or_default();\n    let is_literate = mode_str == \"markdown\";\n    serde_wasm_bindgen::to_value(&diagnose_impl(source, is_literate)).unwrap_or(JsValue::NULL)\n}\n",
    "pub fn diagnose(source: &str, mode: JsValue) -> JsValue {\n    let mode_str = mode.as_string().unwrap_or_default();\n    let is_literate = mode_str == \"markdown\";\n    let presentation = if init_if_needed().is_ok() {\n        SESSION.with(|slot| {\n            slot.borrow()\n                .as_ref()\n                .map(|state| state.surface.presentation())\n                .unwrap_or(PresentationLanguage::Canonical)\n        })\n    } else {\n        PresentationLanguage::Canonical\n    };\n    serde_wasm_bindgen::to_value(&diagnose_impl_with_presentation(\n        source,\n        is_literate,\n        presentation,\n    ))\n    .unwrap_or(JsValue::NULL)\n}\n",
)
replace_once(
    "crates/my-lisp-wasm/src/lib.rs",
    "            assert_eq!(uk.value.to_string(), \"t\");\n",
    "            assert_eq!(uk.value.to_string(), \"t\");\n            assert_eq!(\n                render_value_for_presentation(&uk.value, WebSurface::Ukrainian.presentation()),\n                \"істина\"\n            );\n",
)

replace_once(
    "public/my-lisp-cli-web.html",
    "          printLine(\"Error: \" + (error && error.message ? error.message : String(error)), \"line-error\");\n",
    "          const message = error && error.message ? error.message : String(error);\n          printLine(currentSurface === \"ук\" ? message : \"Error: \" + message, \"line-error\");\n",
)

replace_once(
    "crates/my-lisp/tests/ukrainian_api_docs.rs",
    "    eval_program(UK_SURFACE, &mut session).expect(\"українська поверхня має завантажитися\");\n    session\n}\n",
    "    eval_program(UK_SURFACE, &mut session).expect(\"українська поверхня має завантажитися\");\n    session\n}\n\n#[test]\nfn istina_i_khyba_ie_imenamy_tyh_samykh_kanonichnykh_znachen() {\n    let mut session = uk_session();\n    assert_eq!(\n        eval_program(\"істина\", &mut session)\n            .expect(\"істина має бути доступна\")\n            .value\n            .to_string(),\n        \"t\"\n    );\n    assert_eq!(\n        eval_program(\"хиба\", &mut session)\n            .expect(\"хиба має бути доступна\")\n            .value\n            .to_string(),\n        \"()\"\n    );\n}\n",
)

replace_once(
    "docs/repl-surfaces.md",
    "Ключовий принцип: **одна семантика, кілька поверхонь, незмінний користувацький стан.**",
    "## Українське подання результатів\n\nПоверхня `ук` тепер є двосторонньою: вона локалізує не лише імена у вводі, а й системне подання результату REPL та діагностик. Канонічні значення при цьому не змінюються.\n\n```text\nmy-lisp[ук]> (атом? 'мама)\nістина\n\nmy-lisp[ук]> (перше 5)\nПомилка типу: перше очікує непорожній список\n --> рядок 1, стовпець 1\n1 | (перше 5)\n  | ^^^^^^^^^^\n```\n\n`істина` є українським ім'ям того самого канонічного символу `t`; `хиба` — ім'ям того самого Canon 0 `()`. Саме `()` у виводі не перейменовується, бо це водночас хибність і порожній список: після обчислення ці ролі структурно нерозрізнювані.\n\nУкраїнське presentation-подання також використовує десяткову кому для inexact-чисел і локалізує службові маркери на кшталт `#<вбудована ...>`, `<функція>`, `<макрос>`, `рядок`, `стовпець`. Рядки й користувацькі символи не перекладаються: `\"Hello\"` лишається `\"Hello\"`, `'London` лишається `London`.\n\nВажлива межа: це **подання інтерактивної оболонки**, а не переписування даних програми. Канонічний `Value::Display`, conformance-вивід, протокольні ідентифікатори та явний програмний `print` лишаються машинно стабільними.\n\nКлючовий принцип: **одна семантика, кілька поверхонь, незмінний користувацький стан і локалізоване людське подання.**",
)

print("uk presentation patch applied")

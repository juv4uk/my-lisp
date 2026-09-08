//! Інтерактивний stdio REPL: історія, interaction-only echo fallback і
//! перемикання програмних поверхонь. `:мова` / `:surface` не є Lisp syntax:
//! це команди оболонки над одним і тим самим семантичним ядром.

use my_lisp::syntax::Expr;
use my_lisp::{
    eval_parsed_expressions_incremental, eval_program, parse, render_error_for_presentation,
    render_value_for_presentation, Environment, ErrorKind, ExprKind, PresentationLanguage, Session,
};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;
use std::path::PathBuf;
use std::process;

const SURFACE_PREREQUISITES: &[(&str, &str)] = &[
    ("unify.my", include_str!("../../../lib/unify.my")),
    ("reason.my", include_str!("../../../lib/reason.my")),
    ("forward.my", include_str!("../../../lib/forward.my")),
    ("knowledge.my", include_str!("../../../lib/knowledge.my")),
    (
        "persistent-map.my",
        include_str!("../../../lib/persistent-map.my"),
    ),
    (
        "persistent-vector.my",
        include_str!("../../../lib/persistent-vector.my"),
    ),
    ("time.my", include_str!("../../../lib/time.my")),
    ("epistemic.my", include_str!("../../../lib/epistemic.my")),
];
const UK_SURFACE: &str = include_str!("../../../lib/surface/uk.my");
const SA_SURFACE: &str = include_str!("../../../lib/surface/sa.my");
const UK_API_DOCS: &str = include_str!("../../../lib/surface/uk-docs.wsm");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReplSurface {
    Core,
    English,
    Ukrainian,
    Sanskrit,
}

impl ReplSurface {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "core" | "ядро" => Some(Self::Core),
            "en" | "english" | "англійська" => Some(Self::English),
            "uk" | "ук" | "українська" => Some(Self::Ukrainian),
            "sa" | "sanskrit" | "санскрит" => Some(Self::Sanskrit),
            _ => None,
        }
    }

    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::English => "en",
            Self::Ukrainian => "ук",
            Self::Sanskrit => "sa",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Core => "ядро",
            Self::English => "англійська",
            Self::Ukrainian => "українська",
            Self::Sanskrit => "санскрит",
        }
    }

    fn presentation(self) -> PresentationLanguage {
        match self {
            Self::Core => PresentationLanguage::Canonical,
            Self::English => PresentationLanguage::English,
            Self::Ukrainian => PresentationLanguage::Ukrainian,
            Self::Sanskrit => PresentationLanguage::Sanskrit,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SurfaceDoc {
    category: String,
    canonical: String,
    name: String,
    kind: String,
    call: String,
    description: String,
}

fn expr_list(expr: &Expr) -> Option<&[Expr]> {
    match &expr.kind {
        ExprKind::List(items) => Some(items.as_ref()),
        _ => None,
    }
}

fn expr_symbol(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Symbol(symbol) => Some(symbol.as_ref()),
        _ => None,
    }
}

fn expr_string(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::String(text) => Some(text.as_ref()),
        _ => None,
    }
}

fn expr_usize(expr: &Expr) -> Option<usize> {
    match &expr.kind {
        ExprKind::Number(value, _) if value.is_finite() && *value >= 0.0 && value.fract() == 0.0 => {
            Some(*value as usize)
        }
        _ => None,
    }
}

fn ukrainian_api_docs() -> Result<Vec<SurfaceDoc>, String> {
    let program = parse(UK_API_DOCS)
        .map_err(|error| format!("не вдалося прочитати uk-docs.wsm: {}", error.render(UK_API_DOCS)))?;
    let root = program
        .first()
        .and_then(expr_list)
        .ok_or_else(|| "uk-docs.wsm: очікувався кореневий список".to_string())?;
    if expr_symbol(root.first().ok_or_else(|| "uk-docs.wsm: порожній корінь".to_string())?)
        != Some("uk-api-docs")
    {
        return Err("uk-docs.wsm: невідомий кореневий тег".to_string());
    }

    let expected_count = root
        .iter()
        .filter_map(expr_list)
        .find(|items| items.first().and_then(expr_symbol) == Some("count"))
        .and_then(|items| items.get(1))
        .and_then(expr_usize)
        .ok_or_else(|| "uk-docs.wsm: відсутній коректний count".to_string())?;

    let docs_form = root
        .iter()
        .filter_map(expr_list)
        .find(|items| items.first().and_then(expr_symbol) == Some("docs"))
        .ok_or_else(|| "uk-docs.wsm: відсутня секція docs".to_string())?;

    let mut docs = Vec::with_capacity(expected_count);
    for entry in docs_form.iter().skip(1) {
        let fields = expr_list(entry).ok_or_else(|| "uk-docs.wsm: doc має бути списком".to_string())?;
        if fields.len() != 7 || fields.first().and_then(expr_symbol) != Some("doc") {
            return Err("uk-docs.wsm: некоректний doc-запис".to_string());
        }
        docs.push(SurfaceDoc {
            category: expr_symbol(&fields[1])
                .ok_or_else(|| "uk-docs.wsm: category має бути символом".to_string())?
                .to_string(),
            canonical: expr_symbol(&fields[2])
                .ok_or_else(|| "uk-docs.wsm: canonical має бути символом".to_string())?
                .to_string(),
            name: expr_symbol(&fields[3])
                .ok_or_else(|| "uk-docs.wsm: українське ім'я має бути символом".to_string())?
                .to_string(),
            kind: expr_symbol(&fields[4])
                .ok_or_else(|| "uk-docs.wsm: kind має бути символом".to_string())?
                .to_string(),
            call: expr_string(&fields[5])
                .ok_or_else(|| "uk-docs.wsm: call має бути рядком".to_string())?
                .to_string(),
            description: expr_string(&fields[6])
                .ok_or_else(|| "uk-docs.wsm: description має бути рядком".to_string())?
                .to_string(),
        });
    }

    if docs.len() != expected_count {
        return Err(format!(
            "uk-docs.wsm: count={expected_count}, але прочитано {} doc-записів",
            docs.len()
        ));
    }
    Ok(docs)
}

fn category_title(category: &str) -> &str {
    match category {
        "canon" => "Канон 0+7",
        "forms" => "Форми та макроси",
        "arithmetic" => "Арифметика",
        "comparison" => "Порівняння",
        "predicate" => "Предикати",
        "list" => "Списки",
        "higher-order" => "Функції вищого порядку",
        "string" => "Текст і символи",
        "io" => "Читання, обчислення та вивід",
        "vector" => "Звичайні вектори",
        "time" => "Час",
        "persistent-map" => "Персистентні карти",
        "persistent-vector" => "Персистентні вектори",
        "knowledge" => "Знання",
        "reasoning" => "Логічне міркування",
        "unification" => "Уніфікація",
        "epistemic" => "Епістемічні структури",
        "other" => "Інші засоби",
        other => other,
    }
}

fn kind_title(kind: &str) -> &str {
    match kind {
        "form" => "форма",
        "macro" => "макрос",
        "function" => "функція",
        "predicate" => "предикат",
        "mutation" => "мутація",
        "value" => "значення",
        other => other,
    }
}

fn render_surface_names(surface: ReplSurface) -> Result<String, String> {
    if surface != ReplSurface::Ukrainian {
        return Ok(format!(
            "Публічний машинний каталог для поверхні «{}» ще не визначений.\n\
             Перемкніться на :мова ук для українського каталогу; (env)/(середовище) лишається сирою інтроспекцією середовища.",
            surface.title()
        ));
    }

    let docs = ukrainian_api_docs()?;
    let mut output = format!(
        "Публічна українська поверхня: {} preferred stable імен\n\
         Джерело: lib/surface/uk-docs.wsm\n\
         (середовище) показує всі реально видимі bindings, включно з базовими, compatibility та internal.\n",
        docs.len()
    );

    let mut current_category: Option<&str> = None;
    for doc in &docs {
        if current_category != Some(doc.category.as_str()) {
            current_category = Some(doc.category.as_str());
            output.push('\n');
            output.push_str(category_title(&doc.category));
            output.push_str(":\n  ");
        } else {
            output.push_str(" · ");
        }
        output.push_str(&doc.name);
    }
    Ok(output)
}

fn render_surface_name(surface: ReplSurface, requested: &str) -> Result<String, String> {
    if surface != ReplSurface::Ukrainian {
        return Ok(format!(
            "Команда :ім'я зараз має машинний каталог лише для української поверхні. Поточна: {} ({}).",
            surface.title(),
            surface.code()
        ));
    }

    let docs = ukrainian_api_docs()?;
    let Some(doc) = docs
        .iter()
        .find(|doc| doc.name == requested || doc.canonical == requested)
    else {
        return Ok(format!(
            "«{requested}» не входить до preferred stable українського API.\n\
             Воно може бути compatibility/internal binding; перевірити сире lexical середовище можна через (середовище)."
        ));
    };

    Ok(format!(
        "{}\n  статус: preferred stable\n  категорія: {}\n  тип: {}\n  основа: {}\n  виклик: {}\n  {}",
        doc.name,
        category_title(&doc.category),
        kind_title(&doc.kind),
        doc.canonical,
        doc.call,
        doc.description
    ))
}

fn build_surface_layer(base: &Environment, surface: ReplSurface) -> Result<Environment, String> {
    let layer = base.child();
    if matches!(surface, ReplSurface::Ukrainian | ReplSurface::Sanskrit) {
        let mut session = Session {
            environment: layer.clone(),
        };
        for (name, source) in SURFACE_PREREQUISITES {
            eval_program(source, &mut session).map_err(|error| {
                format!("не вдалося завантажити {name}: {}", error.render(source))
            })?;
        }
        let (name, source) = match surface {
            ReplSurface::Ukrainian => ("uk.my", UK_SURFACE),
            ReplSurface::Sanskrit => ("sa.my", SA_SURFACE),
            ReplSurface::Core | ReplSurface::English => unreachable!(),
        };
        eval_program(source, &mut session)
            .map_err(|error| format!("не вдалося завантажити {name}: {}", error.render(source)))?;
    }
    Ok(layer)
}

struct ReplState {
    session: Session,
    base_environment: Environment,
    user_environment: Environment,
    surface: ReplSurface,
}

impl ReplState {
    fn new(mut session: Session, surface: ReplSurface) -> Result<Self, String> {
        let base_environment = session.environment.clone();
        let surface_environment = build_surface_layer(&base_environment, surface)?;
        let user_environment = surface_environment.child();
        session.environment = user_environment.clone();
        Ok(Self {
            session,
            base_environment,
            user_environment,
            surface,
        })
    }

    fn switch_surface(&mut self, surface: ReplSurface) -> Result<(), String> {
        if self.surface == surface {
            return Ok(());
        }
        let surface_environment = build_surface_layer(&self.base_environment, surface)?;
        self.user_environment
            .reparent(surface_environment)
            .map_err(str::to_string)?;
        self.surface = surface;
        Ok(())
    }
}

/// `~/.my-lisp-history`, якщо home directory доступний.
pub(crate) fn history_path() -> Option<PathBuf> {
    let home = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".my-lisp-history"))
}

fn print_surface_help() {
    println!("Поверхні: :мова ук | en | sa | core");
    println!("Технічний alias: :surface uk | en | sa | core");
    println!("Публічний словник поточної поверхні: :імена; одна назва: :ім'я <назва>");
    println!("Сире lexical середовище без фільтрації поверхнею: (середовище) / (env)");
    println!("Перемикання змінює лише surface-frame; ваші define/closures лишаються живими.");
}

fn handle_meta_command(line: &str, state: &mut ReplState) -> bool {
    let mut parts = line.split_whitespace();
    let Some(command) = parts.next() else {
        return false;
    };

    match command {
        ":мова" | ":surface" => {
            let Some(requested) = parts.next() else {
                println!(
                    "Поточна поверхня: {} ({})",
                    state.surface.title(),
                    state.surface.code()
                );
                print_surface_help();
                return true;
            };
            if parts.next().is_some() {
                eprintln!("Поверхня приймає рівно одне ім'я.");
                print_surface_help();
                return true;
            }
            let Some(surface) = ReplSurface::parse(requested) else {
                eprintln!("Невідома поверхня: {requested}");
                print_surface_help();
                return true;
            };
            match state.switch_surface(surface) {
                Ok(()) => println!("Поверхня: {} ({})", surface.title(), surface.code()),
                Err(error) => eprintln!("Помилка перемикання поверхні: {error}"),
            }
            true
        }
        ":імена" | ":names" => {
            if parts.next().is_some() {
                eprintln!("Команда :імена не приймає аргументів.");
                return true;
            }
            match render_surface_names(state.surface) {
                Ok(output) => println!("{output}"),
                Err(error) => eprintln!("Помилка каталогу поверхні: {error}"),
            }
            true
        }
        ":ім'я" | ":name" => {
            let Some(requested) = parts.next() else {
                eprintln!("Використання: :ім'я <назва>");
                return true;
            };
            if parts.next().is_some() {
                eprintln!("Команда :ім'я приймає рівно одну назву.");
                return true;
            }
            match render_surface_name(state.surface, requested) {
                Ok(output) => println!("{output}"),
                Err(error) => eprintln!("Помилка каталогу поверхні: {error}"),
            }
            true
        }
        ":допомога" | ":help" => {
            print_surface_help();
            true
        }
        _ => false,
    }
}

pub(crate) fn run_repl(session: Session, initial_surface: ReplSurface) {
    let mut state = match ReplState::new(session, initial_surface) {
        Ok(state) => state,
        Err(error) => {
            eprintln!("Error: could not initialize REPL surface: {error}");
            process::exit(1);
        }
    };

    println!("my-lisp REPL v{} (pure Rust)", env!("CARGO_PKG_VERSION"));
    println!(
        "Поверхня: {} ({}) · змінити: :мова ук|en|sa|core · :допомога",
        state.surface.title(),
        state.surface.code()
    );
    println!("Ctrl-C або Ctrl-D — вихід.");

    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(err) => {
            eprintln!("Error: could not start the REPL line editor: {err}");
            process::exit(1);
        }
    };

    let history_path = history_path();
    if let Some(path) = &history_path {
        let _ = rl.load_history(path);
    }

    loop {
        let prompt = format!("my-lisp[{}]> ", state.surface.code());
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(line);
                if let Some(path) = &history_path {
                    let _ = rl.append_history(path);
                }

                if handle_meta_command(line, &mut state) {
                    continue;
                }

                match parse(line) {
                    Ok(ast) => {
                        match eval_parsed_expressions_incremental(&ast, &mut state.session) {
                            Ok(result) => {
                                for out in result.output {
                                    println!("{out}");
                                }
                                println!(
                                    "{}",
                                    render_value_for_presentation(
                                        &result.value,
                                        state.surface.presentation(),
                                    )
                                );
                            }
                            Err(e) => {
                                // Це лише interaction policy: невідомий standalone symbol
                                // вітається через `echo`, але всередині справжньої форми
                                // UnknownSymbol лишається звичайною мовною помилкою.
                                if e.kind == ErrorKind::UnknownSymbol
                                    && ast.len() == 1
                                    && matches!(ast[0].kind, ExprKind::Symbol(_))
                                {
                                    println!("echo {line}");
                                } else {
                                    eprintln!(
                                        "{}",
                                        render_error_for_presentation(
                                            &e,
                                            line,
                                            state.surface.presentation(),
                                        )
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!(
                        "{}",
                        render_error_for_presentation(&e, line, state.surface.presentation(),)
                    ),
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {err:?}");
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn core_state() -> ReplState {
        let mut session = Session::default();
        my_lisp::load_core_library(&mut session).expect("core bootstrap");
        ReplState::new(session, ReplSurface::Core).expect("REPL state")
    }

    fn value(state: &mut ReplState, source: &str) -> String {
        eval_program(source, &mut state.session)
            .unwrap_or_else(|error| panic!("{source}: {}", error.render(source)))
            .value
            .to_string()
    }

    #[test]
    fn ukrainian_surface_is_real_and_disappears_when_switching_back_to_core() {
        let mut state = core_state();
        state.switch_surface(ReplSurface::Ukrainian).expect("uk");
        assert_eq!(value(&mut state, "(атом? 'мама)"), "t");
        let truth = eval_program("(атом? 'мама)", &mut state.session)
            .expect("uk predicate")
            .value;
        assert_eq!(
            render_value_for_presentation(&truth, state.surface.presentation()),
            "істина"
        );
        assert!(state.session.environment.get("атом?").is_some());
        assert_eq!(value(&mut state, "істина"), "t");
        assert_eq!(value(&mut state, "хиба"), "()");

        state.switch_surface(ReplSurface::Core).expect("core");
        assert!(state.session.environment.get("атом?").is_none());
        assert_eq!(value(&mut state, "(atom 'мама)"), "t");
    }

    #[test]
    fn switching_surface_preserves_user_frame_and_closure_view() {
        let mut state = core_state();
        value(&mut state, "(define крок 1)");
        value(&mut state, "(define додай-крок (lambda (x) (+ x крок)))");
        assert_eq!(value(&mut state, "(додай-крок 5)"), "6");

        state.switch_surface(ReplSurface::Ukrainian).expect("uk");
        value(&mut state, "(define крок 2)");
        assert_eq!(value(&mut state, "(додай-крок 5)"), "7");

        state.switch_surface(ReplSurface::English).expect("en");
        assert_eq!(value(&mut state, "(додай-крок 5)"), "7");
    }

    #[test]
    fn ukrainian_catalog_is_read_from_machine_docs_and_uses_preferred_names() {
        let docs = ukrainian_api_docs().expect("machine-readable UK docs");
        let member = docs
            .iter()
            .find(|doc| doc.canonical == "member?")
            .expect("member? public mapping");
        assert_eq!(member.name, "значення-у-списку?");
        assert!(docs.iter().all(|doc| doc.name != "містить?"));

        let rendered = render_surface_names(ReplSurface::Ukrainian).expect("render UK catalog");
        assert!(rendered.contains("значення-у-списку?"));
        assert!(rendered.contains("Джерело: lib/surface/uk-docs.wsm"));
    }

    #[test]
    fn surface_name_help_distinguishes_public_catalog_from_raw_bindings() {
        let preferred = render_surface_name(ReplSurface::Ukrainian, "значення-у-списку?")
            .expect("preferred name help");
        assert!(preferred.contains("статус: preferred stable"));
        assert!(preferred.contains("основа: member?"));

        let legacy = render_surface_name(ReplSurface::Ukrainian, "містить?")
            .expect("legacy name result");
        assert!(legacy.contains("не входить до preferred stable українського API"));
        assert!(legacy.contains("(середовище)"));
    }

    #[test]
    fn raw_environment_remains_truthful_under_ukrainian_surface() {
        let mut state = core_state();
        state.switch_surface(ReplSurface::Ukrainian).expect("uk");
        let snapshot = state.session.environment.snapshot();
        assert!(snapshot.iter().any(|(name, _)| name.as_ref() == "atom"));
        assert!(snapshot.iter().any(|(name, _)| name.as_ref() == "атом?"));
    }
}

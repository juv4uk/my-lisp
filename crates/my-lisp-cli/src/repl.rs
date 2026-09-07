//! Інтерактивний stdio REPL: історія, interaction-only echo fallback і
//! перемикання програмних поверхонь. `:мова` / `:surface` не є Lisp syntax:
//! це команди оболонки над одним і тим самим семантичним ядром.

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
}

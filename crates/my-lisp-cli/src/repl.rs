//! repl.rs - the interactive stdio REPL: rustyline editing, persistent
//! history, and the interactive-only `echo <symbol>` fallback for unknown
//! standalone symbols. Moved verbatim from main.rs's inline else-branch
//! (2026-08-22 mechanical split); it takes the prepared Session so main
//! stays a dispatcher.

use my_lisp::{eval_parsed_expressions_incremental, parse, ErrorKind, ExprKind, Session};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;
use std::path::PathBuf;
use std::process;

/// `~/.my-lisp-history`, if a home directory can be found. REPL history
/// persistence is best-effort: without a home directory (or if writing
/// fails) the REPL still works, it just starts each session with no
/// remembered history.
pub(crate) fn history_path() -> Option<PathBuf> {
    let home = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".my-lisp-history"))
}

pub(crate) fn run_repl(mut session: Session) {
        println!("my-lisp REPL v{} (pure Rust)", env!("CARGO_PKG_VERSION"));
        println!("Press Ctrl-C or Ctrl-D to exit.");

        // rustyline can fail to init on an unusual terminal (e.g. no TTY); report it
        // cleanly instead of panicking, so a redirected/CI invocation exits with a message.
        // rustyline може не ініціалізуватися на нетиповому терміналі (напр. без TTY);
        // повідомляємо про це чисто замість паніки, щоб перенаправлений/CI-запуск завершився з повідомленням.
        // rustyline kann bei einem ungewöhnlichen Terminal (z. B. ohne TTY) fehlschlagen;
        // das wird sauber gemeldet statt einen Panic auszulösen, damit ein umgeleiteter/CI-Aufruf mit Meldung endet.
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
            let readline = rl.readline("my-lisp> ");
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

                    match parse(line) {
                        Ok(ast) => {
                            match eval_parsed_expressions_incremental(&ast, &mut session) {
                                Ok(result) => {
                                    for out in result.output {
                                        println!("{}", out);
                                    }
                                    println!("{}", result.value);
                                }
                                Err(e) => {
                                    // Echo fallback: an interactive REPL greets an
                                    // unknown standalone symbol with `echo <it>` instead
                                    // of an error — an interaction policy, NOT language
                                    // semantics. It fires only when the whole input is
                                    // a single top-level Symbol that the evaluator
                                    // genuinely couldn't resolve; unknown symbols inside
                                    // real forms, and every other error kind, keep the
                                    // exact same named failure (S2) as file execution.
                                    // (Non-ASCII identifiers like `мама` reach this
                                    // exactly the same way as `hello`.)
                                    //
                                    // Echo-fallback: interaktyvnyi REPL vitaye nevidomyi
                                    // okremyi symvol `echo <nei>` zamist pomylky — tse
                                    // polityka vzaiemodii, NE semantyka movy. Spraciovuie
                                    // lyshe koly ves vkhid — odyn verkhno-rivnevyi Symbol,
                                    // yakyi evaluator spravdi ne zmih rozviazaty; nevidomi
                                    // symvoly vseredyni spravzhnikh form i vsi inshi vydy
                                    // pomylok zberihaiut tochno toi samyi nazvanyi proval
                                    // (S2), shcho i vykonannia failu.
                                    if e.kind == ErrorKind::UnknownSymbol
                                        && ast.len() == 1
                                        && matches!(ast[0].kind, ExprKind::Symbol(_))
                                    {
                                        println!("echo {}", line);
                                    } else {
                                        eprintln!("Error: {}", e.render(line));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Parse error: {}", e.render(line));
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl-C
                    break;
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl-D
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }
}

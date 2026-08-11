use my_lisp::{eval_parsed_expressions, parse, Environment, Session, Value};
use std::rc::Rc;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, TcpListener};
use std::path::PathBuf;
use std::process;

/// `~/.my-lisp-history`, if a home directory can be found. REPL history
/// persistence is best-effort: without a home directory (or if writing
/// fails) the REPL still works, it just starts each session with no
/// remembered history.
/// `~/.my-lisp-history`, якщо домашню теку вдалось знайти. Збереження
/// історії REPL — best-effort: без домашньої теки (або якщо запис
/// падає) REPL все одно працює, просто кожна сесія стартує без
/// запам'ятованої історії.
/// `~/.my-lisp-history`, sofern ein Home-Verzeichnis gefunden werden
/// kann. Die REPL-Verlaufspersistenz ist Best-Effort: ohne
/// Home-Verzeichnis (oder wenn das Schreiben fehlschlägt) funktioniert
/// die REPL weiterhin, sie startet nur jede Sitzung ohne gespeicherten
/// Verlauf.
fn history_path() -> Option<PathBuf> {
    let home = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".my-lisp-history"))
}

/// `--allow-process=git,cargo` (PLAN.md item 21's follow-up) — the only
/// way a my-lisp program running under this CLI can ever get `process-run`
/// to succeed: `Environment::root()` defaults to disabled (see that
/// method's own comment for why), and nothing in the language itself can
/// grant this to a program that wasn't explicitly launched with it. Kept
/// as a small hand-rolled parser rather than a dependency (`clap` etc.) —
/// this crate's only external dependency today is `rustyline` for the
/// REPL line editor, and one flag doesn't justify a second.
/// `--allow-process=git,cargo` (продовження PLAN.md, пункт 21) — єдиний
/// спосіб, яким my-lisp-програма під цим CLI може взагалі отримати робочий
/// `process-run`: `Environment::root()` типово вимкнений (див. власний
/// коментар цього методу чому), і ніщо в самій мові не може дати це
/// програмі, яку не запустили явно з цим прапором. Залишено як маленький
/// власноруч написаний парсер, не залежність (`clap` тощо) — єдина
/// зовнішня залежність цього крейта сьогодні — `rustyline` для
/// REPL-редактора рядка, один прапор не виправдовує другу.
fn allowed_processes(args: &[String]) -> Vec<String> {
    args.iter()
        .find_map(|arg| arg.strip_prefix("--allow-process="))
        .map(|list| list.split(',').map(str::to_string).collect())
        .unwrap_or_default()
}

/// `--tcp` / `--tcp=PORT` — a REPL reachable over TCP instead of stdio, for
/// other local processes (e.g. a cross-session tool) to eval expressions
/// against without shelling out to the CLI per call. Bound to
/// `127.0.0.1` only — never `0.0.0.0` — since there is no authentication:
/// anything that can reach this port can eval arbitrary my-lisp, including
/// `process-run` if `--allow-process` was also passed. Loopback-only keeps
/// that blast radius to "processes already running as this user on this
/// machine", matching what the stdio REPL already allows.
/// One connection is served at a time against the same `Session` the file
/// or stdio REPL would use, so state persists across reconnects the same
/// way it persists across REPL lines.
fn run_tcp_repl(port: u16, session: &mut Session) {
    let listener = match TcpListener::bind((Ipv4Addr::LOCALHOST, port)) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("Error: could not bind TCP REPL to 127.0.0.1:{port}: {err}");
            process::exit(1);
        }
    };
    let actual_port = listener.local_addr().map(|a| a.port()).unwrap_or(port);
    println!("my-lisp TCP REPL v{} listening on 127.0.0.1:{actual_port}", env!("CARGO_PKG_VERSION"));

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(_) => continue,
        };
        let peer = stream.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "?".into());
        eprintln!("TCP REPL: connection from {peer}");

        let mut reader = BufReader::new(stream.try_clone().expect("clone TCP stream"));
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break, // connection closed
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let response = match parse(trimmed) {
                        Ok(ast) => match eval_parsed_expressions(&ast, session) {
                            Ok(result) => {
                                let mut out = String::new();
                                for line in result.output {
                                    out.push_str(&line);
                                    out.push('\n');
                                }
                                out.push_str(&result.value.to_string());
                                out
                            }
                            Err(e) => format!("Error: {}", e.render(trimmed)),
                        },
                        Err(e) => format!("Parse error: {}", e.render(trimmed)),
                    };
                    if writeln!(stream, "{response}").is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        eprintln!("TCP REPL: {peer} disconnected");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let allowed = allowed_processes(&args);
    let args: Vec<String> = args
        .into_iter()
        .filter(|arg| !arg.starts_with("--allow-process="))
        .collect();
    let environment = if allowed.is_empty() {
        Environment::root()
    } else {
        Environment::root().with_process_allowlist(allowed)
    };
    let mut session = Session { environment };

    // Load standard library
    let core_lib = include_str!("../../../lib/core.my");
    if let Ok(core_ast) = parse(core_lib) {
        let _ = eval_parsed_expressions(&core_ast, &mut session);
    }

    if args.len() > 1 {
        let arg = &args[1];
        
        if arg == "--version" || arg == "-V" || arg == "-v" {
            println!("my-lisp {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        
        if arg == "--help" || arg == "-h" {
            println!("Usage: my-lisp [file]");
            println!("If no file is provided, starts the REPL.");
            println!("\nOptions:");
            println!("  -V, --version               Print version information");
            println!("  -h, --help                  Print help information");
            println!("  --allow-process=a,b,c        Allow (process-run) to run exactly these program names");
            println!("  --tcp[=PORT]                 Serve the REPL over TCP on 127.0.0.1 (default port 9999) instead of stdio");
            return;
        }

        if arg == "--tcp" || arg.starts_with("--tcp=") {
            let port = arg
                .strip_prefix("--tcp=")
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(9999);
            run_tcp_repl(port, &mut session);
            return;
        }

        // Run file
        let filename = arg;

        // `*argv*` (PLAN.md item 21's follow-up, for scripts/release.my
        // taking a version on the command line) — everything after the
        // filename, as a my-lisp list of strings, defined before the
        // script runs. Empty when nothing follows the filename, not an
        // error — a script that wants an argument checks for that itself
        // (`(atom *argv*)`), the same way any other missing-input case in
        // this language is handled, not a special CLI-only mechanism.
        // `*argv*` (продовження PLAN.md, пункту 21, для scripts/release.my,
        // яка бере версію з командного рядка) — усе після імені файлу, як
        // my-lisp-список рядків, визначений до запуску скрипта. Порожній,
        // якщо нічого не йде після імені файлу, не помилка — скрипт, якому
        // потрібен аргумент, сам перевіряє це (`(atom *argv*)`), так само
        // як будь-який інший випадок відсутнього вводу в цій мові, не
        // окремий CLI-специфічний механізм.
        let argv = Value::list(
            args[2..]
                .iter()
                .map(|arg| Value::String(Rc::from(arg.as_str()))),
        );
        session.environment.define("*argv*", argv);

        match fs::read_to_string(filename) {
            Ok(source) => {
                match parse(&source) {
                    Ok(ast) => {
                        match eval_parsed_expressions(&ast, &mut session) {
                            Ok(result) => {
                                for out in result.output {
                                    println!("{}", out);
                                }
                                println!("{}", result.value);
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e.render(&source));
                                process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Parse error: {}", e.render(&source));
                        process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        }
    } else {
        // REPL mode
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
                            match eval_parsed_expressions(&ast, &mut session) {
                                Ok(result) => {
                                    for out in result.output {
                                        println!("{}", out);
                                    }
                                    println!("{}", result.value);
                                }
                                Err(e) => {
                                    eprintln!("Error: {}", e.render(line));
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
}

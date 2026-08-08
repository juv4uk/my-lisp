mod llm;
use my_lisp::{eval_parsed_expressions, parse, Session};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;
use std::fs;
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

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut session = Session::default();
    
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
            println!("  -V, --version  Print version information");
            println!("  -h, --help     Print help information");
            return;
        }

        // Run file
        let filename = arg;
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

                    // `:tell`/`:ask` are the NLP bridge (see `llm.rs` and `docs/vision.md`):
                    // the LLM only proposes Lisp syntax, `tell-knowledge`/`reason-in` still
                    // do the real symbolic verification.
                    // `:tell`/`:ask` — NLP-міст (див. `llm.rs` і `docs/vision.md`): LLM лише
                    // пропонує синтаксис Lisp, а справжню символьну верифікацію виконують
                    // `tell-knowledge`/`reason-in`.
                    // `:tell`/`:ask` sind die NLP-Brücke (siehe `llm.rs` und `docs/vision.md`):
                    // das LLM schlägt nur Lisp-Syntax vor, die eigentliche symbolische
                    // Verifikation übernehmen weiterhin `tell-knowledge`/`reason-in`.
                    if line.starts_with(":tell ") {
                        let text = &line[6..];
                        match llm::generate_rule(text) {
                            Ok(rules_str) => {
                                println!("🤖 LLM Translation:\n{}", rules_str);
                                // Here we evaluate it by wrapping in a (tell-knowledge ...) call
                                // which we will implement in Lisp
                                let lisp_cmd = format!("(tell-knowledge 'nlp '{})", rules_str);
                                match parse(&lisp_cmd) {
                                    Ok(ast) => {
                                        match eval_parsed_expressions(&ast, &mut session) {
                                            Ok(res) => {
                                                if res.value.to_string() == "Conflict-detected" {
                                                    eprintln!("⚠️ Conflict detected! The rule contradicts existing knowledge.");
                                                } else {
                                                    println!("✅ Knowledge added to 'nlp' module.");
                                                }
                                            }
                                            Err(e) => eprintln!("Verification Error: {}", e.render(&lisp_cmd)),
                                        }
                                    }
                                    Err(e) => eprintln!("Parse error on LLM output: {}", e.render(&rules_str)),
                                }
                            }
                            Err(e) => eprintln!("LLM Error: {}", e),
                        }
                        continue;
                    }

                    if line.starts_with(":ask ") {
                        let text = &line[5..];
                        match llm::generate_query(text) {
                            Ok(query_str) => {
                                println!("🤖 LLM Translation:\n{}", query_str);
                                let lisp_cmd = format!("(reason-in 'nlp '{})", query_str);
                                match parse(&lisp_cmd) {
                                    Ok(ast) => {
                                        match eval_parsed_expressions(&ast, &mut session) {
                                            Ok(result) => {
                                                for out in result.output { println!("{}", out); }
                                                println!("{}", result.value);
                                            }
                                            Err(e) => eprintln!("Error: {}", e.render(&lisp_cmd)),
                                        }
                                    }
                                    Err(e) => eprintln!("Parse error on LLM output: {}", e.render(&query_str)),
                                }
                            }
                            Err(e) => eprintln!("LLM Error: {}", e),
                        }
                        continue;
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

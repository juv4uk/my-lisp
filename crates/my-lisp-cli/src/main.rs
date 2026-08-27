use my_lisp::{eval_parsed_expressions, parse, Environment, Expr, Session, Value};
use std::env;
use std::fs;
use std::process;
use std::rc::Rc;
mod lsp_entry;
mod repl;
mod swarm;
mod tcp_repl;
use swarm::{dotted_alist_lookup, run_client, run_tcp_repl_sexpr};
use tcp_repl::run_tcp_repl;

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
fn main() {
    // The CLI is a trusted local surface: install the OS capability layer
    // (filesystem, process-run allowlist, TCP). The core itself ships none.
    my_lisp_host::install();
    let args: Vec<String> = env::args().collect();
    let allowed = allowed_processes(&args);
    let sexpr_protocol = args.iter().any(|a| a == "--protocol=sexpr");
    let args: Vec<String> = args
        .into_iter()
        .filter(|arg| !arg.starts_with("--allow-process=") && arg != "--protocol=sexpr")
        .collect();
    let allowed_for_tcp = allowed.clone();
    // Plain `f64`s, not a `Value` — `run_tcp_repl_sexpr` spawns one thread
    // per connection, and `Value`'s `Rc`-based sharing isn't `Send`; each
    // connection rebuilds its own `contract_version` `Value` locally from
    // these two numbers instead of cloning a shared one across threads.
    let (contract_major, contract_minor) = {
        let contract_source = include_str!("../../../language-contract.my");
        let mut throwaway = Session {
            environment: Environment::root(),
        };
        let quoted = format!("(quote {contract_source})");
        parse(&quoted)
            .ok()
            .and_then(|ast| eval_parsed_expressions(&ast, &mut throwaway).ok())
            .map(|r| r.value)
            .and_then(|v| {
                let major = dotted_alist_lookup(&v, "major")?;
                let minor = dotted_alist_lookup(&v, "minor")?;
                let Value::Number(major, _) = major else {
                    return None;
                };
                let Value::Number(minor, _) = minor else {
                    return None;
                };
                Some((major, minor))
            })
            .unwrap_or((0.0, 0.0))
    };
    let environment = if allowed.is_empty() {
        Environment::root()
    } else {
        Environment::root().with_process_allowlist(allowed)
    };
    let mut session = Session { environment };

    // Load standard library — FASL snapshot first (parse-output cache,
    // OPT-CORE-MY-AST-SNAPSHOT), text parse as the always-available fallback.
    // Invalidation: the snapshot embeds sha256(lib/core.my); any drift
    // between the compiled-in bytes and the compiled-in source flips us to
    // the parse path, never to a wrong program.
    const CORE_SRC: &str = include_str!("../../../lib/core.my");
    const CORE_FASL: &[u8] = include_bytes!("../../../lib/core.my.fasl");
    let fasl_hash_ok = my_lisp::fasl_decode_program(CORE_FASL)
        .map(|(_, hash)| hash == my_lisp::sha256_source(CORE_SRC.as_bytes()))
        .unwrap_or(false);
    if !fasl_hash_ok {
        eprintln!(
            "warning: lib/core.my.fasl is stale (source changed); run gen-fasl to regenerate"
        );
    }
    let core_expressions: Option<Vec<Expr>> = if fasl_hash_ok {
        my_lisp::fasl_decode_program(CORE_FASL).map(|(expressions, _)| expressions)
    } else {
        None
    };
    match core_expressions {
        Some(core_ast) => {
            let _ = eval_parsed_expressions(&core_ast, &mut session);
        }
        None => {
            if let Ok(core_ast) = parse(CORE_SRC) {
                let _ = eval_parsed_expressions(&core_ast, &mut session);
            }
        }
    }
    // Text form stays in scope for downstream consumers (tcp repl seed,
    // --lint path) without re-reading the file.
    #[allow(unused_variables)]
    let core_lib = CORE_SRC;

    if args.len() > 1 {
        let arg = &args[1];

        // LSP mode: forwards to the my-lisp-lsp crate's stdio entrypoint.
        if arg == "lsp" {
            lsp_entry::run();
            return;
        }

        if arg == "--version" || arg == "-V" || arg == "-v" {
            println!("my-lisp {}", env!("CARGO_PKG_VERSION"));
            return;
        }

        if arg == "--help" || arg == "-h" {
            println!("Usage: my-lisp [file]");
            println!("If no file is provided, starts the REPL.");
            println!("Canonical source extension: .wsm (.my and .lisp are supported aliases)");
            println!("\nOptions:");
            println!("  lsp                          Run the Language Server (LSP over stdio)");
            println!("  -V, --version               Print version information");
            println!("  -h, --help                  Print help information");
            println!("  --allow-process=a,b,c        Allow (process-run) to run exactly these program names");
            println!("  --lint                        Run the linter on the provided file and exit with non-zero if thresholds are exceeded");
            println!("  --tcp[=PORT]                 Serve the REPL over TCP on 127.0.0.1 (default port 9999) instead of stdio");
            println!("  --protocol=sexpr              With --tcp: strict (request (id) (op) (source)) / (response ...) envelope, no banner/prompt");
            println!("  --connect=HOST:PORT            P2P client: forward one sexpr request from stdin to a peer's TCP REPL, print the response");
            return;
        }

        if arg == "--tcp" || arg.starts_with("--tcp=") {
            let port = arg
                .strip_prefix("--tcp=")
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(9999);
            if sexpr_protocol {
                run_tcp_repl_sexpr(
                    port,
                    core_lib,
                    allowed_for_tcp,
                    contract_major,
                    contract_minor,
                );
            } else {
                run_tcp_repl(port, core_lib, &allowed_for_tcp);
            }
            return;
        }

        if arg.starts_with("--connect=") {
            let address = arg.strip_prefix("--connect=").unwrap_or_default();
            if address.is_empty() {
                eprintln!("my-lisp: --connect requires HOST:PORT");
                process::exit(1);
            }
            run_client(address);
            return;
        }

        if arg == "--lint" {
            if args.len() < 3 {
                eprintln!("Usage: my-lisp --lint <file>");
                process::exit(1);
            }
            let filename = &args[2];

            // Load linter
            let linter_lib = include_str!("../../../lib/linter.my");
            if let Ok(linter_ast) = parse(linter_lib) {
                if let Err(e) = eval_parsed_expressions(&linter_ast, &mut session) {
                    eprintln!("Error loading linter: {}", e.render(linter_lib));
                    process::exit(1);
                }
            } else {
                eprintln!("Failed to parse linter.my");
                process::exit(1);
            }

            match fs::read_to_string(filename) {
                Ok(source) => {
                    let quoted_src = format!("(quote (begin {}\n))", source);
                    match parse(&quoted_src) {
                        Ok(q_ast) => {
                            let target_ast = match eval_parsed_expressions(&q_ast, &mut session) {
                                Ok(r) => r.value,
                                Err(e) => {
                                    eprintln!(
                                        "Error evaluating quoted source: {}",
                                        e.render(&quoted_src)
                                    );
                                    process::exit(1);
                                }
                            };
                            session.environment.define("*lint-target*", target_ast);

                            let lint_call_src = "(lint-check *lint-target* (quote ((max-size . 5000) (max-nesting . 50) (max-complexity . 100) (max-globals . 500) (max-effects . 10))))";
                            match parse(lint_call_src) {
                                Ok(lint_ast) => {
                                    match eval_parsed_expressions(&lint_ast, &mut session) {
                                        Ok(result) => {
                                            if let Value::Nil = result.value {
                                                println!("Linter passed: {}", filename);
                                                return;
                                            } else {
                                                eprintln!(
                                                    "Linter violations found in {}:",
                                                    filename
                                                );
                                                eprintln!("{}", result.value);
                                                process::exit(1);
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Error running linter: {}",
                                                e.render(lint_call_src)
                                            );
                                            process::exit(1);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Parse error creating lint call: {}",
                                        e.render(lint_call_src)
                                    );
                                    process::exit(1);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Parse error in {}: {}", filename, e.render(&source));
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading file {}: {}", filename, e);
                    process::exit(1);
                }
            }
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
            Ok(source) => match parse(&source) {
                Ok(ast) => match eval_parsed_expressions(&ast, &mut session) {
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
                },
                Err(e) => {
                    eprintln!("Parse error: {}", e.render(&source));
                    process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        }
    } else {
        // REPL mode
        repl::run_repl(session);
    }
}

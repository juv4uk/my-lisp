use my_lisp::{eval_parsed_expressions, parse, Environment, ErrorKind, Exactness, Session, Value};
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
/// Each connection gets its own fresh `Session` (core.my reloaded from
/// scratch) rather than sharing one across every caller — tried the shared
/// version first, and it let one connection's `def` (accidental or not)
/// corrupt every other caller's environment with no way to trace it back.
/// `Environment` clones cheaply (`Rc<RefCell<Frame>>`) but that's exactly
/// the problem: a clone shares the underlying frame, it doesn't fork it,
/// so cloning an existing `Session` would not have fixed this — a genuinely
/// new `Environment::root()` per connection is what isolates state.
/// State does NOT persist across reconnects within the same connection is
/// fine (a single connection's lines share state, same as one REPL
/// session), but two different connections never see each other's `def`s.
/// Every expression is still logged to stderr with its peer address, for
/// the same accountability reason the isolation itself was added for.
fn run_tcp_repl(port: u16, core_lib: &str, allowed: &[String]) {
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

        let environment = if allowed.is_empty() {
            Environment::root()
        } else {
            Environment::root().with_process_allowlist(allowed.to_vec())
        };
        let mut session = Session { environment };
        if let Ok(core_ast) = parse(core_lib) {
            let _ = eval_parsed_expressions(&core_ast, &mut session);
        }

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
                    eprintln!("TCP REPL: {peer} > {trimmed}");
                    let response = match parse(trimmed) {
                        Ok(ast) => match eval_parsed_expressions(&ast, &mut session) {
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

/// Walks a `Value` list (the `Pair`-chain shape `Value::list` builds) into
/// a `Vec`, stopping at the first non-`Pair` tail. Used only for reading
/// the machine-protocol's own request/response envelopes — the language
/// itself has `car`/`cdr` for this, but the CLI here is reading a `Value`
/// that was never `def`d into a running `Session`.
fn list_items(mut value: &Value) -> Vec<Value> {
    let mut items = Vec::new();
    while let Value::Pair(head, tail) = value {
        items.push((**head).clone());
        value = tail;
    }
    items
}

/// Looks up `(key . value)` in a dotted-pair alist like
/// `language-contract.my`'s `((major . 1) (minor . 0) ...)` — distinct
/// from `list_items`' 2-element-list reading of the request/response
/// envelope, since a dotted pair's cdr is the value directly, not a
/// nested one-element list.
fn dotted_alist_lookup(alist: &Value, key: &str) -> Option<Value> {
    list_items(alist).into_iter().find_map(|item| match &item {
        Value::Pair(k, v) => match &**k {
            Value::Symbol(name) if &**name == key => Some((**v).clone()),
            _ => None,
        },
        _ => None,
    })
}

/// `output` carries every `print`/`println`-style side-effect line the
/// evaluated expression produced, in order — dropping it (the first cut
/// of this protocol did) silently discards real program output, which is
/// exactly the "optimistic" half-truth this protocol exists to prevent.
fn ok_response(id: &Value, value: Value, output: &[String], contract_version: &Value) -> Value {
    Value::list([
        Value::Symbol("response".into()),
        Value::list([Value::Symbol("id".into()), id.clone()]),
        Value::list([Value::Symbol("status".into()), Value::Symbol("ok".into())]),
        Value::list([Value::Symbol("value".into()), value]),
        Value::list([
            Value::Symbol("output".into()),
            Value::list(output.iter().map(|line| Value::String(line.as_str().into()))),
        ]),
        Value::list([Value::Symbol("contract-version".into()), contract_version.clone()]),
    ])
}

fn error_response(id: &Value, kind: &str, message: &str, contract_version: &Value) -> Value {
    Value::list([
        Value::Symbol("response".into()),
        Value::list([Value::Symbol("id".into()), id.clone()]),
        Value::list([Value::Symbol("status".into()), Value::Symbol("error".into())]),
        Value::list([Value::Symbol("kind".into()), Value::Symbol(kind.into())]),
        Value::list([Value::Symbol("message".into()), Value::String(message.into())]),
        Value::list([Value::Symbol("contract-version".into()), contract_version.clone()]),
    ])
}

/// A single `notify`d message, kept in `run_tcp_repl_sexpr`'s in-memory
/// mailbox — deliberately separate from any `Session`/`Environment`, so
/// agent coordination never touches the isolated eval-oracle state each
/// connection gets (see this function's own doc comment). `to: None`
/// means broadcast to every `poll`er.
struct MailboxEntry {
    id: u64,
    from: String,
    to: Option<String>,
    message: String,
}

fn mailbox_entry_to_value(entry: &MailboxEntry) -> Value {
    Value::list([
        Value::list([Value::Symbol("id".into()), Value::Number(entry.id as f64, Exactness::Exact)]),
        Value::list([Value::Symbol("from".into()), Value::String(entry.from.as_str().into())]),
        Value::list([
            Value::Symbol("to".into()),
            match &entry.to {
                Some(to) => Value::String(to.as_str().into()),
                None => Value::Nil,
            },
        ]),
        Value::list([Value::Symbol("message".into()), Value::String(entry.message.as_str().into())]),
    ])
}

fn error_kind_symbol(kind: &ErrorKind) -> &'static str {
    match kind {
        ErrorKind::Parse => "parse-error",
        ErrorKind::UnknownSymbol => "unknown-symbol",
        ErrorKind::Arity => "arity-error",
        ErrorKind::Type => "type-error",
        ErrorKind::InvalidForm => "invalid-form",
        ErrorKind::OutOfMemory => "out-of-memory",
        ErrorKind::NumericOverflow => "numeric-overflow",
    }
}

/// `--tcp=PORT --protocol=sexpr` — the same live oracle as `run_tcp_repl`,
/// but for machines instead of humans: no banner, no prompt, one strict
/// `(request (id ..) (op ..) (source ..))` in, one `(response (id ..)
/// (status ..) ..)` out, every time, so `cml`/`fpga-lisp`/`my-idea` can
/// parse a response without guessing whether a given line is a value, an
/// error, or REPL chrome. Op set: `eval`, `parse`, `diagnose`,
/// `contract-version` for semantic-oracle use (same per-connection
/// isolation as the human REPL — a `def` on one connection is invisible
/// to every other), plus `notify`/`poll` for lightweight agent-to-agent
/// coordination (owner decision, 2026-08-12) — those two share one
/// mailbox across every connection instead of per-connection state,
/// deliberately kept separate from eval sessions so the oracle's
/// isolation guarantee still holds for `eval`/`parse`/`diagnose`.
fn run_tcp_repl_sexpr(port: u16, core_lib: &str, allowed: &[String], contract_version: Value) {
    let listener = match TcpListener::bind((Ipv4Addr::LOCALHOST, port)) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("Error: could not bind TCP REPL to 127.0.0.1:{port}: {err}");
            process::exit(1);
        }
    };
    let actual_port = listener.local_addr().map(|a| a.port()).unwrap_or(port);
    eprintln!("my-lisp TCP REPL v{} (sexpr protocol) listening on 127.0.0.1:{actual_port}", env!("CARGO_PKG_VERSION"));

    // Shared across every connection this process ever accepts — connections
    // are handled one at a time (no threads), so plain `Vec`s need no lock.
    let mut mailbox: Vec<MailboxEntry> = Vec::new();
    let mut next_mailbox_id: u64 = 1;

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(_) => continue,
        };
        let peer = stream.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "?".into());
        eprintln!("TCP REPL: connection from {peer}");

        let environment = if allowed.is_empty() {
            Environment::root()
        } else {
            Environment::root().with_process_allowlist(allowed.to_vec())
        };
        let mut session = Session { environment };
        if let Ok(core_ast) = parse(core_lib) {
            let _ = eval_parsed_expressions(&core_ast, &mut session);
        }

        let mut reader = BufReader::new(stream.try_clone().expect("clone TCP stream"));
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    eprintln!("TCP REPL: {peer} > {trimmed}");

                    // The request envelope itself is read as literal data
                    // (`quote`), never evaluated — `(op eval)` deciding to
                    // evaluate `source` is the only place code ever runs.
                    let quoted = format!("(quote {trimmed})");
                    let request = match parse(&quoted).ok().and_then(|ast| {
                        eval_parsed_expressions(&ast, &mut session).ok().map(|r| r.value)
                    }) {
                        Some(value) => value,
                        None => {
                            let resp = error_response(&Value::Nil, "parse-error", "request envelope is not a valid s-expression", &contract_version);
                            let _ = writeln!(stream, "{resp}");
                            continue;
                        }
                    };

                    let fields = list_items(&request);
                    // fields[0] is the `request` tag symbol itself.
                    let mut id = Value::Nil;
                    let mut op: Option<String> = None;
                    let mut source: Option<String> = None;
                    let mut from: Option<String> = None;
                    let mut to: Option<String> = None;
                    let mut message: Option<String> = None;
                    let mut for_agent: Option<String> = None;
                    let mut since: u64 = 0;
                    for field in fields.iter().skip(1) {
                        let kv = list_items(field);
                        let (Some(key), Some(val)) = (kv.first(), kv.get(1)) else { continue };
                        if let Value::Symbol(name) = key {
                            match &**name {
                                "id" => id = val.clone(),
                                "op" => {
                                    if let Value::Symbol(s) = val {
                                        op = Some(s.to_string());
                                    }
                                }
                                "source" => {
                                    if let Value::String(s) = val {
                                        source = Some(s.to_string());
                                    }
                                }
                                "from" => {
                                    if let Value::String(s) = val {
                                        from = Some(s.to_string());
                                    }
                                }
                                "to" => {
                                    if let Value::String(s) = val {
                                        to = Some(s.to_string());
                                    }
                                }
                                "message" => {
                                    if let Value::String(s) = val {
                                        message = Some(s.to_string());
                                    }
                                }
                                "for" => {
                                    if let Value::String(s) = val {
                                        for_agent = Some(s.to_string());
                                    }
                                }
                                "since" => {
                                    if let Value::Number(n, _) = val {
                                        since = *n as u64;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    let response = match op.as_deref() {
                        Some("contract-version") => ok_response(&id, contract_version.clone(), &[], &contract_version),
                        // `parse` renders the canonical structure via the same
                        // `quote`-and-print path the request envelope itself
                        // uses, not Rust's `{:?}` — the caller gets my-lisp
                        // syntax back, not this CLI's internal AST debug
                        // format. Limited to a single top-level form, the
                        // same arity `quote` itself has.
                        Some("parse") => match &source {
                            None => error_response(&id, "invalid-form", "op `parse` requires a `source` field", &contract_version),
                            Some(src) => match parse(src) {
                                Ok(ast) if ast.len() == 1 => {
                                    let quoted_src = format!("(quote {src})");
                                    match parse(&quoted_src).ok().and_then(|q| {
                                        eval_parsed_expressions(&q, &mut session).ok().map(|r| r.value)
                                    }) {
                                        Some(structure) => ok_response(&id, structure, &[], &contract_version),
                                        None => error_response(&id, "parse-error", "source parsed but could not be rendered as data", &contract_version),
                                    }
                                }
                                Ok(_) => error_response(&id, "invalid-form", "op `parse` accepts exactly one top-level form", &contract_version),
                                Err(e) => error_response(&id, error_kind_symbol(&e.kind), &e.message, &contract_version),
                            },
                        },
                        Some(op_name @ ("eval" | "diagnose")) => match &source {
                            None => error_response(&id, "invalid-form", &format!("op `{op_name}` requires a `source` field"), &contract_version),
                            Some(src) => match parse(src) {
                                Ok(ast) => match eval_parsed_expressions(&ast, &mut session) {
                                    Ok(result) => ok_response(&id, result.value, &result.output, &contract_version),
                                    Err(e) => error_response(&id, error_kind_symbol(&e.kind), &e.message, &contract_version),
                                },
                                Err(e) => error_response(&id, error_kind_symbol(&e.kind), &e.message, &contract_version),
                            },
                        },
                        Some("notify") => match (&from, &message) {
                            (None, _) => error_response(&id, "invalid-form", "op `notify` requires a `from` field", &contract_version),
                            (_, None) => error_response(&id, "invalid-form", "op `notify` requires a `message` field", &contract_version),
                            (Some(from), Some(message)) => {
                                let entry_id = next_mailbox_id;
                                next_mailbox_id += 1;
                                mailbox.push(MailboxEntry {
                                    id: entry_id,
                                    from: from.clone(),
                                    to: to.clone(),
                                    message: message.clone(),
                                });
                                // Bounded so a long-lived server (or a
                                // runaway notifier) can't grow this
                                // in-memory, non-persistent mailbox
                                // without limit — oldest entries are
                                // dropped first; a `poll` with `since`
                                // older than what's left just gets
                                // whatever's still here.
                                const MAILBOX_CAPACITY: usize = 500;
                                if mailbox.len() > MAILBOX_CAPACITY {
                                    let excess = mailbox.len() - MAILBOX_CAPACITY;
                                    mailbox.drain(0..excess);
                                }
                                ok_response(&id, Value::Number(entry_id as f64, Exactness::Exact), &[], &contract_version)
                            }
                        },
                        Some("poll") => match &for_agent {
                            None => error_response(&id, "invalid-form", "op `poll` requires a `for` field", &contract_version),
                            Some(for_agent) => {
                                let matches: Vec<Value> = mailbox
                                    .iter()
                                    .filter(|entry| entry.id > since)
                                    .filter(|entry| entry.to.as_deref().is_none_or(|to| to == for_agent))
                                    .map(mailbox_entry_to_value)
                                    .collect();
                                ok_response(&id, Value::list(matches), &[], &contract_version)
                            }
                        },
                        Some(other) => error_response(&id, "invalid-form", &format!("unknown op `{other}`"), &contract_version),
                        None => error_response(&id, "invalid-form", "request is missing an `op` field", &contract_version),
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
    let sexpr_protocol = args.iter().any(|a| a == "--protocol=sexpr");
    let args: Vec<String> = args
        .into_iter()
        .filter(|arg| !arg.starts_with("--allow-process=") && arg != "--protocol=sexpr")
        .collect();
    let allowed_for_tcp = allowed.clone();
    let contract_version = {
        let contract_source = include_str!("../../../language-contract.my");
        let mut throwaway = Session { environment: Environment::root() };
        let quoted = format!("(quote {contract_source})");
        parse(&quoted)
            .ok()
            .and_then(|ast| eval_parsed_expressions(&ast, &mut throwaway).ok())
            .map(|r| r.value)
            .and_then(|v| {
                let major = dotted_alist_lookup(&v, "major")?;
                let minor = dotted_alist_lookup(&v, "minor")?;
                Some(Value::list([major, minor]))
            })
            .unwrap_or(Value::Nil)
    };
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
            println!("  --protocol=sexpr              With --tcp: strict (request (id) (op) (source)) / (response ...) envelope, no banner/prompt");
            return;
        }

        if arg == "--tcp" || arg.starts_with("--tcp=") {
            let port = arg
                .strip_prefix("--tcp=")
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(9999);
            if sexpr_protocol {
                run_tcp_repl_sexpr(port, core_lib, &allowed_for_tcp, contract_version);
            } else {
                run_tcp_repl(port, core_lib, &allowed_for_tcp);
            }
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

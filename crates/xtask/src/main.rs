//! xtask — project verification/policy tool.
//! xtask — інструмент верифікації/політик проєкту.
//!
//! Hosts documentation/governance/policy checks that were previously
//! implemented as `cargo test` tests but do not exercise executable
//! my-lisp behavior (markdown grepping, external script shell-outs,
//! contract-metadata text matching). `cargo test` should verify
//! executable behavior only; this tool verifies everything else.
//!
//! Розміщує перевірки документації/врядування/політик, які раніше були
//! реалізовані як тести `cargo test`, але не перевіряють виконувану
//! поведінку my-lisp (grep по markdown, виклики зовнішніх скриптів,
//! звірка тексту метаданих контрактів). `cargo test` має перевіряти лише
//! виконувану поведінку; цей інструмент перевіряє все інше.

mod checks;
#[cfg(test)]
mod external_oracle;
mod gen_functions_md;

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("verify") => run_verify(),
        Some("gen-functions-md") => match gen_functions_md::run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("gen-functions-md failed: {error}");
                ExitCode::FAILURE
            }
        },
        Some(other) => {
            eprintln!("unknown xtask subcommand: {other}");
            print_usage();
            ExitCode::FAILURE
        }
        None => {
            print_usage();
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!("usage: cargo xtask <verify|gen-functions-md>");
}

fn run_verify() -> ExitCode {
    let all_checks = checks::all();
    let mut failures = Vec::new();

    for check in &all_checks {
        print!("[xtask verify] {} ... ", check.name);
        match (check.run)() {
            Ok(()) => println!("ok"),
            Err(message) => {
                println!("FAILED");
                failures.push((check.name, message));
            }
        }
    }

    if failures.is_empty() {
        println!(
            "[xtask verify] all {} checks passed",
            all_checks.len()
        );
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "\n[xtask verify] {} of {} checks failed:\n",
            failures.len(),
            all_checks.len()
        );
        for (name, message) in &failures {
            eprintln!("  - {name}:\n{message}\n");
        }
        ExitCode::FAILURE
    }
}

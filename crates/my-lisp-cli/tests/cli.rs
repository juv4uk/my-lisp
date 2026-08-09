//! Integration tests for the `my-lisp` CLI binary.
//! Інтеграційні тести для CLI-бінарника `my-lisp`.
//! Integrationstests für die `my-lisp`-CLI-Binärdatei.
//!
//! These exercise the compiled binary as a black box (argv in, stdout/stderr/exit
//! code out) instead of calling internal functions directly, since main.rs itself
//! has no unit-testable functions — the behavior lives in argument handling and I/O.
//! Вони перевіряють скомпільований бінарник як чорну скриньку (argv на вході,
//! stdout/stderr/код виходу на виході), а не викликають внутрішні функції напряму,
//! бо main.rs не має власних функцій для unit-тестів — поведінка живе в обробці
//! аргументів та I/O.
//! Sie prüfen die kompilierte Binärdatei als Black Box (argv als Eingabe,
//! stdout/stderr/Exit-Code als Ausgabe) statt interne Funktionen direkt aufzurufen,
//! da main.rs selbst keine unit-testbaren Funktionen besitzt — das Verhalten steckt
//! in der Argumentverarbeitung und E/A.

use std::process::Command;

fn my_lisp() -> Command {
    Command::new(env!("CARGO_BIN_EXE_my-lisp"))
}

#[test]
fn version_flag_prints_the_crate_version() {
    let output = my_lisp().arg("--version").output().expect("binary should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), format!("my-lisp {}", env!("CARGO_PKG_VERSION")));
}

#[test]
fn short_version_flags_match_the_long_form() {
    for flag in ["-V", "-v"] {
        let output = my_lisp().arg(flag).output().expect("binary should run");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), format!("my-lisp {}", env!("CARGO_PKG_VERSION")));
    }
}

#[test]
fn help_flag_prints_usage() {
    let output = my_lisp().arg("--help").output().expect("binary should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: my-lisp [file]"));
}

#[test]
fn running_a_source_file_prints_its_result() {
    let dir = std::env::temp_dir();
    let path = dir.join("my-lisp-cli-test-ok.my");
    std::fs::write(&path, "(+ 1 2)").expect("should write temp file");

    let output = my_lisp().arg(&path).output().expect("binary should run");
    let _ = std::fs::remove_file(&path);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "3");
}

#[test]
fn running_a_file_with_an_evaluation_error_exits_nonzero() {
    let dir = std::env::temp_dir();
    let path = dir.join("my-lisp-cli-test-eval-error.my");
    std::fs::write(&path, "(car '())").expect("should write temp file");

    let output = my_lisp().arg(&path).output().expect("binary should run");
    let _ = std::fs::remove_file(&path);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("Error:"));
}

#[test]
fn running_a_file_with_a_parse_error_exits_nonzero() {
    let dir = std::env::temp_dir();
    let path = dir.join("my-lisp-cli-test-parse-error.my");
    std::fs::write(&path, "(1 2").expect("should write temp file");

    let output = my_lisp().arg(&path).output().expect("binary should run");
    let _ = std::fs::remove_file(&path);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("Parse error:"));
}

#[test]
fn running_a_missing_file_reports_a_read_error() {
    let output = my_lisp()
        .arg("this-file-does-not-exist-my-lisp-cli.my")
        .output()
        .expect("binary should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("Error reading file"));
}

#[test]
fn repl_history_persists_across_separate_sessions() {
    // Isolate HOME/USERPROFILE per test run so this doesn't read or write the
    // real user's ~/.my-lisp-history, and so parallel test runs don't collide.
    // Ізолюємо HOME/USERPROFILE для кожного запуску тесту, щоб не читати й не
    // писати в реальний ~/.my-lisp-history користувача, і щоб паралельні
    // запуски тестів не конфліктували.
    // Isoliert HOME/USERPROFILE pro Testlauf, damit weder das echte
    // ~/.my-lisp-history des Nutzers gelesen/geschrieben wird noch parallele
    // Testläufe kollidieren.
    let dir = std::env::temp_dir().join(format!(
        "my-lisp-cli-test-history-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("should create temp home dir");

    let run = |input: &str| {
        use std::io::Write;
        use std::process::Stdio;
        let mut child = my_lisp()
            .env("HOME", &dir)
            .env("USERPROFILE", &dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("binary should spawn");
        child
            .stdin
            .take()
            .expect("stdin should be piped")
            .write_all(input.as_bytes())
            .expect("should write to stdin");
        child.wait_with_output().expect("binary should run")
    };

    run("(+ 1 2)\n");
    run("(+ 3 4)\n");

    let history = std::fs::read_to_string(dir.join(".my-lisp-history"))
        .expect("second session should find history left by the first");
    let _ = std::fs::remove_dir_all(&dir);

    assert!(history.contains("(+ 1 2)"));
    assert!(history.contains("(+ 3 4)"));
}

#[test]
fn read_with_no_arguments_reads_one_line_from_real_stdin() {
    // Reliable in file mode, where the CLI's own stdin isn't also owned by
    // rustyline's line editor (see the caveat on read_stdin_line in
    // crates/my-lisp/src/eval/special_forms.rs for the interactive-REPL case).
    use std::io::Write;
    use std::process::Stdio;

    let dir = std::env::temp_dir();
    let path = dir.join("my-lisp-cli-test-read-stdin.my");
    std::fs::write(&path, "(eval (read))").expect("should write temp file");

    let mut child = my_lisp()
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("binary should spawn");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(b"(+ 1 2)\n")
        .expect("should write to stdin");
    let output = child.wait_with_output().expect("binary should run");
    let _ = std::fs::remove_file(&path);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "3");
}

#[test]
fn core_lib_is_preloaded_before_running_a_file() {
    // lib/core.my defines `identity`; if the CLI stopped injecting core.my this
    // would fail with an "unknown symbol" evaluation error instead of returning 5.
    // lib/core.my визначає `identity`; якби CLI перестав вставляти core.my, це б
    // провалилось помилкою "unknown symbol" замість повернення 5.
    // lib/core.my definiert `identity`; würde die CLI core.my nicht mehr einspeisen,
    // schlüge dies mit einem "unknown symbol"-Fehler fehl statt 5 zurückzugeben.
    let dir = std::env::temp_dir();
    let path = dir.join("my-lisp-cli-test-core-lib.my");
    std::fs::write(&path, "(identity 5)").expect("should write temp file");

    let output = my_lisp().arg(&path).output().expect("binary should run");
    let _ = std::fs::remove_file(&path);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "5");
}

#[test]
fn argv_carries_everything_after_the_filename() {
    // *argv* (PLAN.md item 21's follow-up, for scripts/release.my taking a
    // version on the command line) is whatever follows the filename, as a
    // my-lisp list of strings — not parsed as code, just passed through.
    // *argv* (продовження PLAN.md, пункту 21, для scripts/release.my, яка
    // бере версію з командного рядка) — усе, що йде після імені файлу, як
    // my-lisp-список рядків — не парситься як код, лише передається як є.
    let dir = std::env::temp_dir();
    let path = dir.join("my-lisp-cli-test-argv.my");
    std::fs::write(&path, "*argv*").expect("should write temp file");

    let output = my_lisp()
        .arg(&path)
        .arg("0.4.4")
        .arg("extra")
        .output()
        .expect("binary should run");
    let _ = std::fs::remove_file(&path);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "(\"0.4.4\" \"extra\")");
}

#[test]
fn argv_is_empty_when_nothing_follows_the_filename() {
    let dir = std::env::temp_dir();
    let path = dir.join("my-lisp-cli-test-argv-empty.my");
    std::fs::write(&path, "*argv*").expect("should write temp file");

    let output = my_lisp().arg(&path).output().expect("binary should run");
    let _ = std::fs::remove_file(&path);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "()");
}

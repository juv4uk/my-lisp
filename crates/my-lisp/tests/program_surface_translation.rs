use my_lisp::{eval_program, Session};
use std::io::Write;
use std::process::{Command, Stdio};

fn translate(source: &str, from: &str, to: &str) -> String {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts/translate-program.py");
    let mut child = Command::new("python3")
        .args([script, "--from", from, "--to", to, "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Python surface translator should start");
    child
        .stdin
        .take()
        .expect("translator stdin")
        .write_all(source.as_bytes())
        .expect("program should reach translator");
    let output = child.wait_with_output().expect("translator should finish");
    assert!(output.status.success(), "translator failed");
    String::from_utf8(output.stdout).expect("translated source should remain UTF-8")
}

#[test]
fn translated_programs_execute_with_the_same_result_on_all_three_surfaces() {
    let english = "(car (cons 'cat 'dog))";
    let ukrainian = translate(english, "en", "uk");
    let sanskrit = translate(english, "en", "sa");

    let mut english_session = Session::default();
    let mut ukrainian_session = Session::default();
    let mut sanskrit_session = Session::default();

    let english_result = eval_program(english, &mut english_session).expect("English program");
    let ukrainian_result =
        eval_program(&ukrainian, &mut ukrainian_session).expect("Ukrainian program");
    let sanskrit_result =
        eval_program(&sanskrit, &mut sanskrit_session).expect("Sanskrit program");

    assert_eq!(english_result.value.to_string(), "cat");
    assert_eq!(ukrainian_result.value.to_string(), "cat");
    assert_eq!(sanskrit_result.value.to_string(), "cat");
}

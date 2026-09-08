use my_lisp::{eval_program, Session};

const УКРАЇНСЬКА_ПРОГРАМА: &str =
    include_str!("../../../tests/fixtures/rivnopravnist-uk.my");

fn виконуваний_код(джерело: &str) -> String {
    джерело
        .lines()
        .map(|рядок| рядок.split(';').next().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn україномовна_програма_приймання_не_потребує_латинської_розкладки() {
    let код = виконуваний_код(УКРАЇНСЬКА_ПРОГРАМА);

    let латинські_літери = код
        .chars()
        .filter(|символ| символ.is_ascii_alphabetic())
        .collect::<String>();

    assert!(
        латинські_літери.is_empty(),
        "у виконуваному українському коді знайдено латинські літери: {латинські_літери:?}"
    );
}

#[test]
fn україномовна_програма_приймання_виконується_самостійно() {
    let mut сесія = Session::default();
    let результат = eval_program(УКРАЇНСЬКА_ПРОГРАМА, &mut сесія)
        .expect("україномовна acceptance-програма повинна виконуватися")
        .value
        .to_string();

    assert_eq!(
        результат, "гаразд",
        "українська поверхня повинна самостійно пройти acceptance-програму"
    );
}

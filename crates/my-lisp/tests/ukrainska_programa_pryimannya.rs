use my_lisp::{eval_program, Session};

const УКРАЇНСЬКА_ПРОГРАМА: &str =
    include_str!("../../../tests/fixtures/rivnopravnist-uk.my");

#[test]
fn україномовна_програма_приймання_виконується_без_англійських_форм() {
    for заборонена_англійська_форма in ["define", "lambda", "quote", "cond"] {
        assert!(
            !УКРАЇНСЬКА_ПРОГРАМА.contains(заборонена_англійська_форма),
            "acceptance-програма не повинна покладатися на EN форму {заборонена_англійська_форма:?}"
        );
    }

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

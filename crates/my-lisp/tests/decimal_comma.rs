use my_lisp::{eval_program, parse, ErrorKind, ExprKind, Session};

fn obchyslyty(source: &str) -> String {
    let mut session = Session::default();
    eval_program(source, &mut session)
        .expect("обчислення має бути успішним")
        .value
        .to_string()
}

/// Поточна канонічна форма результату `eq` — запитана в самого Lisp
/// (#114/#220), а не вписана тут: що #218's контракт зараз каже, що
/// повертає `eq`, те й порівнюємо.
fn eq_forma(same: bool) -> String {
    obchyslyty(if same { "(eq 1 1)" } else { "(eq 1 2)" })
}

fn ochikuvaty_symvol(source: &str) {
    let forms = parse(source).expect("читання символу має бути успішним");
    assert_eq!(forms.len(), 1);
    match &forms[0].kind {
        ExprKind::Symbol(symbol) => assert_eq!(&**symbol, source),
        other => panic!("очікував символ {source:?}, отримав {other:?}"),
    }
}

#[test]
fn desiatkova_koma_i_krapka_maiut_odnu_tochnu_semantyku() {
    assert_eq!(obchyslyty("(eq 12,455 12.455)"), eq_forma(true));
    assert_eq!(obchyslyty("(+ 1,5 2,5)"), "4");
    assert_eq!(obchyslyty("(eq -0,25 -0.25)"), eq_forma(true));
    assert_eq!(obchyslyty("(eq 1,5e3 1500)"), eq_forma(true));
    assert_eq!(obchyslyty("(eq (read \"12,455\") 12.455)"), eq_forma(true));
}

#[test]
fn koma_ne_staied_punktuatsiieiu_zvychaynykh_symvoliv() {
    ochikuvaty_symvol("а,б");
    ochikuvaty_symvol("версія1,2");
    ochikuvaty_symvol("1,2,3");
    ochikuvaty_symvol("1,2.3");
}

#[test]
fn desiatkova_koma_zberihaie_nazvanu_vidmovu_na_resursnii_mezhi() {
    // Межа захищає фактичний степінь 10^N, який reader матеріалізує після
    // врахування цифр після десяткового роздільника. Тому 1,25e10002 має
    // effective exponent 10000 і ще допустиме, а 1,25e10003 -> 10001 вже ні.
    assert!(parse("1,25e10002").is_ok());
    assert!(parse("1.25e10002").is_ok());

    for source in ["1,25e10003", "1.25e10003"] {
        let error = parse(source).expect_err("фактичний степінь 10 має перевищити межу");
        assert_eq!(error.kind, ErrorKind::NumericOverflow);
    }
}

#[test]
fn f32_buffer_pryimaie_desiatkovu_komu() {
    let forms = parse("#f32(1,5 2,25)").expect("#f32 має приймати десяткову кому");
    assert_eq!(forms.len(), 1);
    assert!(matches!(forms[0].kind, ExprKind::NumericBuffer(_)));

    assert_eq!(obchyslyty("(eq #f32(-0,0) #f32(0,0))"), eq_forma(false));
    assert_eq!(obchyslyty("(eq #f32(-0.0) #f32(0.0))"), eq_forma(false));
}

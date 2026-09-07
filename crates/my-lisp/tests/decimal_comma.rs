use my_lisp::{eval_program, parse, ExprKind, Session};

fn obchyslyty(source: &str) -> String {
    let mut session = Session::default();
    eval_program(source, &mut session)
        .expect("обчислення має бути успішним")
        .value
        .to_string()
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
    assert_eq!(obchyslyty("(eq 12,455 12.455)"), "t");
    assert_eq!(obchyslyty("(+ 1,5 2,5)"), "4");
    assert_eq!(obchyslyty("(eq -0,25 -0.25)"), "t");
    assert_eq!(obchyslyty("(eq 1,5e3 1500)"), "t");
}

#[test]
fn koma_ne_staied_punktuatsiieiu_zvychaynykh_symvoliv() {
    ochikuvaty_symvol("а,б");
    ochikuvaty_symvol("версія1,2");
    ochikuvaty_symvol("1,2,3");
    ochikuvaty_symvol("1,2.3");
}

#[test]
fn f32_buffer_pryimaie_desiatkovu_komu() {
    let forms = parse("#f32(1,5 2,25)").expect("#f32 має приймати десяткову кому");
    assert_eq!(forms.len(), 1);
    assert!(matches!(forms[0].kind, ExprKind::NumericBuffer(_)));
}

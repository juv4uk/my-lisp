from pathlib import Path

parser_path = Path("crates/my-lisp/src/parser.rs")
test_path = Path("crates/my-lisp/tests/decimal_comma.rs")

parser = parser_path.read_text(encoding="utf-8")
old = '''                        for element in elements {
                            let number = match element.kind {
                                ExprKind::Number(value, _) => value,
                                ExprKind::Rational(value) => value.as_f64(),
                                _ => {
                                    return Err(self.error(
                                        "#f32 expects numeric elements",
                                        element.span.start,
                                        element.span.end,
                                    ))
                                }
                            };
'''
new = '''                        for element in elements {
                            let span = element.span;
                            let number = match element.kind {
                                ExprKind::Number(_, _) => {
                                    let spelling = &self.source[span.start..span.end];
                                    let spelling_with_dot = if spelling.contains(',')
                                        && !spelling.contains('.')
                                    {
                                        Some(spelling.replace(',', "."))
                                    } else {
                                        None
                                    };
                                    spelling_with_dot
                                        .as_deref()
                                        .unwrap_or(spelling)
                                        .parse::<f32>()
                                        .map(f64::from)
                                        .map_err(|_| {
                                            self.error(
                                                "#f32 expects numeric elements",
                                                span.start,
                                                span.end,
                                            )
                                        })?
                                }
                                ExprKind::Rational(value) => value.as_f64(),
                                _ => {
                                    return Err(self.error(
                                        "#f32 expects numeric elements",
                                        element.span.start,
                                        element.span.end,
                                    ))
                                }
                            };
'''
if parser.count(old) != 1:
    raise SystemExit(f"parser marker count: {parser.count(old)}")
parser_path.write_text(parser.replace(old, new, 1), encoding="utf-8")

test = test_path.read_text(encoding="utf-8")
old_test = '''#[test]
fn f32_buffer_pryimaie_desiatkovu_komu() {
    let forms = parse("#f32(1,5 2,25)").expect("#f32 має приймати десяткову кому");
    assert_eq!(forms.len(), 1);
    assert!(matches!(forms[0].kind, ExprKind::NumericBuffer(_)));
}
'''
new_test = '''#[test]
fn f32_buffer_pryimaie_desiatkovu_komu() {
    let forms = parse("#f32(1,5 2,25)").expect("#f32 має приймати десяткову кому");
    assert_eq!(forms.len(), 1);
    assert!(matches!(forms[0].kind, ExprKind::NumericBuffer(_)));

    assert_eq!(obchyslyty("(eq #f32(-0,0) #f32(0,0))"), "()");
    assert_eq!(obchyslyty("(eq #f32(-0.0) #f32(0.0))"), "()");
}
'''
if test.count(old_test) != 1:
    raise SystemExit(f"test marker count: {test.count(old_test)}")
test_path.write_text(test.replace(old_test, new_test, 1), encoding="utf-8")

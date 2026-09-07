#!/usr/bin/env python3
from pathlib import Path

# Цей файл є одноразовим fail-closed applicator-ом і сам видаляється після успіху.

def zaminyty_odyn_raz(shliakh: str, stare: str, nove: str) -> None:
    path = Path(shliakh)
    text = path.read_text(encoding="utf-8")
    count = text.count(stare)
    if count != 1:
        raise SystemExit(f"{shliakh}: очікував рівно 1 збіг, отримав {count}")
    path.write_text(text.replace(stare, nove, 1), encoding="utf-8")


parser = "crates/my-lisp/src/parser.rs"

zaminyty_odyn_raz(
    parser,
    """                                ExprKind::Number(_, _) => self.source[span.start..span.end]\n                                    .parse::<f32>()\n                                    .map(f64::from)\n                                    .map_err(|_| {\n                                        self.error(\n                                            \"#f32 expects numeric elements\",\n                                            span.start,\n                                            span.end,\n                                        )\n                                    })?,\n""",
    """                                ExprKind::Number(value, _) => value,\n""",
)

zaminyty_odyn_raz(
    parser,
    """        let token = &self.source[start..self.cursor];\n""",
    """        let token = &self.source[start..self.cursor];\n        let decimal_with_dot = if token.contains(',') && !token.contains('.') {\n            Some(token.replace(',', \".\"))\n        } else {\n            None\n        };\n        let decimal_text = decimal_with_dot.as_deref().unwrap_or(token);\n""",
)

zaminyty_odyn_raz(
    parser,
    """        } else if token.contains(['.', 'e', 'E']) {\n            let kind = match crate::value::Rational::from_decimal_literal(token) {\n""",
    """        } else if token.contains(['.', ',', 'e', 'E']) {\n            let kind = match crate::value::Rational::from_decimal_literal(decimal_text) {\n""",
)

contract = "language-contract.my"
zaminyty_odyn_raz(contract, "((major . 4) (minor . 0)", "((major . 5) (minor . 0)")

old_note_start = ''' (note . "RATIFIED by owner 2026-09-07. Reader apostrophe semantics are observable S2 semantics: an apostrophe at expression start is canonical QUOTE reader sugar, while an apostrophe inside an identifier remains an ordinary symbol character (for example, об'єкт and п'ять stay single identifiers). This corrects the earlier over-broad rule that removed quote sugar entirely; because existing source text beginning with apostrophe changes observable parsing/evaluation, the contract bumps 3.0 -> 4.0. Contract 3.0 error classifications remain unchanged. · Апостроф на початку виразу є синтаксичним цукром канонічного QUOTE; апостроф усередині ідентифікатора лишається звичайним знаком символу. Це спостережувана семантика reader-а, тому контракт піднято з 3.0 до 4.0.")'''
new_note_start = ''' (note . "RATIFIED by owner 2026-09-07. Contract 5.0 accepts both dot and comma as spellings of the decimal separator for an otherwise valid finite decimal or base-10 scientific numeric token: 12.455 and 12,455 denote the same exact value. Comma gains numeric meaning only when the whole token parses as a number; ordinary symbols containing commas remain symbols. This is observable reader semantics: 12,455 was previously a symbol, so the contract bumps 4.0 -> 5.0. Contract 4.0 apostrophe semantics and Contract 3.0 error classifications remain unchanged. · Контракт 5.0 дозволяє і крапку, і кому як написання десяткового роздільника у коректному числовому токені: 12.455 і 12,455 означають те саме точне значення. Кома має числовий сенс лише тоді, коли весь токен є числом; у звичайному символі вона лишається частиною символу. Оскільки раніше 12,455 читалося як символ, це спостережувана зміна reader-а і major-перехід 4.0 -> 5.0.")'''
zaminyty_odyn_raz(contract, old_note_start, new_note_start)

zaminyty_odyn_raz(
    contract,
    '''      (error-classification\n''',
    '''      (reader-decimal-separator\n        . "Contract 5.0: dot and comma are equivalent decimal-separator spellings only for an otherwise valid finite decimal/base-10 scientific numeric token. 12.455 and 12,455 denote the same exact rational value; -0,25 and 1,5e3 are valid numeric spellings. A comma in a non-numeric token remains an ordinary symbol character, so а,б and версія1,2 remain symbols; mixed or repeated separators that do not form a valid number also remain symbols under the existing malformed-literal rule.")\n      (error-classification\n''',
)

readme = "README.md"
zaminyty_odyn_raz(
    readme,
    "Поточний машинний семантичний контракт — [`language-contract.my`](language-contract.my), версія **4.0**.",
    "Поточний машинний семантичний контракт — [`language-contract.my`](language-contract.my), версія **5.0**.",
)

zaminyty_odyn_raz(
    readme,
    """Апостроф на початку виразу — reader syntax для `QUOTE`; апостроф усередині слова — звичайна частина ідентифікатора.\n\n---\n\n## Українською можна програмувати\n""",
    """Апостроф на початку виразу — reader syntax для `QUOTE`; апостроф усередині слова — звичайна частина ідентифікатора.\n\n### Десяткова кома\n\nНа українській розкладці десятковий роздільник можна набирати комою. Крапка й кома є двома написаннями **того самого точного числового значення**:\n\n```lisp\n(eq 12,455 12.455)   ; t\n(+ 1,5 2,5)          ; 4\n(eq -0,25 -0.25)     ; t\n(eq 1,5e3 1500)      ; t\n```\n\nКома отримує числовий сенс лише тоді, коли весь токен є коректним числом. Тому `а,б` і `версія1,2` лишаються звичайними символами.\n\n---\n\n## Українською можна програмувати\n""",
)

axioms = "docs/language-core-axioms.md"
zaminyty_odyn_raz(
    axioms,
    """**Numeric literal syntax does not imply inexactness. A finite decimal or base-10 scientific literal denotes its exact mathematical rational value.** `(/ 1 3)` means exactly `1/3`, not `0.333...`. A concrete machine may have real resource limits and refuse the operation (`NumericOverflow` or similar) — but it must never quietly turn `1/3` into `0.333343` and pretend that's the same value.\n""",
    """**Контракт 5.0:** крапка й кома є рівноправними написаннями десяткового роздільника, коли весь токен є коректним скінченним десятковим або base-10 scientific числом. `12.455` і `12,455` означають одне й те саме точне раціональне значення; кома в нечисловому символі лишається частиною символу.\n\n**Numeric literal syntax does not imply inexactness. A finite decimal or base-10 scientific literal denotes its exact mathematical rational value. Contract 5.0 accepts either `.` or `,` as its decimal-separator spelling when the whole token is otherwise numeric.** `(/ 1 3)` means exactly `1/3`, not `0.333...`. A concrete machine may have real resource limits and refuse the operation (`NumericOverflow` or similar) — but it must never quietly turn `1/3` into `0.333343` and pretend that's the same value.\n""",
)

test_path = Path("crates/my-lisp/tests/decimal_comma.rs")
if test_path.exists():
    raise SystemExit(f"{test_path}: файл уже існує")
test_path.write_text(
    '''use my_lisp::{eval_program, parse, ExprKind, Session};\n\nfn obchyslyty(source: &str) -> String {\n    let mut session = Session::default();\n    eval_program(source, &mut session)\n        .expect("обчислення має бути успішним")\n        .to_string()\n}\n\nfn ochikuvaty_symvol(source: &str) {\n    let forms = parse(source).expect("читання символу має бути успішним");\n    assert_eq!(forms.len(), 1);\n    match &forms[0].kind {\n        ExprKind::Symbol(symbol) => assert_eq!(&**symbol, source),\n        other => panic!("очікував символ {source:?}, отримав {other:?}"),\n    }\n}\n\n#[test]\nfn desiatkova_koma_i_krapka_maiut_odnu_tochnu_semantyku() {\n    assert_eq!(obchyslyty("(eq 12,455 12.455)"), "t");\n    assert_eq!(obchyslyty("(+ 1,5 2,5)"), "4");\n    assert_eq!(obchyslyty("(eq -0,25 -0.25)"), "t");\n    assert_eq!(obchyslyty("(eq 1,5e3 1500)"), "t");\n}\n\n#[test]\nfn koma_ne_staied_punktuatsiieiu_zvychaynykh_symvoliv() {\n    ochikuvaty_symvol("а,б");\n    ochikuvaty_symvol("версія1,2");\n    ochikuvaty_symvol("1,2,3");\n    ochikuvaty_symvol("1,2.3");\n}\n\n#[test]\nfn f32_buffer_pryimaie_desiatkovu_komu() {\n    let forms = parse("#f32(1,5 2,25)").expect("#f32 має приймати десяткову кому");\n    assert_eq!(forms.len(), 1);\n    assert!(matches!(forms[0].kind, ExprKind::NumericBuffer(_)));\n}\n''',
    encoding="utf-8",
)

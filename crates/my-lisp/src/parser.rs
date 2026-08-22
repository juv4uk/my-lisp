use crate::{Exactness, ErrorKind, Expr, ExprKind, LanguageError, Span};
use std::rc::Rc;

/// `true` for a token that is exactly the single character `.` — the reader
/// marker for a dotted pair's tail, never a symbol name in ordinary use.
fn is_dot_symbol(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Symbol(symbol) if &**symbol == ".")
}

/// Folds `items` right-to-left onto `tail`, building nested `ExprKind::Pair`
/// nodes — `(a b . c)` becomes `Pair(a, Pair(b, c))`, the same shape `cons`
/// builds at runtime. Every node shares the whole list's span; only the
/// individual `items`/`tail` sub-expressions keep their own precise spans.
fn dotted_list(items: Vec<Expr>, tail: Expr, start: usize, end: usize) -> Expr {
    let span = Span { start, end };
    items.into_iter().rev().fold(tail, |acc, item| Expr {
        kind: ExprKind::Pair(Rc::new(item), Rc::new(acc)),
        span,
    })
}

pub fn parse(source: &str) -> Result<Vec<Expr>, LanguageError> {
    let mut parser = Parser { source, cursor: 0 };
    let mut expressions = Vec::new();
    parser.skip_ignored();
    while parser.cursor < source.len() {
        expressions.push(parser.expression()?);
        parser.skip_ignored();
    }
    Ok(expressions)
}

struct Parser<'a> {
    source: &'a str,
    cursor: usize,
}

impl Parser<'_> {
    fn expression(&mut self) -> Result<Expr, LanguageError> {
        self.skip_ignored();
        let start = self.cursor;
        match self.peek() {
            Some('(') => self.list(start),
            Some(')') => Err(self.error(
                "unexpected closing parenthesis · neochikuvana zakryvna duzhka · unerwartete schließende Klammer",
                start,
                start + 1,
            )),
            Some('"') => self.string(start),
            Some(_) => self.atom(start),
            None => Err(self.error(
                "expected an expression · ochikuvavsia vyraz · Ausdruck erwartet",
                start,
                start,
            )),
        }
    }



    fn list(&mut self, start: usize) -> Result<Expr, LanguageError> {
        self.bump();
        let mut items = Vec::new();
        loop {
            self.skip_ignored();
            match self.peek() {
                Some(')') => {
                    self.bump();
                    return Ok(Expr {
                        kind: ExprKind::List(items.into()),
                        span: Span {
                            start,
                            end: self.cursor,
                        },
                    });
                }
                Some(_) => {
                    let item = self.expression()?;
                    if is_dot_symbol(&item) {
                        if items.is_empty() {
                            return Err(self.error(
                                "unexpected '.' with nothing before it · neochikuvana '.' bez nichoho pered neiu · unerwartetes '.' ohne vorangehenden Ausdruck",
                                item.span.start,
                                item.span.end,
                            ));
                        }
                        self.skip_ignored();
                        if matches!(self.peek(), None | Some(')')) {
                            return Err(self.error(
                                "expected an expression after '.' · ochikuvavsia vyraz pislia '.' · Ausdruck nach '.' erwartet",
                                self.cursor,
                                self.cursor,
                            ));
                        }
                        let tail = self.expression()?;
                        self.skip_ignored();
                        return match self.peek() {
                            Some(')') => {
                                self.bump();
                                Ok(dotted_list(items, tail, start, self.cursor))
                            }
                            _ => Err(self.error(
                                "expected ')' after a dotted pair's tail · ochikuvalas ')' pislia khvosta dotted-pary · ')' nach dem Ende eines Dotted Pair erwartet",
                                self.cursor,
                                self.cursor,
                            )),
                        };
                    }
                    items.push(item);
                }
                None => {
                    return Err(self.error(
                        "unclosed list · nezakrytyi spysok · nicht geschlossene Liste",
                        start,
                        self.cursor,
                    ))
                }
            }
        }
    }

    fn string(&mut self, start: usize) -> Result<Expr, LanguageError> {
        self.bump();
        let mut value = String::new();
        while let Some(character) = self.bump() {
            match character {
                '"' => {
                    return Ok(Expr {
                        kind: ExprKind::String(value.into()),
                        span: Span {
                            start,
                            end: self.cursor,
                        },
                    })
                }
                '\\' => match self.bump() {
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some('r') => value.push('\r'),
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some(other) => value.push(other),
                    None => {
                        return Err(self.error(
                            "unfinished string escape · nezavershena escape-poslidovnist · unvollständige Escape-Sequenz",
                            start,
                            self.cursor,
                        ))
                    }
                },
                other => value.push(other),
            }
        }
        Err(self.error(
            "unclosed string · nezakrytyi riadok · nicht geschlossene Zeichenkette",
            start,
            self.cursor,
        ))
    }

    fn atom(&mut self, start: usize) -> Result<Expr, LanguageError> {
        while let Some(character) = self.peek() {
            if character.is_whitespace() || matches!(character, '(' | ')' | ';') {
                break;
            }
            self.bump();
        }
        let token = &self.source[start..self.cursor];
        // `Rational::from_literal` parses arbitrary-precision numerator/denominator
        // text directly (see bignum.rs) — a token like `123456789012345678901/2`,
        // far too big for `i64`, is still an exact rational literal, not a symbol.
        // `Rational::from_literal` parsyt tekst chyselnyka/znamennyka dovilnoi
        // tochnosti napriamu (dyv. bignum.rs) — token na kshtalt
        // `123456789012345678901/2`, zavelykyi dlia `i64`, use odno tochnyi
        // ratsionalnyi literal, ne symvol.
        // `Rational::from_literal` parst Zähler-/Nenner-Text beliebiger Genauigkeit
        // direkt (siehe bignum.rs) — ein Token wie `123456789012345678901/2`, viel
        // zu groß für `i64`, ist weiterhin ein exaktes rationales Literal, kein Symbol.
        // Integer literal → exact; decimal or exponential-notation literal →
        let kind = if let Some((num, den)) = token.split_once('/') {
            if let Some(r) = crate::value::Rational::from_literal(num, den) {
                ExprKind::Rational(r)
            } else {
                ExprKind::Symbol(token.into())
            }
        } else if token.contains(['.', 'e', 'E']) {
            let kind = match crate::value::Rational::from_decimal_literal(token) {
                Ok(r) => match r.as_precise_i64() {
                    Some(value) => ExprKind::Number(value as f64, Exactness::Exact),
                    None => ExprKind::Rational(r),
                },
                Err(crate::value::DecimalLiteralError::InvalidSyntax) => ExprKind::Symbol(token.into()),
                Err(crate::value::DecimalLiteralError::ResourceLimitExceeded) => {
                    // S3: a syntactically valid numeric literal must never become
                    // an ordinary symbol just because a parser resource limit
                    // refused to build it - that would change the token's
                    // meaning silently. It fails named, `NumericOverflow`, the
                    // same category runtime arithmetic uses for exact results
                    // past `with_numeric_bit_limit`.
                    return Err(LanguageError::new(
                        ErrorKind::NumericOverflow,
                        "decimal literal exponent exceeds the parser resource limit / eksponenta desiatkovoho literala perevyshchuie resursnu mezhu parsera / der Exponent des Dezimalliterals ueberschreitet die Ressourcengrenze des Parsers",
                        Span {
                            start,
                            end: self.cursor,
                        },
                    ));
                }
            };
            return Ok(Expr {
                kind,
                span: Span {
                    start,
                    end: self.cursor,
                },
            });
        } else if let Some(r) = crate::value::Rational::from_literal(token, "1") {
            // Preserve the compact f64-backed representation only where it is
            // mathematically exact; larger integer literals enter the same
            // arbitrary-precision Rational path as n/1 arithmetic results.
            match r.as_precise_i64() {
                Some(value) => ExprKind::Number(value as f64, Exactness::Exact),
                None => ExprKind::Rational(r),
            }
        } else {
            ExprKind::Symbol(token.into())
        };
        Ok(Expr {
            kind,
            span: Span {
                start,
                end: self.cursor,
            },
        })
    }

    fn skip_ignored(&mut self) {
        loop {
            while self.peek().is_some_and(char::is_whitespace) {
                self.bump();
            }
            if self.peek() != Some(';') {
                break;
            }
            while self.peek().is_some_and(|character| character != '\n') {
                self.bump();
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.source[self.cursor..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.cursor += character.len_utf8();
        Some(character)
    }

    fn error(&self, message: &str, start: usize, end: usize) -> LanguageError {
        LanguageError::new(ErrorKind::Parse, message, Span { start, end })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_one(source: &str) -> Expr {
        let expressions = parse(source).expect("parsing should succeed");
        assert_eq!(expressions.len(), 1, "expected exactly one top-level form");
        expressions.into_iter().next().unwrap()
    }

    #[test]
    fn parses_integers_as_exact_numbers() {
        assert!(matches!(parse_one("42").kind, ExprKind::Number(n, Exactness::Exact) if n == 42.0));
    }

    #[test]
    fn decimal_literal_is_parsed_as_exact_rational_or_exact_integer() {
        assert!(matches!(parse_one("3").kind, ExprKind::Number(n, Exactness::Exact) if n == 3.0));
        assert!(matches!(parse_one("3.0").kind, ExprKind::Number(n, Exactness::Exact) if n == 3.0));
        assert!(matches!(parse_one("3.00").kind, ExprKind::Number(n, Exactness::Exact) if n == 3.0));
        assert!(matches!(parse_one("3e0").kind, ExprKind::Number(n, Exactness::Exact) if n == 3.0));
        
        let ExprKind::Rational(rational) = parse_one("-3.5").kind else {
            panic!("expected a rational literal");
        };
        assert_eq!(rational, crate::value::Rational::new(-7, 2).unwrap());
    }

    #[test]
    fn large_integer_literal_uses_arbitrary_precision_without_rounding() {
        let ExprKind::Rational(integer) = parse_one("123456789012345678901234567890").kind else {
            panic!("large exact integer should use the arbitrary-precision path");
        };
        assert_eq!(integer.to_string(), "123456789012345678901234567890");
    }

    #[test]
    fn parses_slash_notation_as_exact_rational() {
        let ExprKind::Rational(rational) = parse_one("5/336").kind else {
            panic!("expected a rational literal");
        };
        assert_eq!(rational, crate::value::Rational::new(5, 336).unwrap());
    }

    #[test]
    fn malformed_decimal_literals_fall_back_to_plain_symbols() {
        for literal in [".", ".e3", "1e", "1e+", "1e-", "1.2.3", "1ee3", "--0.5", "+", "-"] {
            assert!(
                matches!(parse_one(literal).kind, ExprKind::Symbol(s) if &*s == literal),
                "literal {literal:?} should be an ordinary symbol, not a number"
            );
        }
    }

    /// A syntactically valid decimal literal whose exponent exceeds the parser's
    /// resource cap must fail *named* (`NumericOverflow`), never silently become
    /// an ordinary symbol (S3) — and it must refuse to serve as an identifier,
    /// exactly the hole that used to let `(def 1e100001 5)` define a symbol.
    /// Syntaksychno korektnyi desiatkovyi literal, chyia eksponenta perevyshchuie
    /// resursnu mezhu parsera, musi provaliuvatys *nazvano* (`NumericOverflow`),
    /// nikoly ne staiaty movchky zvychainym symvolom (S3) — i musi vidmovliatys
    /// sluzhyty identyfikatorom, tse toi samyi otvir, yakym `(def 1e100001 5)`
    /// ranishe vyznachav symvol.
    #[test]
    fn decimal_literals_past_the_resource_limit_fail_named_not_as_symbols() {
        for literal in ["1e100001", "1e-100001"] {
            let error = parse(literal).expect_err(&format!("{literal} should be refused"));
            assert_eq!(
                error.kind,
                ErrorKind::NumericOverflow,
                "{literal} must be NumericOverflow, not a symbol or Parse"
            );
            assert_eq!((error.span.start, error.span.end), (0, literal.len()));
        }
    }

    /// `(def 1e100001 5)` must NOT define an identifier named `1e100001` — a
    /// valid numeric literal past the resource limit is a parse failure, not a
    /// symbol name. This is the exact regression the S3 fix closes.
    /// `(def 1e100001 5)` NE maie vyznachaty identyfikator na imia `1e100001` —
    /// korektnyi chyslovyi literal ponad resursnu mezhu tse proval parsera, ne
    /// imia symvola. Tse tochno ta rehresiia, yaku zakryvaie fiks S3.
    #[test]
    fn a_giant_decimal_literal_cannot_serve_as_an_identifier() {
        let error = parse("(def 1e100001 5)").expect_err("should be refused");
        assert_eq!(error.kind, ErrorKind::NumericOverflow);
    }

    /// Exponent magnitudes comfortably below the cap must still parse. The
    /// exact ±100000 boundary itself is deliberately *not* exercised here:
    /// reducing `10^100000` by GCD is quadratic in the bignum's decimal
    /// digits (see `bignum.rs`'s `div_rem`) and turns a unit test into a
    /// minutes-long computation — the boundary behavior is already pinned by
    /// the overflow tests above (`1e100001`/`1e-100001`) and the boundary
    /// value itself is an internal DoS-hedge, not a contract fact.
    /// Velychyny eksponenty, komfortno nyzhchi za mezhu, musi vse shche
    /// parsytsia. Tochna mezha ±100000 svidomo *ne* pereviriaietsia tut:
    /// skorochennia `10^100000` za GCD kvadratychne za desiatkovymy tsyframy
    /// bignum (dyv. `div_rem` u `bignum.rs`) i peretvoriuie yunit-test na
    /// bahatokhvylvynne obchyslennia — povedinka mezhovoho znachennia vzhe
    /// zakriplena testamy overflow vyshche (`1e100001`/`1e-100001`), a same
    /// znachennia mezhі — vnutrishnii DoS-zakhyst, ne fakt kontraktu.
    #[test]
    fn decimal_literals_within_the_resource_limit_still_parse() {
        for literal in ["1e1000", "1e-1000", "1.25e1000", "1.25e-1000"] {
            parse(literal).unwrap_or_else(|e| panic!("{literal} is within the limit, failed: {e}"));
        }
    }

    #[test]
    fn decimal_literal_edge_cases_parse_as_exact_numbers() {
        let cases: &[(&str, &str)] = &[
            ("0.0", "0"),
            ("-0.0", "0"),
            (".5", "1/2"),
            ("-.5", "-1/2"),
            ("5.", "5"),
            ("000.500", "1/2"),
            ("1e3", "1000"),
            ("1e-3", "1/1000"),
            ("1.25e2", "125"),
            ("1e100", "10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
            ("1e-100", "1/10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        ];
        for (literal, expected) in cases {
            let expr = parse_one(literal);
            let printed = match &expr.kind {
                ExprKind::Number(n, _) => format!("{n}"),
                ExprKind::Rational(r) => r.to_string(),
                other => panic!("{literal:?} should be a number, got {other:?}"),
            };
            assert_eq!(&printed, expected, "literal {literal:?}");
        }
    }

    #[test]
    fn zero_denominator_falls_back_to_a_plain_symbol() {
        // `1/0` is not a valid Rational (see value::Rational::new), so the reader
        // treats it as an ordinary symbol instead of failing the whole parse.
        // `1/0` ne ye korektnym Rational (dyv. value::Rational::new), tomu reader
        // traktuie yoho yak zvychainyi symvol, a ne provaliuie ves parsynh.
        // `1/0` ist kein gültiges Rational (siehe value::Rational::new), daher
        // behandelt der Reader es als gewöhnliches Symbol statt das Parsing scheitern zu lassen.
        assert!(matches!(parse_one("1/0").kind, ExprKind::Symbol(s) if &*s == "1/0"));
    }

    #[test]
    fn parses_symbols() {
        assert!(matches!(parse_one("foo-bar?").kind, ExprKind::Symbol(s) if &*s == "foo-bar?"));
    }

    #[test]
    fn parses_strings_with_escapes() {
        let ExprKind::String(value) = parse_one(r#""line\n\ttab\"quote""#).kind else {
            panic!("expected a string literal");
        };
        assert_eq!(&*value, "line\n\ttab\"quote");
    }

    /// `\r` used to silently fall through the "unrecognized escape" branch
    /// (drop the backslash, keep the literal letter) — `"\r"` parsed as the
    /// one-character string `"r"`, not carriage-return 0x0D. Found via a
    /// real bug in the fpga-lisp session's assembler.my: code checking
    /// `(eq (string-first s) "\r")` to strip CR silently ate every literal
    /// 'r' character in unrelated text instead. `\r` now joins `\n`/`\t` as
    /// a real recognized escape — the same category, not a new capability.
    #[test]
    fn parses_carriage_return_escape() {
        let ExprKind::String(value) = parse_one(r#""a\rb""#).kind else {
            panic!("expected a string literal");
        };
        assert_eq!(&*value, "a\rb");
    }

    #[test]
    fn unclosed_string_is_a_parse_error() {
        let error = parse(r#""unterminated"#).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Parse);
    }

    #[test]
    fn parses_nested_lists() {
        let ExprKind::List(items) = parse_one("(1 (2 3) 4)").kind else {
            panic!("expected a list");
        };
        assert_eq!(items.len(), 3);
        assert!(matches!(&items[1].kind, ExprKind::List(inner) if inner.len() == 2));
    }

    #[test]
    fn parses_a_dotted_pair() {
        let ExprKind::Pair(head, tail) = parse_one("(1 . 2)").kind else {
            panic!("expected a dotted pair");
        };
        assert!(matches!(head.kind, ExprKind::Number(n, Exactness::Exact) if n == 1.0));
        assert!(matches!(tail.kind, ExprKind::Number(n, Exactness::Exact) if n == 2.0));
    }

    #[test]
    fn parses_a_multi_element_dotted_list_as_nested_pairs() {
        // `(a b . c)` folds right-to-left onto the tail, the same shape
        // `cons` builds at runtime: `Pair(a, Pair(b, c))`.
        let ExprKind::Pair(head, rest) = parse_one("(a b . c)").kind else {
            panic!("expected a dotted pair");
        };
        assert!(matches!(&head.kind, ExprKind::Symbol(s) if &**s == "a"));
        let ExprKind::Pair(inner_head, inner_tail) = &rest.kind else {
            panic!("expected a nested dotted pair");
        };
        assert!(matches!(&inner_head.kind, ExprKind::Symbol(s) if &**s == "b"));
        assert!(matches!(&inner_tail.kind, ExprKind::Symbol(s) if &**s == "c"));
    }

    #[test]
    fn a_lone_dot_outside_a_list_is_an_ordinary_symbol() {
        // Only special between two sub-expressions inside parentheses — a
        // bare top-level `.` has nothing to be a separator between.
        assert!(matches!(parse_one(".").kind, ExprKind::Symbol(s) if &*s == "."));
    }

    #[test]
    fn a_dot_with_nothing_before_it_is_a_parse_error() {
        let error = parse("(. 1)").unwrap_err();
        assert_eq!(error.kind, ErrorKind::Parse);
    }

    #[test]
    fn a_dot_with_nothing_after_it_is_a_parse_error() {
        let error = parse("(1 .)").unwrap_err();
        assert_eq!(error.kind, ErrorKind::Parse);
    }

    #[test]
    fn a_dot_followed_by_more_than_one_tail_expression_is_a_parse_error() {
        let error = parse("(1 . 2 3)").unwrap_err();
        assert_eq!(error.kind, ErrorKind::Parse);
    }

    #[test]
    fn unclosed_list_is_a_parse_error() {
        let error = parse("(1 2 3").unwrap_err();
        assert_eq!(error.kind, ErrorKind::Parse);
    }

    #[test]
    fn unexpected_closing_paren_is_a_parse_error() {
        let error = parse(")").unwrap_err();
        assert_eq!(error.kind, ErrorKind::Parse);
    }

    #[test]
    fn apostrophe_is_no_longer_quote_sugar_but_part_of_symbol() {
        assert!(matches!(parse_one("'x").kind, ExprKind::Symbol(s) if &*s == "'x"));
    }

    #[test]
    fn apostrophe_works_inside_ukrainian_identifiers() {
        assert!(matches!(parse_one("об'єкт").kind, ExprKind::Symbol(s) if &*s == "об'єкт"));
        assert!(matches!(parse_one("зв'язок").kind, ExprKind::Symbol(s) if &*s == "зв'язок"));
        assert!(matches!(parse_one("п'ять").kind, ExprKind::Symbol(s) if &*s == "п'ять"));
    }

    #[test]
    fn semicolon_comments_are_skipped() {
        let expressions = parse("; a comment\n42 ; trailing comment").expect("should parse");
        assert_eq!(expressions.len(), 1);
        assert!(matches!(expressions[0].kind, ExprKind::Number(n, Exactness::Exact) if n == 42.0));
    }

    #[test]
    fn unicode_symbols_and_comments_are_supported() {
        let expressions = parse("; komentar\npryvit").expect("should parse");
        assert!(matches!(&expressions[0].kind, ExprKind::Symbol(s) if &**s == "pryvit"));
    }

    #[test]
    fn parses_multiple_top_level_expressions() {
        let expressions = parse("1 2 3").expect("should parse");
        assert_eq!(expressions.len(), 3);
    }

    #[test]
    fn empty_source_parses_to_no_expressions() {
        assert_eq!(parse("   ; only a comment\n").expect("should parse"), vec![]);
    }
}

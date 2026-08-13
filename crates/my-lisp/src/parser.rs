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
            Some('\'') => self.quoted(start),
            Some('"') => self.string(start),
            Some(_) => self.atom(start),
            None => Err(self.error(
                "expected an expression · ochikuvavsia vyraz · Ausdruck erwartet",
                start,
                start,
            )),
        }
    }

    /// Reader sugar is normalized here, so the evaluator only needs `quote`.
    /// Syntaksychnyi tsukor normalizuietsia tut, tomu obchysliuvachu dostatno `quote`.
    /// Reader-Syntaxzucker wird hier normalisiert, sodass der Evaluator nur `quote` benötigt.
    fn quoted(&mut self, start: usize) -> Result<Expr, LanguageError> {
        self.bump();
        let value = self.expression()?;
        let end = value.span.end;
        Ok(Expr {
            span: Span { start, end },
            kind: ExprKind::List(vec![
                Expr {
                    kind: ExprKind::Symbol("quote".into()),
                    span: Span {
                        start,
                        end: start + 1,
                    },
                },
                value,
            ].into()),
        })
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
        // inexact (PLAN.md item 10, Path A) — the rule keys off literal
        // *syntax* ('.', 'e'/'E'), not "does the value happen to be a whole
        // number," so a future `3e0`/`1.2e3` is inexact without needing a
        // decimal point, and `3.0`/`3.00`/`3.000` are all the same inexact
        // value regardless of trailing zeros.
        // Tsilyi literal → exact; desiatkovyi chy literal v eksponentsiinii
        // notatsii → inexact (PLAN.md, punkt 10, shliakh A) — pravylo dyvytsia
        // na *syntaksys* napysannia ('.', 'e'/'E'), ne na te, chy znachennia
        // vypadkovo tsile, tozh maibutnii `3e0`/`1.2e3` bude inexact bez
        // potreby v desiatkovii kraptsi, a `3.0`/`3.00`/`3.000` — te same
        // inexact znachennia nezalezhno vid kilkosti nuliv naprykintsi.
        let exactness = |text: &str| {
            if text.contains(['.', 'e', 'E']) {
                Exactness::Inexact
            } else {
                Exactness::Exact
            }
        };
        let kind = if let Some((num, den)) = token.split_once('/') {
            if let Some(r) = crate::value::Rational::from_literal(num, den) {
                ExprKind::Rational(r)
            } else {
                token
                    .parse::<f64>()
                    .map(|n| ExprKind::Number(n, exactness(token)))
                    .unwrap_or_else(|_| ExprKind::Symbol(token.into()))
            }
        } else if exactness(token) == Exactness::Exact {
            // Preserve the compact f64-backed representation only where it is
            // mathematically exact; larger integer literals enter the same
            // arbitrary-precision Rational path as n/1 arithmetic results.
            crate::value::Rational::from_literal(token, "1")
                .map(|integer| match integer.as_precise_i64() {
                    Some(value) => ExprKind::Number(value as f64, Exactness::Exact),
                    None => ExprKind::Rational(integer),
                })
                .unwrap_or_else(|| ExprKind::Symbol(token.into()))
        } else {
            token.parse::<f64>()
                .map(|n| ExprKind::Number(n, Exactness::Inexact))
                .unwrap_or_else(|_| ExprKind::Symbol(token.into()))
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
    fn parses_integers_and_floats_as_numbers() {
        assert!(matches!(parse_one("42").kind, ExprKind::Number(n, Exactness::Exact) if n == 42.0));
        assert!(matches!(parse_one("-3.5").kind, ExprKind::Number(n, Exactness::Inexact) if n == -3.5));
    }

    #[test]
    fn integer_literal_is_exact_decimal_literal_is_inexact() {
        assert!(matches!(parse_one("3").kind, ExprKind::Number(n, Exactness::Exact) if n == 3.0));
        assert!(matches!(parse_one("3.0").kind, ExprKind::Number(n, Exactness::Inexact) if n == 3.0));
        assert!(matches!(parse_one("3.00").kind, ExprKind::Number(n, Exactness::Inexact) if n == 3.0));
        assert!(matches!(parse_one("3e0").kind, ExprKind::Number(n, Exactness::Inexact) if n == 3.0));
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
    fn quote_sugar_desugars_to_quote_form() {
        let ExprKind::List(items) = parse_one("'(1 2)").kind else {
            panic!("expected a list");
        };
        assert_eq!(items.len(), 2);
        assert!(matches!(&items[0].kind, ExprKind::Symbol(s) if &**s == "quote"));
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

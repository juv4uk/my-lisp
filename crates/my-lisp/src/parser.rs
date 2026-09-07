use crate::{ErrorKind, Exactness, Expr, ExprKind, LanguageError, Span};
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
    let mut parser = Parser {
        source,
        cursor: 0,
        depth: 0,
    };
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
    depth: u32,
}

impl Parser<'_> {
    fn expression(&mut self) -> Result<Expr, LanguageError> {
        self.skip_ignored();
        let start = self.cursor;
        if self.source[self.cursor..].starts_with("#i32(") {
            return self.numeric_buffer(start, false);
        }
        if self.source[self.cursor..].starts_with("#f32(") {
            return self.numeric_buffer(start, true);
        }
        match self.peek() {
            Some('(') => self.list(start),
            Some(')') => Err(self.error(
                "unexpected closing parenthesis · neochikuvana zakryvna duzhka · unerwartete schließende Klammer",
                start,
                start + 1,
            )),
            Some('"') => self.string(start),
            Some('\'') => self.quote_sugar(start),
            Some(_) => self.atom(start),
            None => Err(self.error(
                "expected an expression · ochikuvavsia vyraz · Ausdruck erwartet",
                start,
                start,
            )),
        }
    }

    /// Reader sugar: `'form` is exactly `(quote form)` in the produced AST.
    /// The evaluator therefore sees the existing canonical QUOTE identity;
    /// the apostrophe introduces no eighth primitive and no duplicate semantics.
    fn quote_sugar(&mut self, start: usize) -> Result<Expr, LanguageError> {
        self.bump();
        self.skip_ignored();
        if self.cursor >= self.source.len() {
            return Err(self.error(
                "expected an expression after apostrophe · pislia apostrofa ochikuietsia vyraz · nach dem Apostroph wird ein Ausdruck erwartet",
                start,
                self.cursor,
            ));
        }
        let quoted = self.expression()?;
        Ok(Expr {
            kind: ExprKind::List(
                vec![
                    Expr {
                        kind: ExprKind::Symbol("quote".into()),
                        span: Span {
                            start,
                            end: start + 1,
                        },
                    },
                    quoted,
                ]
                .into(),
            ),
            span: Span {
                start,
                end: self.cursor,
            },
        })
    }

    fn numeric_buffer(&mut self, start: usize, f32_elements: bool) -> Result<Expr, LanguageError> {
        self.enter(start)?;
        self.cursor += 5;
        let result = (|| {
            let mut elements: Vec<Expr> = Vec::new();
            loop {
                self.skip_ignored();
                if self.peek() == Some(')') {
                    self.bump();
                    let buffer = if f32_elements {
                        let mut values = Vec::with_capacity(elements.len());
                        for element in elements {
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
                            let narrowed = number as f32;
                            if !number.is_finite() || !narrowed.is_finite() {
                                return Err(LanguageError::new(
                                    ErrorKind::NumericOverflow,
                                    "#f32 element is outside the finite binary32 domain",
                                    element.span,
                                ));
                            }
                            values.push(narrowed);
                        }
                        crate::NumericBuffer::F32(values.into())
                    } else {
                        let mut values = Vec::with_capacity(elements.len());
                        for element in elements {
                            let integer = match element.kind {
                                ExprKind::Number(value, Exactness::Exact)
                                    if value.fract() == 0.0 =>
                                {
                                    value as i64
                                }
                                ExprKind::Rational(value) if value.is_integer() => {
                                    value.as_precise_i64().ok_or_else(|| {
                                        LanguageError::new(
                                            ErrorKind::NumericOverflow,
                                            "#i32 element is outside the signed 32-bit range",
                                            element.span,
                                        )
                                    })?
                                }
                                _ => {
                                    return Err(self.error(
                                        "#i32 expects exact integer elements",
                                        element.span.start,
                                        element.span.end,
                                    ))
                                }
                            };
                            values.push(i32::try_from(integer).map_err(|_| {
                                LanguageError::new(
                                    ErrorKind::NumericOverflow,
                                    "#i32 element is outside the signed 32-bit range",
                                    element.span,
                                )
                            })?);
                        }
                        crate::NumericBuffer::I32(values.into())
                    };
                    return Ok(Expr {
                        kind: ExprKind::NumericBuffer(buffer),
                        span: Span {
                            start,
                            end: self.cursor,
                        },
                    });
                }
                if self.peek().is_none() {
                    return Err(self.error("unclosed numeric buffer", start, self.cursor));
                }
                elements.push(self.expression()?);
            }
        })();
        self.depth -= 1;
        result
    }

    fn enter(&mut self, span_start: usize) -> Result<(), LanguageError> {
        self.depth += 1;
        if self.depth > crate::syntax::MAX_STRUCTURE_DEPTH {
            return Err(self.error(
                "nesting exceeds reader limit · hlybyna vkladennia perevyshchuie mezhu chytacha · Verschachtelungstiefe überschreitet die Ressourcengrenze",
                span_start,
                span_start + 1,
            ));
        }
        Ok(())
    }

    fn list(&mut self, start: usize) -> Result<Expr, LanguageError> {
        self.enter(start)?;
        let result = self.list_inner(start);
        self.depth -= 1;
        result
    }

    fn list_inner(&mut self, start: usize) -> Result<Expr, LanguageError> {
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
        let decimal_with_dot = if token.contains(',') && !token.contains('.') {
            Some(token.replace(',', "."))
        } else {
            None
        };
        let decimal_text = decimal_with_dot.as_deref().unwrap_or(token);
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
        } else if token.contains(['.', ',', 'e', 'E']) {
            let kind = match crate::value::Rational::from_decimal_literal(decimal_text) {
                Ok(r) => match r.as_precise_i64() {
                    Some(value) => ExprKind::Number(value as f64, Exactness::Exact),
                    None => ExprKind::Rational(r),
                },
                Err(crate::value::DecimalLiteralError::InvalidSyntax) => {
                    ExprKind::Symbol(token.into())
                }
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

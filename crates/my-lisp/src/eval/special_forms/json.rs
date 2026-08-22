//! `json-parse` — the wire-format decode boundary for talking to
//! OpenAI-compatible HTTP endpoints (PLAN.md item 21's "talk to other AI
//! systems"). Deliberately generic: it knows JSON, nothing about any agent,
//! message schema, or tool protocol — those live in .my libraries on top.
//!
//! Why this cannot be expressed in .my itself: string walking primitives
//! (`string-first`/`string-rest`) make a hand-rolled JSON tokenizer possible,
//! but constructing a character from a numeric codepoint is not expressible —
//! `digit->string` in lib/core.my is a hardcoded ASCII digit table and no
//! primitive maps an integer to an arbitrary character string. `\uXXXX`
//! escapes (emitted by real providers for any non-ASCII content) are therefore
//! the proven kernel gap; decoding them happens here, once, at the I/O
//! boundary.
//!
//! Representation handed back to the language:
//! - object  → alist of dotted pairs `("key" . value)`
//! - array   → proper list
//! - string  → `Value::String`
//! - number  → `Value::Number` (exact when integral, inexact otherwise)
//! - true/false → `Value::Bool`
//! - null    → `Value::Nil`
//!
//! Hand-rolled, no external dependency — this crate's zero-dependency
//! policy (see Cargo.toml) predates and outlives this primitive.

use super::core::exact_arity;
use crate::eval::evaluate;
use crate::{Environment, ErrorKind, Expr, LanguageError, Span, Value};
use std::rc::Rc;

pub(crate) fn evaluate_json_parse(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("json-parse", arguments, 1, span)?;
    let text_value = evaluate(&arguments[0], environment)?;
    let Value::String(ref text) = text_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "json-parse expects a string · json-parse ochikuie riadok · json-parse erwartet einen String",
            arguments[0].span,
        ));
    };
    parse_json(text).map_err(|message| {
        LanguageError::new(ErrorKind::InvalidForm, format!("json-parse: {message}"), span)
    })
}

/// Plain-function form of the canonical JSON decoder, for host adapters
/// (LSP, CLI tooling) that must consume JSON without going through `eval`.
/// This is an extraction of existing logic, not a new capability: the
/// special form above delegates here. The matching *serializer* is
/// deliberately NOT provided — JSON output is a transport concern of each
/// adapter, not my-lisp semantics.
pub fn parse_json(text: &str) -> Result<Value, String> {
    let mut parser = JsonParser { bytes: text.as_bytes(), pos: 0 };
    parser.skip_ws();
    let value = parser.parse_value()?;
    parser.skip_ws();
    if parser.pos != parser.bytes.len() {
        return Err(parser.error("trailing characters after JSON value"));
    }
    Ok(value)
}

struct JsonParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn error(&self, message: &str) -> String {
        format!("json-parse: {message} at byte {}", self.pos)
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek();
        if byte.is_some() {
            self.pos += 1;
        }
        byte
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        if self.bump() == Some(byte) {
            Ok(())
        } else {
            Err(self.error(&format!("expected '{}'", byte as char)))
        }
    }

    fn parse_value(&mut self) -> Result<Value, String> {
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => self.parse_string().map(|s| Value::String(Rc::from(s.as_str()))),
            Some(b't') => self.parse_literal("true", Value::Bool(true)),
            Some(b'f') => self.parse_literal("false", Value::Bool(false)),
            Some(b'n') => self.parse_literal("null", Value::Nil),
            Some(c) if c == b'-' || c.is_ascii_digit() => self.parse_number(),
            _ => Err(self.error("unexpected character")),
        }
    }

    fn parse_literal(&mut self, word: &str, value: Value) -> Result<Value, String> {
        if self.bytes[self.pos..].starts_with(word.as_bytes()) {
            self.pos += word.len();
            Ok(value)
        } else {
            Err(self.error(&format!("invalid literal, expected {word}")))
        }
    }

    fn parse_object(&mut self) -> Result<Value, String> {
        self.expect(b'{')?;
        let mut pairs: Vec<(Value, Value)> = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Value::Nil);
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect(b':')?;
            self.skip_ws();
            let value = self.parse_value()?;
            pairs.push((Value::String(Rc::from(key.as_str())), value));
            self.skip_ws();
            match self.bump() {
                Some(b',') => continue,
                Some(b'}') => break,
                _ => return Err(self.error("expected ',' or '}' in object")),
            }
        }
        // Build the dotted-pair alist right-to-left so each entry is a real
        // `(key . value)` pair, matching language-contract.my's data convention.
        let mut alist = Value::Nil;
        for (key, value) in pairs.into_iter().rev() {
            alist = Value::Pair(Rc::new(Value::Pair(Rc::new(key), Rc::new(value))), Rc::new(alist));
        }
        Ok(alist)
    }

    fn parse_array(&mut self) -> Result<Value, String> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Value::Nil);
        }
        loop {
            self.skip_ws();
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.bump() {
                Some(b',') => continue,
                Some(b']') => break,
                _ => return Err(self.error("expected ',' or ']' in array")),
            }
        }
        Ok(Value::list(items))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            match self.bump() {
                None => return Err(self.error("unterminated string")),
                Some(b'"') => return Ok(out),
                Some(b'\\') => match self.bump() {
                    Some(b'"') => out.push('"'),
                    Some(b'\\') => out.push('\\'),
                    Some(b'/') => out.push('/'),
                    Some(b'b') => out.push('\u{0008}'),
                    Some(b'f') => out.push('\u{000C}'),
                    Some(b'n') => out.push('\n'),
                    Some(b'r') => out.push('\r'),
                    Some(b't') => out.push('\t'),
                    Some(b'u') => {
                        let code = self.parse_hex4()?;
                        // Surrogate pair handling: high surrogate must be
                        // followed by \uDC00-\uDFFF to form one codepoint;
                        // a lone surrogate decodes as replacement char
                        // rather than failing the whole document.
                        let codepoint = if (0xD800..0xDC00).contains(&code) {
                            if self.bytes[self.pos..].starts_with(b"\\u") {
                                self.pos += 2;
                                let low = self.parse_hex4()?;
                                if (0xDC00..0xE000).contains(&low) {
                                    0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00)
                                } else {
                                    0xFFFD
                                }
                            } else {
                                0xFFFD
                            }
                        } else if (0xDC00..0xE000).contains(&code) {
                            0xFFFD
                        } else {
                            code
                        };
                        match char::from_u32(codepoint) {
                            Some(ch) => out.push(ch),
                            None => out.push('\u{FFFD}'),
                        }
                    }
                    _ => return Err(self.error("invalid escape sequence")),
                },
                Some(_byte) => {
                    // Collect one full UTF-8 scalar: continuation bytes
                    // (0b10xxxxxx) append to whatever is being built.
                    let start = self.pos - 1;
                    let mut end = self.pos;
                    while end < self.bytes.len() && self.bytes[end] & 0xC0 == 0x80 {
                        end += 1;
                    }
                    self.pos = end;
                    match std::str::from_utf8(&self.bytes[start..end]) {
                        Ok(text) => out.push_str(text),
                        Err(_) => return Err(self.error("invalid UTF-8 in string")),
                    }
                }
            }
        }
    }

    fn parse_hex4(&mut self) -> Result<u32, String> {
        let mut code: u32 = 0;
        for _ in 0..4 {
            let digit = match self.bump() {
                Some(b) if b.is_ascii_hexdigit() => (b as char).to_digit(16).unwrap(),
                _ => return Err(self.error("invalid \\u escape: expected 4 hex digits")),
            };
            code = code * 16 + digit;
        }
        Ok(code)
    }

    fn parse_number(&mut self) -> Result<Value, String> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }
        let mut integral = true;
        if self.peek() == Some(b'.') {
            integral = false;
            self.pos += 1;
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            integral = false;
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| self.error("invalid number"))?;
        if integral {
            if let Ok(n) = text.parse::<i64>() {
                return Ok(Value::Number(n as f64, crate::Exactness::Exact));
            }
        }
        text.parse::<f64>()
            .map(|n| Value::Number(n, crate::Exactness::Inexact))
            .map_err(|_| self.error("invalid number"))
    }
}

#[cfg(test)]
mod extraction_tests {
    // The plain-function form must behave exactly like the special form:
    // same input, same Value, same failure strictness.
    use super::*;

    #[test]
    fn parse_json_decodes_object_to_dotted_alist() {
        let value = parse_json(r#"{"a": 1, "b": [true, null]}"#).unwrap();
        let a = &value.to_string();
        assert_eq!(a, r#"(("a" . 1) ("b" t ()))"#);
    }

    #[test]
    fn parse_json_rejects_trailing_garbage() {
        assert!(parse_json("{} x").is_err());
        assert!(parse_json("").is_err());
    }
}

use crate::Span;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Parse,
    UnknownSymbol,
    Arity,
    Type,
    InvalidForm,
}

/// Structured errors let the IDE underline the exact source range later.
/// Структурована помилка дозволить IDE підкреслити точне місце в коді.
/// Strukturierte Fehler ermöglichen der IDE später, den genauen Quellbereich zu markieren.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Span,
}

impl LanguageError {
    pub(crate) fn new(kind: ErrorKind, message: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
        }
    }
}

impl fmt::Display for LanguageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} · at / позиція / Stelle {}..{}",
            self.message, self.span.start, self.span.end
        )
    }
}

impl std::error::Error for LanguageError {}

impl LanguageError {
    /// 1-indexed (line, column) of the span's start, counted in chars (not
    /// bytes) so multi-byte UTF-8 source — Cyrillic identifiers included —
    /// still lines up visually with the caret in `render`.
    /// 1-індексовані (рядок, стовпець) початку діапазону, пораховані в
    /// символах (не байтах), щоб багатобайтовий UTF-8-код — включно з
    /// кириличними ідентифікаторами — все одно збігався з "^" у `render`.
    /// 1-indizierte (Zeile, Spalte) des Span-Starts, gezählt in Zeichen
    /// (nicht Bytes), damit mehrbyteiger UTF-8-Quellcode — auch kyrillische
    /// Bezeichner — visuell mit dem "^" in `render` übereinstimmt.
    pub fn line_col(&self, source: &str) -> (usize, usize) {
        let offset = self.span.start.min(source.len());
        let mut line = 1;
        let mut column = 1;
        for ch in source[..offset].chars() {
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        (line, column)
    }

    /// A human-readable, rustc-style rendering: the message, a
    /// line:column location, the offending source line, and a caret
    /// underline spanning the error's width on that line.
    /// Людяне, у стилі rustc, подання: повідомлення, місце у форматі
    /// рядок:стовпець, вихідний рядок коду та підкреслення "^" на ширину
    /// помилки в цьому рядку.
    /// Eine menschenlesbare Darstellung im rustc-Stil: die Meldung, ein
    /// Zeile:Spalte-Ort, die betroffene Quellzeile und eine
    /// "^"-Unterstreichung über die Breite des Fehlers in dieser Zeile.
    pub fn render(&self, source: &str) -> String {
        let (line, column) = self.line_col(source);
        let line_text = source.lines().nth(line - 1).unwrap_or("");
        let span_chars = source[self.span.start.min(source.len())..self.span.end.min(source.len())]
            .chars()
            .count()
            .max(1);
        let gutter = format!("{line}");
        let indent = " ".repeat(gutter.len());
        let caret = " ".repeat(column.saturating_sub(1)) + &"^".repeat(span_chars);
        format!(
            "{message}\n{indent} --> line/рядок/Zeile {line}:{column}\n{indent} |\n{gutter} | {line_text}\n{indent} | {caret}",
            message = self.message,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_col_counts_chars_not_bytes_across_newlines() {
        let source = "(def a 1)\n(car а)"; // second line uses Cyrillic а (2 bytes)
        let error = LanguageError::new(ErrorKind::Type, "boom", Span { start: 15, end: 17 });
        assert_eq!(error.line_col(source), (2, 6));
    }

    #[test]
    fn render_underlines_the_full_span_width() {
        let source = "(car 5)";
        let error = LanguageError::new(ErrorKind::Type, "boom", Span { start: 0, end: 7 });
        let rendered = error.render(source);
        assert!(rendered.contains("1:1"));
        assert!(rendered.contains("(car 5)"));
        assert!(rendered.ends_with(&"^".repeat(7)));
    }

    #[test]
    fn line_col_clamps_offsets_beyond_source_length() {
        let source = "(car";
        let error = LanguageError::new(ErrorKind::Parse, "unexpected eof", Span { start: 999, end: 999 });
        assert_eq!(error.line_col(source), (1, 5));
    }
}

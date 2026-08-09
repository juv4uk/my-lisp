use crate::value::Rational;
use std::rc::Rc;

/// Byte range in the original UTF-8 source.
/// Діапазон байтів у початковому тексті UTF-8.
/// Bytebereich im ursprünglichen UTF-8-Quelltext.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Number(f64),
    Rational(Rational),
    String(Rc<str>),
    Symbol(Rc<str>),
    List(Rc<[Expr]>),
    /// A reader-level dotted pair, `(a . b)` — distinct from `List` because a
    /// proper list is nil-terminated and an improper one isn't. Only ever
    /// produced by a literal `.` between exactly two sub-expressions inside
    /// parentheses; never appears as executable code (only inside `quote`,
    /// or wherever a reader/`read`-style caller asks for data).
    /// Dotted-пара на рівні reader'а, `(a . b)` — окремо від `List`, бо
    /// правильний список nil-термінований, а неправильний — ні. З'являється
    /// лише через літеральну `.` між рівно двома під-виразами всередині
    /// дужок; ніколи не з'являється як виконуваний код (лише всередині
    /// `quote`, чи де завгодно, де викликач читає це як дані через `read`).
    /// Ein Reader-level Dotted Pair, `(a . b)` — getrennt von `List`, weil
    /// eine korrekte Liste nil-terminiert ist, eine unkorrekte nicht. Wird
    /// nur durch einen literalen `.` zwischen genau zwei Teilausdrücken
    /// innerhalb von Klammern erzeugt; erscheint nie als ausführbarer Code
    /// (nur innerhalb von `quote`, oder wo ein Aufrufer es über `read` als
    /// Daten liest).
    Pair(Rc<Expr>, Rc<Expr>),
}

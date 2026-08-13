use crate::value::Rational;
use std::rc::Rc;

/// Byte range in the original UTF-8 source.
/// Diapazon baitiv u pochatkovomu teksti UTF-8.
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

/// Whether a numeric value is a precise quantity or a floating-point
/// approximation — a property of the value itself (PLAN.md item 10, Path A),
/// not of how it happens to print. Set once at the reader (integer literal
/// → `Exact`; decimal/exponential literal → `Inexact`) and propagated by
/// arithmetic's promotion rule (`Exact ⊕ Exact → Exact`, anything touching
/// `Inexact` → `Inexact`), never re-guessed from a result's shape.
/// Chy ye chyslove znachennia tochnoiu velychynoiu, chy nablyzhenniam iz plavaiuchoiu
/// komoiu — vlastyvist samoho znachennia (PLAN.md, punkt 10, shliakh A), ne
/// toho, yak vono drukuietsia. Vstanovliuietsia odyn raz u readeri (tsilyi
/// literal → `Exact`; desiatkovyi/eksponentsiinyi literal → `Inexact`) i
/// poshyriuietsia pravylom promotion v aryfmetytsi (`Exact ⊕ Exact → Exact`,
/// bud-yakyi dotyk do `Inexact` → `Inexact`), nikoly ne vhaduietsia zanovo
/// z formy rezultatu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exactness {
    Exact,
    Inexact,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Number(f64, Exactness),
    Rational(Rational),
    String(Rc<str>),
    Symbol(Rc<str>),
    List(Rc<[Expr]>),
    /// A reader-level dotted pair, `(a . b)` — distinct from `List` because a
    /// proper list is nil-terminated and an improper one isn't. Only ever
    /// produced by a literal `.` between exactly two sub-expressions inside
    /// parentheses; never appears as executable code (only inside `quote`,
    /// or wherever a reader/`read`-style caller asks for data).
    /// Dotted-para na rivni readera, `(a . b)` — okremo vid `List`, bo
    /// pravylnyi spysok nil-terminovanyi, a nepravylnyi — ni. Ziavliaietsia
    /// lyshe cherez literalnu `.` mizh rivno dvoma pid-vyrazamy vseredyni
    /// duzhok; nikoly ne ziavliaietsia yak vykonuvanyi kod (lyshe vseredyni
    /// `quote`, chy de zavhodno, de vyklykach chytaie tse yak dani cherez `read`).
    /// Ein Reader-level Dotted Pair, `(a . b)` — getrennt von `List`, weil
    /// eine korrekte Liste nil-terminiert ist, eine unkorrekte nicht. Wird
    /// nur durch einen literalen `.` zwischen genau zwei Teilausdrücken
    /// innerhalb von Klammern erzeugt; erscheint nie als ausführbarer Code
    /// (nur innerhalb von `quote`, oder wo ein Aufrufer es über `read` als
    /// Daten liest).
    Pair(Rc<Expr>, Rc<Expr>),
}

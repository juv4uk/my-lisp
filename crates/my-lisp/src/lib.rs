//! Independent, capability-free core of the my-lisp language.
//! Nezalezhne yadro movy my-lisp bez dostupu do mozhlyvostei operatsiinoi systemy.
//! Unabhängiger Sprachkern von my-lisp ohne Zugriff auf Betriebssystemfunktionen.
//!
//! The crate deliberately knows nothing about Tauri, files, the network, or UI.
//! Kreit navmysno nichoho ne znaie pro Tauri, faily, merezhu chy interfeis.
//! Das Crate kennt bewusst weder Tauri noch Dateien, Netzwerk oder Benutzeroberfläche.

mod bignum;
mod environment;
mod error;
mod eval;
mod parser;
pub mod semantic;
mod syntax;
mod value;

pub use environment::{Environment, Session};
pub use error::{ErrorKind, LanguageError};
pub use eval::{eval_parsed_expressions, eval_program, EvalResult};
pub use parser::parse;
pub use syntax::{Exactness, Expr, ExprKind, Span};
pub use value::{Closure, Rational, Value};

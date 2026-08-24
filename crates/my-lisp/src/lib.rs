//! Independent, capability-free core of the my-lisp language.
//! Nezalezhne yadro movy my-lisp bez dostupu do mozhlyvostei operatsiinoi systemy.
//! Unabhängiger Sprachkern von my-lisp ohne Zugriff auf Betriebssystemfunktionen.
//!
//! The crate physically contains no OS access: no filesystem, no processes,
//! no sockets. Host capabilities live in the `my-lisp-host` crate and are
//! installed into this core's registry at startup by whichever embedder
//! wants them (the CLI does; WASM does not). See eval/capabilities.rs.

pub mod layout;

mod bignum;
mod environment;
mod error;
pub(crate) mod eval;
mod parser;
mod syntax;
mod value;

pub use environment::{Environment, Session};
pub use error::{ErrorKind, LanguageError};

pub use eval::exact_arity;
pub use eval::{capability_installed, installed_capabilities, register_capability, unregister_capability};
pub use eval::parse_json;
pub use eval::{
    evaluate as eval_expr, eval_parsed_expressions, eval_parsed_expressions_incremental,
    eval_program, eval_program_incremental, EvalResult,
};
pub use parser::parse;
pub use syntax::{Exactness, Expr, ExprKind, Span};
pub use value::{Closure, NumericBuffer, Rational, Value};

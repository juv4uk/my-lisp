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
mod language_items;
mod parser;
pub mod syntax;
mod value;

pub use environment::{Environment, Session};
pub use error::{ErrorKind, LanguageError};
pub use language_items::{language_items, LanguageItem, LanguageItemKind};

pub use eval::exact_arity;
pub use eval::{capability_installed, installed_capabilities, register_capability, unregister_capability};
pub use eval::parse_json;
pub use eval::{
    evaluate as eval_expr, eval_parsed_expressions, eval_parsed_expressions_incremental,
    eval_program, eval_program_incremental, EvalResult,
};
pub use parser::parse;
pub use syntax::fasl::{
    decode_program as fasl_decode_program, encode_program as fasl_encode_program,
};

/// Convenience: FASL-encode already-parsed expressions bound to a source hash.
pub fn fasl_encode(expressions: &[Expr], source_hash: &[u8; 32]) -> Vec<u8> {
    syntax::fasl::encode_program(expressions, source_hash)
}

/// Source-hash helper for FASL producers (sha256 over raw source bytes).
pub fn sha256_source(input: &[u8]) -> [u8; 32] {
    eval::digest_sha256(input)
}
pub use syntax::{Exactness, Expr, ExprKind, Span};
pub use value::{Closure, NumericBuffer, Rational, Value};

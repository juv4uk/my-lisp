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
pub use error::{Classification, ErrorKind, LanguageError};
pub use language_items::{language_items, Arity, LanguageItem, LanguageItemKind};

pub use eval::exact_arity;
pub use eval::parse_json;
pub use eval::{
    capability_installed, installed_capabilities, register_capability, unregister_capability,
};
pub use eval::{
    eval_parsed_expressions, eval_parsed_expressions_incremental, eval_program,
    eval_program_incremental, evaluate as eval_expr, EvalResult,
};
pub use parser::parse;
pub use syntax::fasl::{
    decode_program as fasl_decode_program, encode_program as fasl_encode_program,
};

/// Language-owned macro substrate layer. This must be evaluated before
/// `CORE_LIBRARY_SOURCE`: core.my defines macros such as `and`, `or`, and
/// `let`, and after this layer is present their `defmacro` surface resolves
/// to the macro implemented in my-lisp rather than the evaluator fallback.
pub const MACRO_LIBRARY_SOURCE: &str = include_str!("../../../lib/macro.my");

/// The ordinary my-lisp bootstrap library, evaluated after the macro layer.
pub const CORE_LIBRARY_SOURCE: &str = include_str!("../../../lib/core.my");

/// Language-owned time semantics. Host clocks expose raw observations such as
/// `mono-ns` and `unix-time-now`; this library derives coarser clocks,
/// calendar interpretation, UTC structure, and deadline arithmetic.
pub const TIME_LIBRARY_SOURCE: &str = include_str!("../../../lib/time.my");

/// Load the language-owned macro layer and then the ordinary core library.
///
/// This is the canonical bootstrap order for embedders that want `core.my`.
/// Keeping the order in one API prevents each caller from silently falling
/// back to Rust's compatibility `defmacro` while the migration is in flight.
pub fn load_core_library(session: &mut Session) -> Result<EvalResult, LanguageError> {
    eval_program(MACRO_LIBRARY_SOURCE, session)?;
    eval_program(CORE_LIBRARY_SOURCE, session)
}

/// Load language-owned time semantics into a session that already has the
/// ordinary core library. Keeping this separate from `load_core_library`
/// preserves the closed language core while giving embedders one canonical
/// time-layer loader instead of ad-hoc `include_str!` calls.
pub fn load_time_library(session: &mut Session) -> Result<EvalResult, LanguageError> {
    eval_program(TIME_LIBRARY_SOURCE, session)
}

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

/// Return a half-open, Unicode-scalar-indexed substring with clamped bounds.
///
/// This is the shared implementation behind the language primitive and
/// direct host bindings, so adapters cannot silently drift from language
/// semantics. Argument validation remains the caller's responsibility.
pub fn string_slice_text(text: &str, start: usize, end: usize) -> String {
    if start >= end {
        return String::new();
    }
    text.chars().skip(start).take(end - start).collect()
}

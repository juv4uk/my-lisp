//! The McCarthy primitives (`eq`, `car`, `cdr`, `cons`, `cond`, `quote`'s helper),
//! plus the small compatibility/bootstrap surface and host mechanisms that still
//! require Rust. Ordinary eager operations enter the language through first-class
//! `Value::Builtin` bindings rather than by adding names to the evaluator dispatcher.

mod codepoint;
mod core;
pub(crate) mod digest;
mod io;
pub mod json;
mod strings;

pub use core::exact_arity;
pub(super) use codepoint::{evaluate_codepoint_to_string, evaluate_string_to_codepoint};
pub(super) use core::{
    car_value, cdr_value, cons_values, eq_values, evaluate_cond, evaluate_definition,
    evaluate_defmacro, quoted,
};
pub(super) use digest::evaluate_sha256_hex;
pub(super) use io::{
    evaluate_eval, evaluate_princ, evaluate_print, evaluate_read, evaluate_read_all,
    evaluate_write_to_string,
};
pub(super) use strings::{
    evaluate_string_slice, string_append_values, string_first_values, string_less_than_values,
    string_predicate_values, string_rest_values, string_to_symbol_values, symbol_to_string_values,
};
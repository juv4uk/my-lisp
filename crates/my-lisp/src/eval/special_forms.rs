//! The McCarthy primitives (`eq`, `car`, `cdr`, `cons`, `cond`, `quote`'s helper),
//! plus `def`, `defmacro`, `list`, and the host-capability primitives
//! (I/O, files, TCP, subprocesses, string ops) — split across submodules
//! by category rather than kept as one file, since this used to be the
//! single largest file in the crate. `eval/mod.rs` still calls everything
//! as `special_forms::evaluate_x`; only the internal layout changed.
//! Prymityvy Makkarti (`eq`, `car`, `cdr`, `cons`, `cond`, pomichnyk `quote`),
//! a takozh `def`, `defmacro`, `list` i host-prymityvy (I/O, faily, TCP,
//! pidprotsesy, riadkovi operatsii) — rozkladeni za katehoriiamy po
//! pidmoduliakh, a ne v odnomu faili, yakym tsei fail ranishe buv naibilshym u
//! kreiti. `eval/mod.rs` i dali vyklykaie vse yak `special_forms::evaluate_x`;
//! zminylos lyshe vnutrishnie roztashuvannia.
//! Die McCarthy-Primitive (`eq`, `car`, `cdr`, `cons`, `cond`, Helfer für `quote`),
//! sowie `def`, `defmacro`, `list` und die Host-Capability-Primitive (I/O,
//! Dateien, TCP, Subprozesse, String-Operationen) — nach Kategorie auf
//! Submodule aufgeteilt statt in einer Datei, die zuvor die größte im
//! Crate war. `eval/mod.rs` ruft weiterhin alles als
//! `special_forms::evaluate_x` auf; nur die interne Anordnung hat sich
//! geändert.

mod core;
pub(crate) mod digest;
mod io;
pub mod json;
mod strings;

pub use core::exact_arity;
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
    evaluate_string_append, evaluate_string_first, evaluate_string_less_than,
    evaluate_string_predicate, evaluate_string_rest, evaluate_string_slice,
    evaluate_string_to_symbol, evaluate_symbol_to_string,
};

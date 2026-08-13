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
mod file_io;
mod io;
mod process;
mod strings;
mod tcp;

pub(super) use core::{
    evaluate_car, evaluate_cdr, evaluate_cond, evaluate_cons, evaluate_defmacro,
    evaluate_definition, evaluate_eq, exact_arity, quoted,
};
pub(super) use file_io::{
    evaluate_read_file, evaluate_read_file_bytes, evaluate_write_file, evaluate_write_file_bytes,
};
pub(super) use io::{
    evaluate_eval, evaluate_load, evaluate_princ, evaluate_print, evaluate_read,
    evaluate_read_all, evaluate_write_to_string,
};
pub(super) use process::evaluate_process_run;
pub(super) use strings::{
    evaluate_string_append, evaluate_string_first, evaluate_string_less_than,
    evaluate_string_predicate, evaluate_string_rest, evaluate_string_to_symbol,
    evaluate_symbol_to_string,
};
pub(super) use tcp::{
    evaluate_tcp_accept, evaluate_tcp_close, evaluate_tcp_connect, evaluate_tcp_listen,
    evaluate_tcp_read, evaluate_tcp_write,
};

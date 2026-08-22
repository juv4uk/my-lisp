//! Standalone `my-lisp-lsp` binary — a thin wrapper around the library
//! entrypoint. Kept alongside `my-lisp lsp` so both entrypoints share one
//! implementation.

fn main() {
    my_lisp_lsp::run_stdio()
}

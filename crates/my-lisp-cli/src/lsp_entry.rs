//! lsp_entry.rs - dispatch glue for the `my-lisp lsp` subcommand.
//! Deliberately tiny: all LSP protocol, transport and analysis logic
//! lives in the my-lisp-lsp crate; this module only forwards to its
//! public stdio entrypoint so editors need just one binary.

pub(crate) fn run() {
    my_lisp_lsp::run_stdio()
}

//! my-lisp-lsp — a thin LSP adapter over the canonical my-lisp core.
//!
//! Module boundaries:
//!   transport  — stdio Content-Length framing only (no language, no LSP)
//!   jsonout    — transport-local JSON encoding (protocol concern)
//!   protocol   — JSON-RPC decode into my-lisp `Value` + response builders
//!   analysis   — canonical-parser-backed facts: diagnostics, defs, symbols
//!   server     — dispatch mapping LSP methods onto analysis
//!
//! The server never re-parses .my with anything but my_lisp::parse, never
//! detects definitions by text matching, and never guesses semantics.

use my_lisp_lsp::{protocol, server, transport};
use std::io::{BufReader, BufWriter, Write};
use std::process::exit;
use transport::read_message;

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut input = BufReader::new(stdin.lock());
    let mut output = BufWriter::new(stdout.lock());
    let mut lsp = server::Server::new();

    loop {
        match read_message(&mut input) {
            Ok(Some(message)) => {
                if server::Server::wants_exit(&message) {
                    exit(0);
                }
                for outgoing in lsp.handle_message(&message) {
                    if transport::write_message(&mut output, &outgoing).is_err() {
                        return; // client went away; nothing sensible left to do
                    }
                }
                let _ = output.flush();
            }
            Ok(None) => return,
            Err(_) => {
                // Malformed framing: answer once and stop rather than spin.
                let _ = transport::write_message(
                    &mut output,
                    &protocol::response(&None, None, Some((-32700, "invalid framing".into()))),
                );
                let _ = Write::flush(&mut output);
                return;
            }
        }
    }
}

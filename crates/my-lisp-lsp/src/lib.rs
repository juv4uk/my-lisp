//! Library face of my-lisp-lsp so integration tests (and future
//! embedders) can drive the exact same server loop the stdio binary runs.
pub mod analysis;
pub mod jsonout;
pub mod protocol;
pub mod server;
pub mod transport;

/// Convenience used by end-to-end tests: process raw JSON-RPC texts in
/// order, collecting every outgoing message.
pub struct Harness {
    inner: server::Server,
}

impl Default for Harness {
    fn default() -> Self {
        Self::new()
    }
}

impl Harness {
    pub fn new() -> Self {
        Self { inner: server::Server::new() }
    }
    pub fn feed(&mut self, messages: &[String]) -> Vec<String> {
        let mut out = Vec::new();
        for m in messages {
            if server::Server::wants_exit(m) {
                continue;
            }
            out.extend(self.inner.handle_message(m));
        }
        out
    }
}

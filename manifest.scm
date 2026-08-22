;; guix shell -m manifest.scm
;; Toolchain for my-lisp: Rust core/CLI/WASM, plus TLS certs for cargo.
(specifications->manifest
 (quote ("rust"
   "rust:cargo"
   "nss-certs"
   "git"
   "racket")))

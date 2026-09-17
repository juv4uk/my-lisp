use std::fs;
use std::path::Path;

#[test]
fn read_file_consumes_host_proven_byte_domain_without_revalidating_it() {
    // `read-file-bytes` is a host mechanism returning a proper list built
    // directly from Vec<u8>, each element materialized as Exactness::Exact.
    // `read-file` therefore must not spend a second full pass establishing
    // the already-proven 0..255 byte domain. UTF-8 interpretation remains
    // Lisp-owned through the existing decoder worker.
    let fs_lisp = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../lib/fs.lisp"),
    )
    .expect("lib/fs.lisp must be readable from the host crate tests");

    assert!(
        fs_lisp.contains("(utf8-decode-onto (read-file-bytes path) (quote ()))"),
        "read-file should enter the Lisp UTF-8 decoder after the host-proven byte-domain boundary"
    );
    assert!(
        !fs_lisp.contains("(utf8-decode-string (read-file-bytes path))"),
        "read-file must not repeat generic byte-domain validation for Vec<u8>-derived input"
    );
}

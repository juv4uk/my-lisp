#![cfg(all(any(target_os = "linux", target_os = "windows"), target_arch = "x86_64"))]

use my_lisp::{eval_program, load_core_library, Session};
use my_lisp_host::install;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const CHILD_REGISTER_ENV: &str = "MY_LISP_HOST_ABI_CHILD_REGISTER";
const TEST_NAME: &str = "guest_clobber_of_sysv64_callee_saved_gpr_does_not_corrupt_host";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_lisp_file(path: &str, session: &mut Session) {
    let path = repo_root().join(path);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", path.display()));
    eval_program(&source, session)
        .unwrap_or_else(|error| panic!("{} must load as ordinary my-lisp: {error}", path.display()));
}

fn execute_clobbering_guest(register: &str) {
    install();
    let mut session = Session::default();
    load_core_library(&mut session).expect("core must bootstrap before host ABI witness");
    load_lisp_file("lib/machine/encoding/x86-64.lisp", &mut session);
    load_lisp_file("lib/machine/admission/x86-64.lisp", &mut session);

    let source = format!(
        "(x86-call-admitted-u64 (quote ((mov-r64-imm64 {register} 42) (mov-r64-imm64 rax 7) (ret))) 0)"
    );
    let result = eval_program(&source, &mut session)
        .unwrap_or_else(|error| panic!("admitted guest clobbering {register} must return safely: {error}"));

    assert_eq!(
        result.value.to_string(),
        "7",
        "guest result in RAX must survive host ABI isolation when {register} is clobbered"
    );
}

#[test]
fn guest_clobber_of_sysv64_callee_saved_gpr_does_not_corrupt_host() {
    if let Ok(register) = std::env::var(CHILD_REGISTER_ENV) {
        execute_clobbering_guest(&register);
        return;
    }

    let current_exe = std::env::current_exe().expect("test executable path must be available");
    for register in ["rbx", "rbp", "r12", "r13", "r14", "r15"] {
        let status = Command::new(&current_exe)
            .args(["--exact", TEST_NAME, "--nocapture"])
            .env(CHILD_REGISTER_ENV, register)
            .status()
            .unwrap_or_else(|error| panic!("must spawn isolated {register} ABI witness: {error}"));

        assert!(
            status.success(),
            "guest clobbering SysV64 callee-saved {register} corrupted/crashed the host; child status: {status}"
        );
    }
}

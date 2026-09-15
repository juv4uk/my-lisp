use my_lisp::{eval_program, load_core_library, Session};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn encoder_session() -> Session {
    let path = repo_root().join("lib/machine/encoding/x86-64.lisp");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", path.display()));

    let mut session = Session::default();
    load_core_library(&mut session).expect("core must bootstrap before machine encoder");
    eval_program(&source, &mut session).expect("x86-64 encoder must load as ordinary my-lisp");
    session
}

fn eval_bytes(source: &str, session: &mut Session) -> String {
    eval_program(source, session)
        .unwrap_or_else(|error| panic!("encoder expression failed: {source}: {error}"))
        .value
        .to_string()
}

#[test]
fn lisp_encodes_ret_to_exact_machine_byte() {
    let mut session = encoder_session();
    assert_eq!(eval_bytes("(x86-encode-ret)", &mut session), "(195)");
}

#[test]
fn lisp_encodes_mov_eax_imm32_little_endian() {
    let mut session = encoder_session();
    assert_eq!(
        eval_bytes("(x86-encode-mov-eax-imm32 42)", &mut session),
        "(184 42 0 0 0)"
    );
}

#[test]
fn lisp_encodes_add_rax_rbx_without_external_assembler() {
    let mut session = encoder_session();
    assert_eq!(
        eval_bytes(
            "(x86-encode-add-r64-r64 (quote rax) (quote rbx))",
            &mut session
        ),
        "(72 1 216)"
    );
}

/// #176 TDD requirement: compare emitted bytes against pinned external
/// evidence. lib/machine/xed/vendor/base/xed-isa.txt (pinned at #175's
/// commit) is upstream Intel XED's own encoding for the no-operand
/// RET_NEAR form: `PATTERN : 0xC3 ...`. This proves the Lisp encoder's
/// opcode choice for `(ret)` was not invented independently of the
/// admitted ISA evidence it claims to cover.
#[test]
fn ret_encoding_matches_pinned_xed_pattern_for_ret_near() {
    let mut session = encoder_session();
    let emitted = eval_bytes("(x86-encode-ret)", &mut session);
    assert_eq!(emitted, "(195)", "195 decimal must equal 0xC3");

    let vendor_path = repo_root().join("lib/machine/xed/vendor/base/xed-isa.txt");
    let vendor_source = fs::read_to_string(&vendor_path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", vendor_path.display()));

    // The no-operand near-return block: ICLASS RET_NEAR whose PATTERN has
    // no OPERANDS-affecting immediate (the imm16 stack-adjust variant is a
    // separate block starting `PATTERN : 0xC2 ...` and is not what
    // `x86-encode-ret` claims to cover).
    assert!(
        vendor_source.contains("PATTERN   : 0xC3 DF64() IMMUNE66_LOOP64()"),
        "pinned XED evidence must still contain the exact RET_NEAR/0xC3 pattern this encoder was checked against"
    );
}

/// #176 continued: the group-1 ALU register/register family (OR/AND/SUB/
/// XOR/CMP) sharing ADD's shape, one opcode byte apart, per Intel's
/// canonical group-1 opcode layout confirmed against #175's pinned XED
/// evidence for each ICLASS's `MOD[0b11] MOD=3 REG[rrr] RM[nnn]` form.
#[test]
fn lisp_encodes_the_alu_register_family_with_pinned_opcodes() {
    let mut session = encoder_session();
    for (form, expected, xed_pattern) in [
        (
            "(x86-encode-or-r64-r64 (quote rax) (quote rbx))",
            "(72 9 216)",
            "PATTERN   : 0x09 MOD[0b11] MOD=3 REG[rrr] RM[nnn]",
        ),
        (
            "(x86-encode-and-r64-r64 (quote rax) (quote rbx))",
            "(72 33 216)",
            "PATTERN   : 0x21 MOD[0b11] MOD=3 REG[rrr] RM[nnn]",
        ),
        (
            "(x86-encode-sub-r64-r64 (quote rax) (quote rbx))",
            "(72 41 216)",
            "PATTERN   : 0x29 MOD[0b11] MOD=3 REG[rrr] RM[nnn]",
        ),
        (
            "(x86-encode-xor-r64-r64 (quote rax) (quote rbx))",
            "(72 49 216)",
            "PATTERN   : 0x31 MOD[0b11] MOD=3 REG[rrr] RM[nnn]",
        ),
        (
            "(x86-encode-cmp-r64-r64 (quote rax) (quote rbx))",
            "(72 57 216)",
            "PATTERN   : 0x39 MOD[0b11] MOD=3 REG[rrr] RM[nnn]",
        ),
    ] {
        assert_eq!(eval_bytes(form, &mut session), expected, "form: {form}");

        let vendor_path = repo_root().join("lib/machine/xed/vendor/base/xed-isa.txt");
        let vendor_source = fs::read_to_string(&vendor_path)
            .unwrap_or_else(|error| panic!("{} must exist: {error}", vendor_path.display()));
        assert!(
            vendor_source.contains(xed_pattern),
            "pinned XED evidence must contain the exact pattern {form} was checked against: {xed_pattern}"
        );
    }
}

/// Independent decoder for exactly the PUSH r64 / POP r64 byte shapes this
/// encoder emits: an optional REX prefix (0x40-0x4F, bit 0 = REX.B) followed
/// by a single opcode byte in 0x50-0x5F. This is deliberately a *second*,
/// from-scratch implementation of the bit arithmetic (decode direction, not
/// mirroring the encoder's own construction), so a test built only from the
/// encoder's own math could not pass it by coincidence.
fn decode_push_or_pop(bytes: &[u8]) -> Option<(&'static str, u8)> {
    let (rex_b, rest) = match bytes {
        [rex, rest @ ..] if (0x40..=0x4F).contains(rex) => ((rex & 0x01) != 0, rest),
        rest => (false, rest),
    };
    let [opcode] = rest else { return None };
    let reg_low3 = opcode & 0b0000_0111;
    let reg = reg_low3 | if rex_b { 0b1000 } else { 0 };
    match opcode & 0b1111_1000 {
        0x50 => Some(("push", reg)),
        0x58 => Some(("pop", reg)),
        _ => None,
    }
}

/// #176 continued: PUSH r64 (opcode 0x50+rd) / POP r64 (opcode 0x58+rd),
/// both `DF64()` (default 64-bit operand size in long mode, no REX.W)
/// per #175's pinned XED evidence, with REX.B only for r8-r15. Verified two
/// ways: against the pinned XED PATTERN text, and by independently decoding
/// the emitted bytes back to (mnemonic, register index).
#[test]
fn lisp_encodes_push_and_pop_with_pinned_opcodes_and_independent_decode() {
    let mut session = encoder_session();
    let registers = [
        ("rax", 0u8),
        ("rcx", 1),
        ("rdx", 2),
        ("rbx", 3),
        ("rsp", 4),
        ("rbp", 5),
        ("rsi", 6),
        ("rdi", 7),
        ("r8", 8),
        ("r9", 9),
        ("r10", 10),
        ("r11", 11),
        ("r12", 12),
        ("r13", 13),
        ("r14", 14),
        ("r15", 15),
    ];

    for (register_name, register_code) in registers {
        for (op, mnemonic) in [("push", "push"), ("pop", "pop")] {
            let form = format!("(x86-encode-{op}-r64 (quote {register_name}))");
            let rendered = eval_bytes(&form, &mut session);
            let bytes: Vec<u8> = rendered
                .trim_start_matches('(')
                .trim_end_matches(')')
                .split_whitespace()
                .map(|token| token.parse().expect("byte must be a small integer"))
                .collect();

            let decoded = decode_push_or_pop(&bytes)
                .unwrap_or_else(|| panic!("{form} produced undecodable bytes {bytes:?}"));
            assert_eq!(
                decoded,
                (mnemonic, register_code),
                "{form} round-tripped to {decoded:?} via independent decode, from bytes {bytes:?}"
            );

            if register_code < 8 {
                assert_eq!(bytes.len(), 1, "{form} for a low register must need no REX prefix");
            } else {
                assert_eq!(bytes.len(), 2, "{form} for r8-r15 must carry REX.B");
                assert_eq!(bytes[0], 0x41, "REX.B-only prefix must be exactly 0x41");
            }
        }
    }

    let vendor_path = repo_root().join("lib/machine/xed/vendor/base/xed-isa.txt");
    let vendor_source = fs::read_to_string(&vendor_path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", vendor_path.display()));
    assert!(
        vendor_source.contains("PATTERN   : 0b0101_0 SRM[rrr] REX2=0 DF64()"),
        "pinned XED evidence must contain the exact PUSH r64 pattern this encoder was checked against"
    );
    assert!(
        vendor_source.contains("PATTERN   : 0b0101_1 SRM[rrr] REX2=0 DF64()"),
        "pinned XED evidence must contain the exact POP r64 pattern this encoder was checked against"
    );
}

/// Independent decoder for the INC r64 / DEC r64 group-5 shape this encoder
/// emits: REX.W (0x48 or 0x49) + opcode 0xFF + ModRM(mod=3, reg, rm). `reg`
/// selects INC (0) vs DEC (1) -- it is an opcode extension, not a register
/// operand. A from-scratch decode, independent of the encoder's own
/// construction arithmetic.
fn decode_inc_or_dec(bytes: &[u8]) -> Option<(&'static str, u8)> {
    let [rex, opcode, modrm] = bytes else { return None };
    if *opcode != 0xFF {
        return None;
    }
    let rex_w = (rex & 0x08) != 0;
    let rex_b = (rex & 0x01) != 0;
    if !rex_w || (rex & 0xF0) != 0x40 {
        return None;
    }
    let mod_bits = modrm >> 6;
    let reg_field = (modrm >> 3) & 0b111;
    let rm_field = modrm & 0b111;
    if mod_bits != 0b11 {
        return None;
    }
    let register = rm_field | if rex_b { 0b1000 } else { 0 };
    match reg_field {
        0 => Some(("inc", register)),
        1 => Some(("dec", register)),
        _ => None,
    }
}

/// #176 continued: INC r64 / DEC r64 always go through the group-5 ModRM
/// path (opcode 0xFF, /0 or /1), never the legacy single-byte 0x40+r/
/// 0x48+r form -- that pinned XED evidence itself tags the legacy form
/// `not64`, since those exact byte values are REX prefixes in 64-bit mode.
/// Getting this wrong (emitting the legacy form) would silently corrupt
/// any following instruction's REX prefix, so this is exactly the kind of
/// "illegal encoding for this mode" case the encoder must never produce.
#[test]
fn lisp_encodes_inc_and_dec_via_group5_modrm_never_the_not64_legacy_form() {
    let mut session = encoder_session();
    for (register_name, register_code) in [("rax", 0u8), ("rcx", 1), ("r8", 8), ("r15", 15)] {
        for (op, mnemonic) in [("inc", "inc"), ("dec", "dec")] {
            let form = format!("(x86-encode-{op}-r64 (quote {register_name}))");
            let rendered = eval_bytes(&form, &mut session);
            let bytes: Vec<u8> = rendered
                .trim_start_matches('(')
                .trim_end_matches(')')
                .split_whitespace()
                .map(|token| token.parse().expect("byte must be a small integer"))
                .collect();

            assert_eq!(bytes.len(), 3, "{form} must always be REX+0xFF+ModRM, 3 bytes");
            assert_eq!(bytes[1], 255, "{form} must use group-5 opcode 0xFF, never the not64 legacy form");

            let decoded = decode_inc_or_dec(&bytes)
                .unwrap_or_else(|| panic!("{form} produced undecodable bytes {bytes:?}"));
            assert_eq!(
                decoded,
                (mnemonic, register_code),
                "{form} round-tripped to {decoded:?} via independent decode, from bytes {bytes:?}"
            );
        }
    }

    let vendor_path = repo_root().join("lib/machine/xed/vendor/base/xed-isa.txt");
    let vendor_source = fs::read_to_string(&vendor_path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", vendor_path.display()));
    assert!(
        vendor_source.contains("PATTERN   : 0xFF MOD[0b11] MOD=3 REG[0b000] RM[nnn]"),
        "pinned XED evidence must contain the exact INC group-5 pattern this encoder was checked against"
    );
    assert!(
        vendor_source.contains("PATTERN   : 0xFF MOD[0b11] MOD=3 REG[0b001] RM[nnn]"),
        "pinned XED evidence must contain the exact DEC group-5 pattern this encoder was checked against"
    );
    assert!(
        vendor_source.contains("PATTERN   : 0b0100_0 SRM[rrr] not64"),
        "pinned XED evidence must still tag the legacy single-byte INC form not64 -- if this ever changes, the group-5-only encoding choice needs re-justifying"
    );
}

#[test]
fn encoder_source_contains_no_process_or_assembler_escape_hatch() {
    let path = repo_root().join("lib/machine/encoding/x86-64.lisp");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must exist: {error}", path.display()));
    let lower = source.to_ascii_lowercase();

    for forbidden in ["process-run", "nasm", "as ", "objcopy", "keystone", "iced-x86"] {
        assert!(
            !lower.contains(forbidden),
            "proof encoder must construct bytes in Lisp, not escape through {forbidden}"
        );
    }
}

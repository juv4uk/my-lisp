//! Producer for `mylisp-cml-export.wsm` — the versioned semantic export
//! CML consumes so it stops manually duplicating or inventing my-lisp's
//! own language rules. Design fixed before this code:
//! docs/cml-semantic-export-v1-design.md.
//!
//! my-lisp owns producing this file. my-lisp does not touch cml's own
//! repository or code — per docs/agent-doctrine.md rule 4, a neighboring
//! repo is an external authority, not a file this repo edits.

use my_lisp::semantic_registry_export::{admitted_surfaces_for_semantic_id, SurfaceRow};

/// The first vertical slice's form allow-list: exactly the semantic IDs
/// `tests/fixtures/conformance.my`'s fixture #69 (named def + recursion,
/// `count-down`) exercises. Extending this list is slice 2+, deliberately
/// not done here (docs/agent-doctrine.md rule 7: minimize change surface).
const SLICE_1_FORMS: &[(&str, Role, bool)] = &[
    ("0001", Role::Syntax, false),    // quote
    ("0007", Role::Syntax, false),    // cond
    ("0010", Role::Syntax, false),    // lambda
    ("0011", Role::Syntax, false),    // define
    ("0003", Role::Primitive, true),  // eq
    ("1001", Role::Library, true),    // subtraction
];

#[derive(Clone, Copy)]
enum Role {
    Syntax,
    Primitive,
    Library,
}

impl Role {
    fn as_str(self) -> &'static str {
        match self {
            Role::Syntax => "syntax",
            Role::Primitive => "primitive",
            Role::Library => "library",
        }
    }
}

/// FNV-1a (64-bit) — not cryptographic, a drift-detection digest between
/// trusted collaborators. See the design doc for why this is deliberate,
/// not an oversight.
fn fnv1a_hex(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn render_surfaces(surfaces: &[SurfaceRow]) -> String {
    surfaces
        .iter()
        .map(|s| format!("({} {})", s.namespace, s.name))
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_forms_block() -> String {
    let mut lines = Vec::new();
    for &(id, role, callable) in SLICE_1_FORMS {
        let surfaces = admitted_surfaces_for_semantic_id(id);
        lines.push(format!(
            "    ({id} (surfaces {}) (role {}) (callable {}))",
            render_surfaces(&surfaces),
            role.as_str(),
            if callable { "t" } else { "nil" }
        ));
    }
    lines.join("\n")
}

fn main() {
    let forms_block = render_forms_block();
    let digest = fnv1a_hex(forms_block.as_bytes());

    let output = format!(
        "(cml-export/1\n  (contract (major 6) (minor 0))\n  (digest \"{digest}\")\n  (forms\n{forms_block}))\n"
    );

    print!("{output}");
}

# SANSKRIT-P5-AST-SEMANTIC-IDS / PANINI-P5-PARSER-DESIGN-QUESTIONS

Design proposal, not yet implemented. Written by my-lisp-1, shared with
my-lisp-panini-1 for agreement before any P5 code lands (both have a
stake: the parser lives here, the kāraka model lives there).

## The question

Does SLP1 semantic-call syntax — `(dA :kartf server :karman packet
:sampradAna client)` — need its own reader-level grammar, or does it
parse as ordinary my-lisp data that a later pass resolves into a
`SemanticCall`?

## Finding: the current reader already parses it, unchanged

Checked `crates/my-lisp/src/parser.rs`'s `atom()`: a token is anything
up to the next whitespace, `(`, `)`, or `;` — no delimiter treats `:`
specially. `(dA :kartf server :karman packet :sampradAna client)`
already parses today, with zero grammar changes, as an ordinary
`ExprKind::List` of 7 `ExprKind::Symbol`s: `dA`, `:kartf`, `server`,
`:karman`, `packet`, `:sampradAna`, `client`.

## Recommendation: post-parse resolution, not a grammar change

Add a resolution function, roughly:

```rust
fn resolve_semantic_call(expr: &Expr) -> Option<Result<SemanticCall, SemanticCallError>> {
    let ExprKind::List(items) = &expr.kind else { return None };
    let [head, rest @ ..] = &items[..] else { return None };
    let ExprKind::Symbol(head_sym) = &head.kind else { return None };
    let predicate = atoms::by_slp1(head_sym)?; // None -> not a semantic call, fall through to ordinary eval
    if predicate.category != AtomCategory::Dhatu { return None; }

    let mut roles = Vec::new();
    for pair in rest.chunks(2) {
        let [role_expr, value] = pair else { return Some(Err(/* odd role/value count */)) };
        let ExprKind::Symbol(role_sym) = &role_expr.kind else { return Some(Err(/* role must be a symbol */)) };
        let role_slp1 = role_sym.strip_prefix(':').unwrap_or(role_sym); // accept :kartf or kartf
        roles.push((atoms::by_slp1(role_slp1)?.id, value.clone()));
    }
    Some(SemanticCall::new(predicate.id, roles))
}
```

Called from wherever `eval` currently dispatches on a list's head symbol
(the same 33-arm `match` in `eval/mod.rs` per the Phase 0 audit) — if a
form's head resolves to a dhātu, treat it as a semantic call instead of
an ordinary function application; if `by_slp1` returns `None`, it's not
recognized as a semantic call, fall through unchanged. Old and new syntax
therefore coexist automatically — this *is* the compatibility path spec
§13/§14 describes, arriving earlier than P6 strictly requires it, as a
side effect of not needing a grammar change at all.

## Why not a dedicated grammar form

- Matches spec §2's own caution against premature parser/renaming work —
  a new reader-level construct is exactly the kind of change that should
  wait until it's proven necessary, and it isn't here.
- Matches Phase 5's own DoD line: "parser resolve-ить SLP1 до semantic
  ID" describes a *resolution* step, not a *grammar* step.
- Keeps `parser.rs` and `syntax.rs` completely untouched by the Sanskrit
  migration — every change so far (Phases 1-4) has lived entirely under
  `semantic/`, and this keeps that boundary intact one phase further.
- If a dedicated grammar form is ever needed later (e.g. for a terser
  surface syntax than `:role value` pairs), it can still be added then —
  nothing here forecloses it, it just isn't needed for P5's stated scope.

## Open sub-question, not yet resolved here

Role-keyword spelling: `:kartf` (colon-prefixed, matching the spec's own
worked examples throughout) vs. bare `kartf`. The sketch above accepts
both (`strip_prefix(':')`), but the *canonical* stored form and whether
the colon is required, optional, or forbidden in canonical source is a
smaller follow-up decision, not blocking this one.

## Status

Not implemented — this is the design proposal PANINI-P5-PARSER-DESIGN-
QUESTIONS asked for. `SANSKRIT-P5-AST-SEMANTIC-IDS` itself (writing
`resolve_semantic_call` for real, wiring it into `eval`, tests) is
separate follow-up work, held until my-lisp-panini-1 has had a chance to
react to this proposal.

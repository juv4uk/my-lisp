# Thorough self-audit — my-lisp-1, 2026-08-19

Ran two independent checks on my own repo's code and docs, per the
owner's request to do a real analysis rather than a summary of what's
already known.

## 1. Explore-subagent audit (code + docs drift)

Full findings recorded in this commit's diff. Summary, ranked:

- **Confirmed, fixed:** `docs/conformance-tier-map.md` and
  `docs/language-core-axioms.md:205` both undercounted
  `tests/fixtures/conformance.my` badly — the file has grown to 193
  fixtures (verified: `grep -c "^((expr"`) while the tier-map table
  stopped tracking around 65–91 and the axioms doc's "Done, 2026-08-09"
  note still said 66. Same class of bug as the `AGENTS.md`
  contract-1.0-vs-2.0 drift and the contradictory G8 tag I already
  fixed today. `conformance-tier-map.md` was also still using the
  pre-2.0 `'expr` quote-sugar the language contract removed — another
  instance of exactly the migration gap I was just discussing with the
  owner for `shiva-sutras`. Fixed by marking the tier-map as a stale
  historical snapshot (not hand-regenerating 193 rows — that would
  recreate the exact parallel-source-of-truth problem
  `docs/agent-doctrine.md` warns against) and pointing to
  `conformance.my`'s own fields as current.
- **Worth checking, not fixed:** `language-contract.my:73`'s own note
  wording is ambiguous about whether it's describing what changed *to
  reach* 2.0 or a change *within* 1.0 — flagged, not touched, since the
  fact itself (contract is 2.0) is correct and this is pure wording.
- **Worth checking, not fixed:** `docs/language-core-axioms.md` and
  `docs/capabilities.md` reference `private/CLAUDE.md` and `fpga-lisp/...`
  paths that don't exist inside this checkout — plausibly intentional
  (sibling/private repo references in a multi-repo ecosystem) but
  unverifiable from here. Not my call to resolve unilaterally.
- **Negative finding (reported, not a defect):** no drift found in
  `ErrorKind`'s 7-variant count vs. S2/S3's claim, `Value::is_truthy`
  vs. G8's boundary note, `ExprKind::Pair` vs. G2's dotted-pair note,
  or the build/tier scripts' existence. No `TODO`/`FIXME`/`#[allow(dead_code)]`
  anywhere in `crates/my-lisp/src` or `lib/*.my`.

## 2. Sarvam as a second, differently-suited reviewer

Per `docs/agent-doctrine.md`'s Sarvam section, asked it directly (cold)
what it's actually strong at compared to a general coding assistant,
rather than assuming. Its own honest answer: genuinely better for
Indic-language/Devanagari-handling code and culturally-Indian context,
explicitly *worse* than a generalist model for standard-English Rust
code review. Took that at face value and pointed it at something
actually in its lane: `crates/my-lisp/src/semantic/transliteration.rs`'s
SLP1→IAST table (Sanskrit-specific, not general Rust).

Result: asked whether `f`→ṛ / `F`→ṝ (vocalic r / long vocalic r) is
correct SLP1. It answered "No" with no elaboration, and a follow-up
question calling for a *reasoning-model call without `max_tokens`
often reproduces the empty-`content` bug the Sarvam guide already
documents (one call did return empty; a shorter retry got through).
Did not accept "No" at face value: the standard SLP1 scheme (Hellwig's
original definition, used across Sanskrit-corpus tooling) maps
`f`→ṛ/`F`→ṝ exactly as this repo's table does. This reads as another
Sarvam hallucination on a checkable fact, not a real bug — consistent
with `hypotheses/sarvam-independent-findings-2026-08-18.yaml`'s
existing F-ML-003 finding (Sarvam also got a sutra citation wrong that
day). Not treating this as resolved-by-elaboration since Sarvam gave
none; recording it here rather than silently discarding the
disagreement, per the same discipline as F-ML-003.

## Takeaway

Sarvam is worth continuing to use, but specifically for
Indic-language-adjacent content, not as a general code reviewer — its
own self-assessment matched what actually happened when tested. The
subagent audit found one real, confirmed, now-fixed drift bug and
correctly declined to manufacture findings where none existed.

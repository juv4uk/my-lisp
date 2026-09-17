# Meta semantic registry generator: implementation plan

**Goal:** replace the active Python projection generator with a native my-lisp implementation while preserving the authority boundary and exact generated meaning.

**Design:** `docs/superpowers/specs/2026-09-17-python-to-lisp-meta-registry-design.md`.

1. **RED gate:** change the focused projection CI check to build `my-lisp` and invoke `scripts/generate-meta-semantic-registry.lisp --check`. Verify the branch PR fails because that script does not exist yet.
2. **GREEN generator:** add `scripts/generate-meta-semantic-registry.lisp`, structurally read `lib/surface/semantic-registry.lisp`, normalize the reader-only apostrophe shape, admit only stable/compatibility-only surfaces, deduplicate same-ID spellings, fail on cross-ID collision, and render deterministic output.
3. **Parity:** regenerate `lib/generated/meta-semantic-registry.lisp`; the only intentional textual provenance change is `.py` → `.lisp` in the generator header. Run the new `--check` gate.
4. **Negative witness:** add a `--self-test` mode exercising same-ID dedupe, cross-ID collision rejection, reader-only apostrophe exclusion, candidate/missing exclusion, and deterministic sample rendering. Wire it into focused CI.
5. **Cutover:** replace every active CI invocation of the Python generator, update path classification and live documentation/test references, then delete `scripts/generate-meta-semantic-registry.py`.
6. **Verify:** inspect exact PR diff, run PR workflows, ensure no active `generate-meta-semantic-registry.py` references remain except immutable/historical evidence, and confirm unrelated Python scripts remain untouched for later #76 slices.

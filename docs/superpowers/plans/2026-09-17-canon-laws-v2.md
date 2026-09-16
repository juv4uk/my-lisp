# Canon Laws V2 implementation plan

**Goal:** bring `lib/canon.lisp` back into semantic authority after #217/#218/#244 changed control, structural observation, and Canon 0 semantics.

**Architecture:** keep Canon 0+7 identities and surfaces immutable. Replace historical T/NIL conformance verdicts with explicit Lisp data. Individual laws return `(canon-law-result <law> satisfied|violated)`; aggregate conformance returns `(canon-conformance satisfied|violated)`. Canonical three-part `cond` performs explicit result matching. Host tests only transport outcomes into the existing Lisp-owned witness protocol.

## Task 1 — RED executable Canon V2 witness

Files:
- Create `tests/fixtures/canon-laws-v2-v1.lisp`
- Create `crates/my-lisp/tests/canon_laws_v2_contract.rs`
- Modify `scripts/test-current-semantic-slice.sh`
- Modify `tests/authority-inventory.lisp` if the authority guard requires the new observer to be classified

Steps:
1. Add Lisp-owned rows for Canon 0, atom/eq structural laws, car/cdr laws, quote suppression, explicit-control short-circuit, symbolic peer surface, and aggregate conformance.
2. Expected outputs are explicit Canon result records, never `t`/`()`. 
3. Rust observer loads core + `lib/canon.lisp`, evaluates each row, and asks `witness-verdict`/`witness-status` for the verdict.
4. Add the observer to the fast current-semantic lane.
5. Commit RED and confirm CI fails because current `lib/canon.lisp` still speaks historical T/NIL semantics.

## Task 2 — GREEN Canon 0 + structure + structural observation + control

File:
- Modify `lib/canon.lisp`

Steps:
1. Keep `canon-empty-list` exactly structural `()`.
2. Add Canon-owned record constructors using only ordinary Lisp data.
3. Rewrite law bodies to consume ratified #218 results:
   - `atom` -> `(structural-kind empty-list|atom|pair)`
   - `eq` -> `(identity-relation same|distinct)`
4. Rewrite all control to canonical #217 three-part clauses `(query expected-datum expression)`.
5. Preserve short-circuit/evaluation suppression witnesses.
6. Rewrite symbolic `?:/.?/=?/:/:п/:р/'` witness to the same explicit-result semantics.
7. Aggregate a list of law results recursively without generic truth coercion.
8. Run focused semantic lane and authority guards.

## Task 3 — preserve scope boundaries

Do not in this slice:
- activate #216 exact-Q runtime results (harvest `feat/binary-math-216` separately after Canon is current);
- redesign richer reasoning;
- delete the two-part `cond` migration adapter globally;
- change semantic IDs or peer surfaces;
- touch machine/CML lowering except later replay/salvage.

## Task 4 — branch-value preservation after #229

Preserve/replay before any cleanup:
- `feat/binary-math-216` — 3 commits ahead of main; executable #216 fixtures/tests.
- `research/223-many-valued-logic` — 7 commits ahead; research corpus + Belnap FOUR Lisp prototype.
- open PR #208 — valuable machine `COND + CAR(CONS)` composition, but stale historical control witness; replay after Canon/#216 semantics are current.
- open PR #173 — one Lisp truth shared native/meta/CML; replay against current semantic witness protocol.
- open PR #248/#249 — reasoning slices; replay on current main before merge.
- open PR #252 — keep RED until authority/Windows/WASM failures are root-caused.
- #172/#164 — RED historical probes; retain until #173 value is fully harvested.

**Verification rule:** no branch is deleted merely because it is old. A branch becomes disposable only after `main...branch` has `ahead_by=0` or all unique value is explicitly harvested into a current branch/issue.

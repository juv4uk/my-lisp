# #419 Canon sculpture audit implementation plan

## Goal

Before the 1.0 semantic freeze, test the current Canon/core for irreducibility instead of preserving historical primitive status by inertia.

## Parallel evidence lanes

1. `audit/419-canon-sculpt-map`: machine-readable provisional map of Canon 0 + McCarthy-7.
2. `audit/419-canon-sculpt-consumers`: read-only inventory of live consumers and semantic surfaces.
3. `audit/419-canon-sculpt-witnesses`: inventory existing Lisp-owned executable witnesses; do not create duplicate truth.
4. `audit/419-canon-sculpt-cross-backend`: record which laws are independently observed by native/meta/CML/machine paths.
5. `audit/419-canon-sculpt-circularity`: closed negative criteria that reject fake derivations.
6. `audit/419-canon-sculpt-red`: after the inventories converge, choose exactly one candidate and write the smallest independent failing witness.

## Integration order

Merge evidence-only siblings into `audit/419-canon-sculpt`; reconcile contradictions there. Do not merge this umbrella to `main` until the audit format is stable and current active work has been rechecked for overlap.

## First hypotheses to test

`quote` and `cond` are currently classified as evaluation-control forms rather than assumed ordinary callable operations. The audit asks whether they are fundamental syntax rules or whether one can be macro-derived from a smaller declared control mechanism.

`atom`, `eq`, `car`, and `cdr` remain `insufficient-evidence` until a non-circular lower-concept derivation or an irreducibility argument is executable.

`cons` is only an `irreducible-candidate`, not a conclusion: a stronger aggregate constructor would falsify the classification, but admitting such a stronger constructor may merely move complexity downward.

Canon 0 is classified as a ground value, not an operation.

## RED gate

No production implementation before a RED witness. A valid RED must fail because the proposed lower-concept definition is absent or insufficient, not because of missing files, stale CI routing, or an unrelated active migration.

## Coordination exclusions at plan creation

Do not edit files owned by active work around semantic CI routing (#421), symbol consumer migration (#396/#406/#410/#418), repo tooling (#392), machine harvest (#328), filesystem performance (#402/#403), reasoning subtraction (#409/#413), or Python-to-Lisp generator migration (#317).

## Deletion gate

Deletion/reclassification requires all of:

- independent Lisp-owned observable-law preservation;
- explicit lower-concept closure;
- negative circularity proof;
- relevant cross-backend parity;
- owner review in a separate issue/PR;
- no new host semantic authority.

If any item is missing, the result remains `insufficient-evidence`.
